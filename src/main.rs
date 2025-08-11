extern crate svg;
extern crate syntect;

use clap::Parser;
use font_kit::family_name::FamilyName;
use font_kit::source::SystemSource;
use std::fs;
use svg::node::element::{Definitions, Group, Rectangle, TSpan, Text, Use};
use svg::node::Blob;
use svg::Document;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use tin::arguments::{Arguments, Decorations, HighlightMode};
use tin::components::{
    add_window_buttons, add_window_title, embed_font, get_bounding_box, get_shadow, get_text_width,
};
use tin::config::{get_syntax, get_theme, list_themes};
use tin::util::{rgb_to_hex, rgb_to_yuv, yuv_to_rgb};

const DEFAULT_FONT_SIZE: f32 = 14.0;

fn main() -> std::io::Result<()> {
    let args = Arguments::parse();

    // Load the default syntax and theme sets.
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let mut theme_set = ThemeSet::load_defaults();
    if args.list_themes {
        return list_themes(&mut theme_set);
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
    let font_size = DEFAULT_FONT_SIZE;
    let font_scale = font_size / font.metrics().units_per_em as f32;

    let theme = &get_theme(&mut theme_set, &args.theme);

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

    // Define the color to use for highlighting
    let highlight_color = if let Some(hex) = args.highlight_color {
        hex
    } else {
        let (y, u, v) = rgb_to_yuv(bg_color);
        let modified_y = 1.0 - y * y;
        let mut modified_color = yuv_to_rgb(modified_y, u, v);
        modified_color.a = 50;
        rgb_to_hex(modified_color)
    };

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
    let line_height = font_size + args.line_spacing;
    let side_padding = 20.0;
    let mut current_x = 0.0;

    // check if window title and decorations are enabled, if yes then move the text down by 30.0
    let window_bar_exists = if let Some(window_title) = &args.window_title {
        !window_title.is_empty() || args.window_decorations != Decorations::None
    } else {
        false
    };
    let mut current_y = 20.0 +
        if window_bar_exists {
            30.0
        } else {
            0.0
        };
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

    // INFO: Just some estimations on how much space the line numbers are going to take up
    let width_space_char =
        font.advance(
            font.glyph_for_char(' ')
                .expect("Cannot find glyph_id for ' '"),
        )
            .expect("Cannot find advance for ' '")
            .x() * font_scale;
    // FIXME: For non monospaced fonts
    // Also if the two spaces after the line number ever get changed this needs to be adjusted
    // as well
    let line_number_offset = if line_numbers {
        width_space_char * (2 + line_number_width) as f32
    } else {
        0.0
    };

    let mut highlighted_lines_iter = args
        .highlight_lines
        .unwrap_or(vec![])
        .into_iter()
        .peekable();
    let mut highlighted_cols_iter = args
        .highlight_columns
        .unwrap_or(vec![])
        .into_iter()
        .peekable();
    let mut highlight_group = Group::new();

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
            current_y += line_height;
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
        let width = get_text_width(font.clone(), font_scale, line);

        // Create highlighted background for lines in highlighted_lines_iter
        // FIXME: This somewhat depends on the line height... needs to be adjusted if line height
        // is no longer hardcoded...
        if let Some(&line_number_highlighted) = highlighted_lines_iter.peek() {
            if line_number_highlighted == line_number {
                let _ = highlighted_lines_iter.next();
                match args.highlight_mode {
                    HighlightMode::Fit => {
                        // FIXME: Not pretty... don't know whether there is actually a better way
                        // to do this though
                        let code_start_x: f32 =
                            line.chars().take_while(|c| c.is_whitespace()).count() as f32
                                * width_space_char;
                        let highlight_rect = Rectangle::new()
                            .set("x", 20.0 + code_start_x + line_number_offset)
                            .set("y", current_y - font_size)
                            .set("width", width - code_start_x)
                            .set("height", font_size + 4.0)
                            .set("fill", highlight_color.clone());
                        highlight_group = highlight_group.add(highlight_rect);
                    }
                    HighlightMode::AlignRight | HighlightMode::Full => {
                        // NOTE: We set "x" in the defs, since the x values are the same across all
                        // highlights for these two modes
                        let highlight_rect = Use::new()
                            .set(
                                "y",
                                current_y - font_size
                                    + (if args.line_spacing == 0.0 { 3.0 } else { 0.0 }), // FIXME: this is a very hacky way to center the highlight rect when line_spacing is 0 and highlight mode is align right or full...
                            )
                            .set("href", "#highlightRect");
                        highlight_group = highlight_group.add(highlight_rect);
                    }
                }
            }
        }
        // TODO: Add runtime check to reject invalid end columns (to long for current line)
        while let Some(&(line_number_highlighted, start_column, end_column)) =
            highlighted_cols_iter.peek()
        {
            if line_number_highlighted == line_number {
                let _ = highlighted_cols_iter.next();
                let column_start_offset =
                    get_text_width(font.clone(), font_scale, &line[0..start_column - 1]);
                let column_end_offset =
                    get_text_width(font.clone(), font_scale, &line[0..end_column]);
                let highlight_rect = Rectangle::new()
                    .set("x", 20.0 + line_number_offset + column_start_offset)
                    .set("y", current_y - line_height + 4.0)
                    .set("width", column_end_offset - column_start_offset)
                    .set("height", line_height)
                    .set("fill", highlight_color.clone());
                highlight_group = highlight_group.add(highlight_rect);
            } else {
                break;
            }
        }

        if current_x < width {
            current_x = width;
        }
        current_y += line_height;
    }
    let saved_current_x = current_x;
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
    let mut highlight_rect = Rectangle::new()
        .set("id", "highlightRect")
        .set("height", line_height)
        .set("fill", highlight_color.clone());
    match args.highlight_mode {
        HighlightMode::Full => {
            highlight_rect = highlight_rect.set("width", current_x).set("x", 0);
            defs = defs.add(highlight_rect);
        }
        HighlightMode::AlignRight => {
            highlight_rect = highlight_rect
                .set("width", saved_current_x)
                .set("x", 20.0 + line_number_offset);
            defs = defs.add(highlight_rect);
        }
        _ => {}
    }
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
        .add(highlight_group)
        .add(text_elem);
    // Add window title if provided
    if let Some(window_title) = &args.window_title {
        let header_text =
            add_window_title(window_title, &fonts_str, default_text_color, current_x);
        document = document.add(header_text);
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
