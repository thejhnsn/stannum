extern crate svg;
extern crate syntect;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use clap::Parser;
use font_kit::family_name::FamilyName;
use font_kit::font::Font;
use font_kit::source::SystemSource;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use svg::node::element::{
    Circle, Definitions, Filter, FilterEffectOffset, Group, Line, Rectangle, Style, TSpan, Text,
};
use svg::node::element::{FilterEffectComposite, FilterEffectFlood, FilterEffectGaussianBlur};
use svg::node::Blob;
use svg::Document;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;
use tin::arguments::{Arguments, Decorations};

enum WindowButtonStyle {
    MacOS(Circle, Circle, Circle),   // red, yellow, green
    Windows(Line, Rectangle, Group), // minimize, maximize, close
    None,
}

fn add_window_buttons(
    window_decorations: Decorations,
    width: f32,
    font_color: Color,
) -> WindowButtonStyle {
    match window_decorations {
        Decorations::MacOS => {
            let circle_close = Circle::new()
                .set("cx", 15)
                .set("cy", 15)
                .set("r", 6)
                .set("fill", "#ff605c"); // red
            let circle_minimize = Circle::new()
                .set("cx", 35)
                .set("cy", 15)
                .set("r", 6)
                .set("fill", "#ffbd44"); // yellow
            let circle_zoom = Circle::new()
                .set("cx", 55)
                .set("cy", 15)
                .set("r", 6)
                .set("fill", "#00ca4e"); // green
            WindowButtonStyle::MacOS(circle_close, circle_minimize, circle_zoom)
        }
        Decorations::Windows => {
            let padding = 15.0;
            let length = 10.0;
            let center_x = width - padding;
            let center_y = padding;
            let minimize = Line::new()
                // the line starts at (start of the maximize button) - padding - length
                // -> rect_x - length - padding
                // and it ends at the same x position + length
                .set("x1", center_x - 2.5 * length - 2.0 * padding)
                .set("y1", 15)
                .set("x2", center_x - 1.5 * length - 2.0 * padding)
                .set("y2", 15)
                .set("stroke", rgb_to_hex(font_color))
                .set("stroke-width", 2);
            let maximize = Rectangle::new()
                // the rectangle starts at (start of the minimize button) - padding - length
                // -> close_x1 - length - padding
                .set("x", center_x - 1.5 * length - padding)
                .set("y", 10)
                .set("rx", 2)
                .set("ry", 2)
                .set("width", 10)
                .set("height", 10)
                .set("fill", "none")
                .set("stroke", rgb_to_hex(font_color))
                .set("stroke-width", 2);
            // calculate start and end points for the close button
            let close_line1 = Line::new()
                .set("x1", center_x - length / 2.0)
                .set("y1", center_y - length / 2.0)
                .set("x2", center_x + length / 2.0)
                .set("y2", center_y + length / 2.0)
                .set("stroke", rgb_to_hex(font_color))
                .set("stroke-width", 2);
            let close_line2 = Line::new()
                .set("x1", center_x - length / 2.0)
                .set("y1", center_y + length / 2.0)
                .set("x2", center_x + length / 2.0)
                .set("y2", center_y - length / 2.0)
                .set("stroke", rgb_to_hex(font_color))
                .set("stroke-width", 2);
            let close = Group::new().add(close_line1).add(close_line2);
            WindowButtonStyle::Windows(minimize, maximize, close)
        }
        _ => WindowButtonStyle::None,
    }
}

fn add_window_title(
    window_title: Option<String>,
    font: &String,
    font_color: Color,
    rect_width: f32,
) -> Text {
    let header_text = Text::new(window_title.unwrap().as_str())
        .set("x", rect_width / 2.0)
        .set("y", 15)
        .set("dominant-baseline", "middle")
        .set("text-anchor", "middle")
        .set("font-family", font.as_str())
        .set("font-size", 14)
        .set("font-weight", "bold")
        .set("fill", rgb_to_hex(font_color));
    header_text
}

fn embed_font(font: Font, font_name: &str) -> Style {
    let font_bytes = font
        .copy_font_data()
        .expect("Failed to embed font")
        .to_vec();
    let base64_font = STANDARD.encode(&font_bytes);
    let font_face = format!(
        r#"
        @font-face {{
            font-family: '{}';
            src: url(data:font/woff2;base64,{}) format('woff2');
        }}
        "#,
        font_name, base64_font
    );
    Style::new(font_face)
}

fn get_syntax(syntax_set: &SyntaxSet, file: PathBuf, language: Option<String>) -> &SyntaxReference {
    let mut file_extension = if let Some(extension) = file.extension().and_then(OsStr::to_str) {
        extension
    } else {
        "txt"
    };

    let language = if let Some(extension) = language {
        extension
    } else {
        "".to_string()
    };

    if !language.is_empty() {
        file_extension = language.as_str();
    }

    // TODO: Find syntax by first line (shebang) could also be added here
    // Choose syntax and theme
    let syntax = if let Some(syn_ext) = syntax_set.find_syntax_by_extension(file_extension) {
        syn_ext
    } else {
        // TODO: Figure out how to normalize the names according to syntect (as providing
        // --language rust does not work, while --language Rust does)
        if let Some(syn_name) = syntax_set.find_syntax_by_name(file_extension) {
            syn_name
        } else {
            syntax_set.find_syntax_plain_text()
        }
    };
    syntax
}

fn get_shadow(
    shadow_blur: f32,
    shadow_color: String,
    shadow_opacity: f32,
    shadow_offset_x: f32,
    shadow_offset_y: f32,
) -> Filter {
    let flood = FilterEffectFlood::new()
        .set("result", "flood")
        .set("in", "SourceGraphic")
        .set("flood-opacity", shadow_opacity)
        .set("flood-color", shadow_color);
    let blur = FilterEffectGaussianBlur::new()
        .set("result", "blur")
        .set("in", "SourceGraphic")
        .set("stdDeviation", shadow_blur);
    let offset = FilterEffectOffset::new()
        .set("result", "offset")
        .set("in", "blur")
        .set("dx", shadow_offset_x)
        .set("dy", shadow_offset_y);
    let comp1 = FilterEffectComposite::new()
        .set("result", "comp1")
        .set("operator", "in")
        .set("in", "flood")
        .set("in2", "offset");
    let comp2 = FilterEffectComposite::new()
        .set("in", "SourceGraphic")
        .set("in2", "comp1");
    Filter::new()
        .set("id", "shadow")
        .add(flood)
        .add(blur)
        .add(offset)
        .add(comp1)
        .add(comp2)
}

// Utility function to convert a syntect Color to a HEX string.
fn rgb_to_hex(color: Color) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b)
}

fn main() -> std::io::Result<()> {
    let args = Arguments::parse();
    let file = args.input;

    // Load the font from the system for width calculations.
    let source = SystemSource::new();
    let handle = source
        .select_best_match(
            &[FamilyName::Title(args.font.clone())],
            &font_kit::properties::Properties::new(),
        )
        .unwrap();
    let font = handle.load().unwrap();
    let font_size = 14.0;
    let font_scale = font_size / font.metrics().units_per_em as f32;

    // TODO: extract code parsing below to a function

    // Load the default syntax and theme sets.
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let theme_set = ThemeSet::load_defaults();

    let syntax = get_syntax(&syntax_set, file.clone(), args.language);

    let theme = &theme_set.themes[&args.theme];

    // Use the theme's background color if defined; otherwise fallback to white.
    let bg_color = theme.settings.background.unwrap_or(Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    });
    let bg_fill = rgb_to_hex(bg_color);

    // Read the source code from a file.
    let code = fs::read_to_string(file)?;
    let lines: Vec<&str> = LinesWithEndings::from(&code).collect();

    // TODO: Should probably not be hardcoded, adjust this according to the shadow offset and
    // blur...
    // Prepare the overall text element. Provide an empty string as initial content.
    let mut fonts_str = String::new();
    fonts_str.push_str(&args.font);
    fonts_str.push_str(", monospace"); // fallback font
    let mut text_elem = Text::new("")
        .set("x", 20)
        .set("y", 20)
        .set("font-family", fonts_str.as_str())
        .set("font-size", font_size)
        .set("xml:space", "preserve")
        .set("fill", "black");

    // Define the line height in pixels.
    // FIXME: magic numbers
    let line_height = 18;
    let side_padding = 20.0;
    let mut current_x = 0.0;
    let mut current_y = 20.0 + 30.0;
    let line_numbers = args.line_numbers;
    // If line numbers are enabled, calculate the width of the line number column.
    let line_number_width = if line_numbers {
        lines.len().to_string().len()
    } else {
        0
    };

    // Create a HighlightLines instance.
    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut current_line = 0;
    // Process each line from the file.
    for line in lines {
        current_line += 1;
        // Get highlighted regions: Vec<(Style, &str)>
        let regions = highlighter
            .highlight_line(line, &syntax_set)
            .expect("Failed to highlight line");

        // Create an empty string for the line's content
        let mut line_content = String::new();
        if line_numbers {
            // Add the line number to the beginning of the line.
            let line_number = format!("{:>width$}  ", current_line, width = line_number_width);
            let line_number_tspan =
                TSpan::new(line_number).set("fill", rgb_to_hex(theme.settings.foreground.unwrap()));
            line_content = format!("{}{}", line_content, line_number_tspan.to_string());
        }
        // For each region, create a nested tspan.
        for (region_style, region_text) in regions {
            if region_text == "" && region_text == "\n" {
                continue;
            }
            let fill_color = rgb_to_hex(region_style.foreground);
            let region_tspan = TSpan::new(region_text).set("fill", fill_color);
            line_content = format!("{}{}", line_content, region_tspan.to_string());
        }
        line_content = format!(
            "<tspan x=\"{}\" y=\"{}\">{}</tspan>",
            20, current_y, line_content
        );
        let line_blob = Blob::new(line_content);

        // Add the line tspan to the overall text element.
        text_elem = text_elem.add(line_blob);

        // Calculate the width of the current line.
        // This is only an approximation, as every svg renderer may render text slightly differently.
        let width: f32 = line
            .chars()
            .filter_map(|ch| {
                font.glyph_for_char(ch)
                    .map(|glyph_id| {
                        let advance = font.advance(glyph_id).ok()?;
                        Some(advance.x() * font_scale)
                    })
                    .flatten()
            })
            .sum();

        if current_x < width {
            current_x = width;
        }
        current_y += line_height as f32;
    }
    // two times because of padding on both sides
    // FIXME: somehow there's a little bit more space on the right side...
    current_x += 2.0 * side_padding;

    if current_x < args.min_width {
        current_x = args.min_width;
    }
    let shadow = get_shadow(
        args.shadow_blur,
        args.shadow_color,
        args.shadow_opacity,
        args.shadow_offset_x,
        args.shadow_offset_y,
    );
    let mut defs = Definitions::new().add(shadow);
    let style = if !args.no_shadow {
        "filter:url(#shadow)"
    } else {
        ""
    };
    // Embed the font if requested.
    if args.embed_font {
        let embedding = embed_font(font, args.font.as_str());
        defs = defs.add(embedding);
    }
    // Create a background rectangle using the theme's background color.
    let background = Rectangle::new()
        .set("x", 0)
        .set("y", 0)
        .set("rx", args.corner_radius)
        .set("ry", args.corner_radius)
        .set("width", current_x)
        .set("height", current_y)
        .set("fill", bg_fill)
        .set("style", style);

    // Adjust the viewbox to also fit the shadow
    let start_x = if args.shadow_offset_x < 0.0 && !args.no_shadow {
        args.shadow_offset_x
    } else {
        0.0
    };
    let start_y = if args.shadow_offset_y < 0.0 && !args.no_shadow {
        args.shadow_offset_y
    } else {
        0.0
    };
    if args.shadow_offset_x > 0.0 && !args.no_shadow {
        current_x += args.shadow_offset_x;
    }
    if !args.no_shadow {
        current_y += args.shadow_offset_y.abs();
    }
    // Compose the final SVG document.
    let mut document = Document::new()
        .set("viewBox", (start_x, start_y, current_x, current_y))
        .add(defs)
        .add(background)
        .add(text_elem);
    // Add window title if provided
    if args.window_title != None {
        let header_text = add_window_title(
            args.window_title,
            &fonts_str,
            theme.settings.foreground.unwrap(),
            current_x,
        );
        document = document.add(header_text);
    }
    // Add window decorations if provided
    if args.window_decorations != Decorations::None {
        let window_controls = add_window_buttons(
            args.window_decorations,
            current_x - args.shadow_offset_x,
            theme.settings.foreground.unwrap(),
        );
        match window_controls {
            WindowButtonStyle::MacOS(close, minimize, zoom) => {
                document = document.add(close).add(minimize).add(zoom);
            }
            WindowButtonStyle::Windows(minimize, maximize, close) => {
                document = document.add(close).add(minimize).add(maximize);
            }
            _ => {}
        }
    }

    // Save the final SVG
    svg::save(args.output, &document)?;
    Ok(())
}
