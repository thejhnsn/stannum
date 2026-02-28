extern crate svg;
extern crate syntect;

use clap::Parser;
use font_kit::family_name::FamilyName;
use font_kit::source::SystemSource;
use stannum::arguments::Arguments;
use stannum::config::{get_syntax, get_theme, list_themes};
use stannum::render::render;
use std::fs;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

fn main() -> std::io::Result<()> {
    let args = Arguments::parse();

    // Load the default syntax and theme sets.
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let mut theme_set = ThemeSet::load_defaults();
    if args.list_themes {
        return list_themes(&mut theme_set);
    }

    let file = args.input.as_ref().expect("This should never happen!");

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

    let syntax = get_syntax(
        &syntax_set,
        file.to_path_buf(),
        args.language.clone(),
        lines[0],
    );

    let document = render(&args, &lines, &syntax_set, syntax, theme, &font);

    // Save the final SVG
    svg::save(args.output, &document)
}
