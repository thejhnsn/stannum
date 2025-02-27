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
    Circle, Definitions, Filter, FilterEffectDropShadow, FilterEffectOffset, Group, Line,
    Rectangle, Style, TSpan, Text,
};
use svg::node::element::{FilterEffectComposite, FilterEffectFlood, FilterEffectGaussianBlur};
use svg::node::Blob;
use svg::Document;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color, Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;
use tin::arguments::{Arguments, Decorations};

fn add_window_buttons(window_decorations: Decorations, width: f32, font_color: Color) -> Group {
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
            Group::new()
                .add(circle_close)
                .add(circle_minimize)
                .add(circle_zoom)
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
            Group::new()
                .add(minimize)
                .add(maximize)
                .add(close_line1)
                .add(close_line2)
        }
        _ => panic!("This should never happen..."),
    }
}

fn add_window_title(window_title: &str, font: &str, font_color: Color, rect_width: f32) -> Text {
    let header_text = Text::new(window_title)
        .set("x", rect_width / 2.0)
        .set("y", 15)
        .set("dominant-baseline", "middle")
        .set("text-anchor", "middle")
        .set("font-family", font)
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

fn list_themes(theme_set: &mut ThemeSet) {
    let config_dir = match get_config_directory() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("{:?}", e);
            std::process::exit(1);
        }
    };
    if let Err(e) = theme_set.add_from_folder(&config_dir) {
        eprintln!("{:?}", e);
        std::process::exit(1);
    }
    println!("Available themes:");
    // TODO: maybe sort themes alphabetically and format it using syntect's theme format
    for (th, _) in &theme_set.themes {
        println!("{}", th);
    }
    println!(
        "You can add more themes in the config directory: {}",
        &config_dir
    );
    std::process::exit(0);
}

fn get_config_directory() -> Result<String, String> {
    let home = if cfg!(unix) {
        std::env::var("HOME").map_err(|_| "Could not find home directory!".to_string())
    } else if cfg!(windows) {
        std::env::var("LOCALAPPDATA").map_err(|_| "Could not find home directory!".to_string())
    } else {
        Err("Unsupported operating system!".to_string())
    };
    home.map(|home| {
        if cfg!(unix) {
            format!("{}/.config/tin/themes/", home)
        } else {
            format!("{}\\tin\\themes\\", home)
        }
    })
}

fn get_theme(theme_set: &mut ThemeSet, theme: &String) -> Theme {
    // Check whether theme name is a sublime syntax file or just a name
    let theme_path = PathBuf::from(theme);
    if let Some(extension) = theme_path.extension().and_then(OsStr::to_str) {
        if extension == "tmTheme" {
            // Return theme from file or exit on error
            match ThemeSet::get_theme(theme_path) {
                Ok(th) => return th,
                Err(_) => {
                    eprintln!("Something went wrong while loading the supplied theme!");
                    std::process::exit(1);
                }
            }
        }
    }
    let config_dir = match get_config_directory() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("{:?}", e);
            std::process::exit(1);
        }
    };
    // check if directory exists, if not then create it
    if !PathBuf::from(&config_dir).exists() {
        if let Err(e) = fs::create_dir_all(&config_dir) {
            eprintln!("{:?}", e);
            std::process::exit(1);
        }
    }
    if let Err(e) = theme_set.add_from_folder(config_dir) {
        eprintln!("{:?}", e);
        std::process::exit(1);
    }
    if let Some(th) = theme_set.themes.get(theme) {
        // Don't know how performant this clone is but whatever
        // Maybe just return theme names, and do the lookup in the main function?
        th.clone()
    } else {
        eprintln!("Theme does not exist!");
        std::process::exit(1);
    }
}

fn get_syntax<'a>(
    syntax_set: &'a SyntaxSet,
    file: PathBuf,
    language: Option<String>,
    first_line: &str,
) -> &'a SyntaxReference {
    let file_extension = if let Some(lang) = language {
        lang
    } else if let Some(extension) = file.extension().and_then(OsStr::to_str) {
        extension.to_string()
    } else {
        "".to_string()
    };

    // Choose syntax based on language name/file extension/first line of the file
    let syntax = if let Some(syn_ext) = syntax_set.find_syntax_by_extension(&file_extension) {
        syn_ext
    } else {
        if let Some(syn_name) = syntax_set.find_syntax_by_name(&file_extension) {
            syn_name
        } else if let Some(shebang) = syntax_set.find_syntax_by_first_line(first_line) {
            shebang
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
    composite_shadow: bool,
) -> Filter {
    if composite_shadow {
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
    } else {
        let shadow = FilterEffectDropShadow::new()
            .set("stdDeviation", shadow_blur)
            .set("flood-color", shadow_color)
            .set("flood-opacity", shadow_opacity)
            .set("dx", shadow_offset_x)
            .set("dy", shadow_offset_y);
        Filter::new().set("id", "shadow").add(shadow)
    }
}

fn get_bounding_box(
    shadow: bool,
    shadow_offset_x: f32,
    shadow_offset_y: f32,
    mut current_x: f32,
    mut current_y: f32,
) -> (f32, f32, f32, f32) {
    // FIXME: I don't really how to calculate this properly, seems to be clipping/cutoff no matter
    // how large the viebox is (e.g. using a value of 10 for the shadow blur)
    // May also just be implementation defined (maybe some svg renderer renders this correctly???)
    // To large/small offsets also cause clipping adjusting the viewbox doesn't help here either

    // Adjust the viewbox to also fit the shadow
    if shadow {
        let start_x = if shadow_offset_x < 0.0 {
            shadow_offset_x
        } else {
            0.0
        };
        let start_y = if shadow_offset_y < 0.0 {
            shadow_offset_y
        } else {
            0.0
        };
        current_x += shadow_offset_x.abs();
        current_y += shadow_offset_y.abs();
        (start_x, start_y, current_x, current_y)
    } else {
        (0.0, 0.0, current_x, current_y)
    }
}

// Utility function to convert a syntect Color to a HEX string.
fn rgb_to_hex(color: Color) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b)
}

fn main() -> std::io::Result<()> {
    let args = Arguments::parse();

    // Load the default syntax and theme sets.
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let mut theme_set = ThemeSet::load_defaults();
    if args.list_themes {
        list_themes(&mut theme_set);
    }

    let file = args.input.expect("This should never happen!");
    // Whether the image contains a shadow
    let shadow = !args.no_shadow;

    // Load the font from the system for width calculations.
    let source = SystemSource::new();
    let handle = match source.select_best_match(
        &[FamilyName::Title(args.font.clone())],
        &font_kit::properties::Properties::new(),
    ) {
        Ok(handle) => handle,
        Err(_) => {
            eprintln!("Font not found: {}", args.font);
            std::process::exit(1);
        }
    };
    let font = match handle.load() {
        Ok(font) => font,
        Err(_) => {
            eprintln!("Failed to load font: {}", args.font);
            std::process::exit(1);
        }
    };
    let font_size = 14.0;
    let font_scale = font_size / font.metrics().units_per_em as f32;

    let theme = &get_theme(&mut theme_set, &args.theme);

    if args.list_themes {
        println!("You can add more themes in the config directory!");
        for (th, _) in theme_set.themes {
            println!("{}", th);
        }
        std::process::exit(0);
    }

    // Read the source code from a file.
    let code = match fs::read_to_string(&file) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("Error reading file {}: {}", &file.display(), e);
            std::process::exit(1);
        }
    };
    let lines: Vec<&str> = LinesWithEndings::from(&code).collect();
    let lines_of_code = lines.len();
    if lines_of_code < 1 {
        eprintln!("Empty file, exiting!");
        std::process::exit(1);
    }

    let syntax = get_syntax(&syntax_set, file, args.language, lines[0]);

    // Use the theme's background color if defined; otherwise fallback to white.
    let bg_color = theme.settings.background.unwrap_or_else(|| Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    });
    let bg_fill = rgb_to_hex(bg_color);

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
        lines_of_code.to_string().len()
    } else {
        0
    };

    // Create a HighlightLines instance.
    let mut highlighter = HighlightLines::new(syntax, theme);

    // Use the theme's default text color if defined; otherwise fallback to black.
    // TODO: maybe use the background color to determine the text color (invert it?)
    let default_text_color = theme.settings.foreground.unwrap_or_else(|| Color {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    });

    // Determine which lines should be selected in the image
    let mut selected_lines_iter = (1..=lines_of_code).collect::<Vec<usize>>().into_iter();
    if let Some(sel_lines) = args.lines {
        if *sel_lines.last().unwrap_or(&usize::MAX) > lines_of_code {
            eprintln!("Line out of range!");
            std::process::exit(1);
        }
        selected_lines_iter = sel_lines.into_iter()
    }

    let mut prev_line_number = 0;
    for line_number in selected_lines_iter {
        let line = lines[line_number - 1];
        if line_number != prev_line_number + 1 {
            // Add ... to skipped lines
            let dots = if line_numbers {
                format!("{:>width$}  ...", "", width = line_number_width)
            } else {
                "...".to_string()
            };
            let dots = TSpan::new(dots)
                .set("x", 20)
                .set("y", current_y)
                .set("fill", rgb_to_hex(default_text_color));
            current_y += line_height as f32;
            text_elem = text_elem.add(dots);
            // We need to feed the highlighter every line, otherwise some colors may be incorrect
            for skipped_line in (prev_line_number + 1)..line_number {
                let _ = highlighter.highlight_line(lines[skipped_line], &syntax_set);
            }
        }
        prev_line_number = line_number;
        // Process each line from the file.
        // Get highlighted regions: Vec<(Style, &str)>
        let regions = highlighter
            .highlight_line(line, &syntax_set)
            .expect("Failed to highlight line");

        // Create an empty string for the line's content
        let mut line_content = String::new();
        if line_numbers {
            // Add the line number to the beginning of the line.
            let line_number = format!("{:>width$}  ", line_number, width = line_number_width);
            let line_number_tspan =
                TSpan::new(line_number).set("fill", rgb_to_hex(default_text_color));
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
        // TODO: fallback to line height if width is not available (e.g. for unknown characters in unicode)
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
    let shadow_filter = get_shadow(
        args.shadow_blur,
        args.shadow_color,
        args.shadow_opacity,
        args.shadow_offset_x,
        args.shadow_offset_y,
        args.composite_shadow,
    );
    let mut defs = Definitions::new().add(shadow_filter);
    let style = if shadow { "filter:url(#shadow)" } else { "" };
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

    let bounding_box = get_bounding_box(
        shadow,
        args.shadow_offset_x,
        args.shadow_offset_y,
        current_x,
        current_y,
    );
    // Compose the final SVG document.
    let mut document = Document::new()
        .set("viewBox", bounding_box)
        .add(defs)
        .add(background)
        .add(text_elem);
    // Add window title if provided
    if args.window_title != None {
        if let Some(window_title) = &args.window_title {
            let header_text =
                add_window_title(window_title, &fonts_str, default_text_color, current_x);
            document = document.add(header_text);
        }
    }
    // Add window decorations if provided
    if args.window_decorations != Decorations::None {
        let window_controls = add_window_buttons(
            args.window_decorations,
            current_x - args.shadow_offset_x,
            default_text_color,
        );
        document = document.add(window_controls);
    }

    // Save the final SVG
    svg::save(args.output, &document)
}
