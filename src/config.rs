use anyhow::{bail, Context, Result};
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};

pub fn list_themes(theme_set: &mut ThemeSet) -> Result<()> {
    let config_dir = get_config_directory().context("Could not find config directory!")?;

    if PathBuf::from(&config_dir).exists() {
        theme_set
            .add_from_folder(&config_dir)
            .context("Could not load themes!")?;
    }
    let default_print = String::from_utf8_lossy(include_bytes!("../assets/hello_world.rs"));
    // find the longest line in the default print and pad all lines to this length + 2
    let longest_line = default_print
        .lines()
        .map(|line| line.len())
        .max()
        .unwrap_or(80) // default to 80 if for some reason this fails
        + 2;
    let formatted_print = default_print
        .lines()
        .map(|line| format!("{:<width$}", line, width = longest_line))
        .collect::<Vec<String>>()
        .join("\n");

    let mut theme_names_sorted: Vec<String> = theme_set.themes.keys().cloned().collect();
    theme_names_sorted.sort_by_key(|a| a.to_lowercase());
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let syntax = syntax_set
        .find_syntax_by_extension("rs")
        .context("Syntax definition for Rust not found!")?;

    println!("Available themes:");
    for name in theme_names_sorted {
        println!("{}", name);
        let theme = theme_set
            .themes
            .get(&name)
            .context("Theme missing from set")?;
        let mut highlighter = HighlightLines::new(syntax, theme);

        let regions = highlighter
            .highlight_line(&formatted_print, &syntax_set)
            .context("Error highlighting theme sample")?;

        for (region_style, region_text) in regions {
            // set correct background color
            print!(
                "\x1b[48;2;{};{};{}m",
                region_style.background.r, region_style.background.g, region_style.background.b
            );
            print!(
                "\x1b[38;2;{};{};{}m{}\x1b[0m",
                region_style.foreground.r,
                region_style.foreground.g,
                region_style.foreground.b,
                region_text
            );
        }
        // reset background color
        print!("\x1b[0m\n\n");
    }

    println!(
        "You can add more themes in the config directory: {}",
        &config_dir
    );
    Ok(())
}

fn get_config_directory() -> Result<String> {
    let home = if cfg!(unix) {
        std::env::var("HOME").context("Could not find home directory!")?
    } else if cfg!(windows) {
        std::env::var("LOCALAPPDATA").context("Could not find home directory!")?
    } else {
        bail!("Unsupported operating system!")
    };

    if cfg!(unix) {
        Ok(format!("{}/.config/stannum/themes/", home))
    } else {
        Ok(format!("{}\\stannum\\themes\\", home))
    }
}

pub fn get_theme(theme_set: &mut ThemeSet, theme: &String) -> Result<Theme> {
    // Check whether theme name is a sublime syntax file or just a name
    let theme_path = PathBuf::from(theme);
    if let Some(extension) = theme_path.extension().and_then(OsStr::to_str) {
        if extension == "tmTheme" {
            // Return theme from file or exit on error
            return ThemeSet::get_theme(&theme_path)
                .context("Something went wrong while loading the supplied theme!");
        }
    }
    let config_dir = get_config_directory()?;

    // check if directory exists, if not then create it
    if !PathBuf::from(&config_dir).exists() {
        fs::create_dir_all(&config_dir).context(format!(
            "Could not create config directory '{}'",
            config_dir
        ))?;
    }

    theme_set
        .add_from_folder(&config_dir)
        .context(format!("Could not load themes from '{}'", config_dir))?;

    if let Some(th) = theme_set.themes.get(theme) {
        // Don't know how performant this clone is but whatever
        // Maybe just return theme names, and do the lookup in the main function?
        Ok(th.clone())
    } else {
        bail!("Theme '{}' does not exist!", theme);
    }
}

pub fn get_syntax<'a>(
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
    } else if let Some(syn_name) = syntax_set.find_syntax_by_name(&file_extension) {
        syn_name
    } else if let Some(shebang) = syntax_set.find_syntax_by_first_line(first_line) {
        shebang
    } else {
        syntax_set.find_syntax_plain_text()
    };
    syntax
}
