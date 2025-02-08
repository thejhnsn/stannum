extern crate svg;
extern crate syntect;

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use svg::node::element::{Circle, Rectangle, TSpan, Text};
use svg::Document;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;


// TODO: refactor for argparsing
struct Metadata {
    input: PathBuf,
    output: PathBuf,
    corner_radius: i8,
    language: String, // language extension
    theme: String, // use sublime_syntax theme
    width: i32,
    height: i32,
    padding_x: i32, // TODO: maybe remove
    padding_y: i32, // TODO: maybe remove
    font_size: i32,
    font_family: String,
    highlighted_code: HashMap<i32, (i32, i32)>, // line number -> (x, y)
    line_break: bool, // whether to break too long lines - best to be used with line_numbers
    line_numbers: bool,
    window_buttons: bool, // TODO: use enum instead? for different types of buttons
    window_title: String,
    background_color: Color,
    shadow_color: Color,
    shadow_blur: i32,
    shadow_offset: i32
}

impl Metadata {
    fn new() -> Metadata {
        Metadata {
            input: PathBuf::from(""),
            output: PathBuf::from("./out.svg"),
            corner_radius: 10,
            language: "plain".to_string(),
            theme: "base16-mocha.dark".to_string(),
            width: 800,
            height: 600,
            padding_x: 20,
            padding_y: 20,
            font_size: 14,
            font_family: "monospace".to_string(),
            highlighted_code: HashMap::new(),
            line_break: false,
            line_numbers: true,
            window_buttons: true,
            window_title: "".to_string(),
            background_color: Color { r: 255, g: 255, b: 255, a: 255 },
            shadow_color: Color { r: 0, g: 0, b: 0, a: 255 },
            shadow_blur: 5,
            shadow_offset: 5
        }
    }
}

// TODO: also implement other button types
fn add_window_buttons(metadata: &Metadata) -> (Circle, Circle, Circle) {
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
    (circle_close, circle_minimize, circle_zoom)
}

fn add_window_title(metadata: &Metadata, text_color: Color) -> Text {
    let header_text = Text::new("")
        .set("x", metadata.width)
        .set("y", 10+5)
        .set("text-anchor", "middle")
        .set("font-family", metadata.font_family.to_string())
        .set("font-size", metadata.font_size)
        .set("fill", rgb_to_hex(text_color)) // FIXME: take theme's text color
        .add(TSpan::new("")
            .add(svg::node::Text::new(metadata.window_title.as_str())));
    header_text

}

fn add_shadow(metadata: Metadata) {
    unimplemented!()
}

fn parse_code(metadata: &mut Metadata) -> (Text, i32) {
    // return the text element and the height of the text
    unimplemented!()
}

fn print_help() {
    // print help message
    unimplemented!()
}

// Utility function to convert a syntect Color to a HEX string.
fn rgb_to_hex(color: Color) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b)
}

fn main() -> std::io::Result<()> {

    let mut metadata = Metadata::new();
    let file = PathBuf::from("pwnXX.py");
    // TODO: parse command line arguments and update metadata

    // TODO: extract code parsing below to a function

    // Load the default syntax and theme sets.
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let theme_set = ThemeSet::load_defaults();

    // Choose syntax and theme, defaulting to plain text and base16-mocha.dark.
    let syntax = syntax_set
        .find_syntax_by_extension(
            file.extension()
                .and_then(OsStr::to_str)
                .unwrap())
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());
    let theme = &theme_set.themes["base16-mocha.dark"];

    // Use the theme's background color if defined; otherwise fallback to white.
    let bg_color = theme
        .settings
        .background
        .unwrap_or(Color { r: 255, g: 255, b: 255, a: 255 });
    let bg_fill = rgb_to_hex(bg_color);

    // Read the source code from a file.
    let code = fs::read_to_string(file)?;
    let lines: Vec<&str> = LinesWithEndings::from(&code).collect();

    // Prepare the overall text element. Provide an empty string as initial content.
    let mut text_elem = Text::new("")
        .set("x", 20)
        .set("y", 20)
        .set("font-family", "monospace")
        .set("font-size", "14")
        .set("xml:space", "preserve")
        .set("fill", "black");

    // Define the line height in pixels.
    // FIXME: magic numbers
    let line_height = 18;
    let mut current_y = 20 + 30;

    // TODO: Implement line numbers.
    let line_numbers = true;
    // If line numbers are enabled, calculate the width of the line number column.
    let line_number_width = if line_numbers {
        lines.len().to_string().len()
    } else {
        0
    };

    // Create a HighlightLines instance.
    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut i = 0;
    // Process each line from the file.
    for line in lines {
        i += 1;
        // Get highlighted regions: Vec<(Style, &str)>
        let regions = highlighter.highlight_line(line, &syntax_set)
            .expect("Failed to highlight line");

        // Create a tspan for this line with an empty initial string.
        let mut line_tspan = TSpan::new("").set("x", 20)
            .set("y", current_y);
        if line_numbers {
            // Add the line number to the beginning of the line.
            let line_number_tspan = TSpan::new("")
                .set("fill", rgb_to_hex(theme.settings.foreground.unwrap()))
                .add(svg::node::Text::new(format!("{:>width$}  ", i, width = line_number_width)));
            line_tspan = line_tspan.add(line_number_tspan);
        }
        // For each region, create a nested tspan.
        for (region_style, region_text) in regions {
            if region_text == "" {
                continue;
            }
            let fill_color = rgb_to_hex(region_style.foreground);
            let region_tspan = TSpan::new("")
                .set("fill", fill_color)
                .add(svg::node::Text::new(region_text));
            line_tspan = line_tspan.add(region_tspan);
        }

        if line_tspan.get_children().len() == 0 {
            continue;
        }
        // Add the line tspan to the overall text element.
        text_elem = text_elem.add(line_tspan);
        current_y += line_height;
    }

    let header_text = add_window_title(&metadata, theme.settings.foreground.unwrap());

    let window_controls = add_window_buttons(&metadata);

    // Create a background rectangle using the theme's background color.
    // FIXME: take info from struct instead
    let background = Rectangle::new()
        .set("x", 0)
        .set("y", 0)
        .set("rx", 10)
        .set("ry", 10)
        .set("width", 800)
        .set("height", current_y)
        .set("fill", bg_fill);

    // Compose the final SVG document.
    // FIXME: take info from struct instead
    let document = Document::new()
        .set("viewBox", (0, 0, 800, current_y))
        .add(background)
        .add(header_text)
        .add(window_controls.0)
        .add(window_controls.1)
        .add(window_controls.2)
        .add(text_elem);
    let mut output = document.to_string();

    // TODO: extract to a function? and maybe there's a better way to do this...
    // removes unnecessary whitespaces
    // remove \n between </tspan> and <tspan>
    output = output.replace("</tspan>\n<tspan fill", "</tspan><tspan fill");
    // remove \n between </tspan> and </tspan>
    output = output.replace("\n</tspan>", "</tspan>");
    // remove \n between > and <tspan>
    output = output.replace(">\n<tspan", "><tspan");
    // Save the SVG document.
    fs::write("highlighted_code.svg", output)?;
    // svg::save("highlighted_code.svg", &document)?;
    Ok(())
}
