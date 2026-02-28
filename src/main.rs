extern crate svg;
extern crate syntect;

use anyhow::{Context, Result};
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

fn main() {
    // Catch the error here instead of returning it to Rust's default handler
    if let Err(err) = run() {
        // \x1b[1;31m makes "error:" bold and red, just like clap!
        eprintln!("\x1b[1;31merror:\x1b[0m {}", err);

        // Print the underlying cause (if there is one) cleanly
        if let Some(cause) = err.source() {
            eprintln!("  ↳ {}", cause);
        }

        // Exit with a failure code so CI/CD scripts know it failed
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Arguments::parse();

    // Load the default syntax and theme sets.
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let mut theme_set = ThemeSet::load_defaults();
    if args.list_themes {
        list_themes(&mut theme_set)?;
        return Ok(());
    }

    let file = args
        .input
        .as_ref()
        .context("Input file is required unless listing themes")?;

    // Load the font from the system for width calculations.
    let source = SystemSource::new();
    let handle = source
        .select_best_match(
            &[FamilyName::Title(args.font.clone())],
            &font_kit::properties::Properties::new(),
        )
        .context(format!("Font not found: {}", args.font))?;

    let font = handle
        .load()
        .context(format!("Failed to load font: {}", args.font))?;

    let theme = &get_theme(&mut theme_set, &args.theme)?;

    // Read the source code from a file.
    let code =
        fs::read_to_string(file).context(format!("Error reading file {}", file.display()))?;

    let lines: Vec<&str> = LinesWithEndings::from(&code).collect();
    let lines_of_code = lines.len();
    if lines_of_code < 1 {
        anyhow::bail!("Empty file!");
    }

    let syntax = get_syntax(
        &syntax_set,
        file.to_path_buf(),
        args.language.clone(),
        lines[0],
    );

    let document = render(&args, &lines, &syntax_set, syntax, theme, &font)?;

    // Save the final SVG
    svg::save(&args.output, &document).context(format!("Failed to save SVG to {}", args.output))?;
    Ok(())
}
