use std::path::PathBuf;

use clap::{Parser, ValueEnum};

/// Possible decorations for the image
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Decorations {
    MacOS,
    Windows,
    None,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ShadowType {
    None,
    DropRight,
    DropLeft,
    TopRight,
    TopLeft,
    Full,
}

#[derive(Parser)]
#[command(version, about = "Create vector images of your source code!", long_about = None)]
pub struct Arguments {
    /// Input filepath
    pub input: PathBuf,

    /// Output filepath
    #[arg(short, long, default_value = "./out.svg")]
    pub output: String,

    /// Define the source language, if not provided inferred by the input file extension
    #[arg(long)]
    pub language: Option<String>,

    /// Set the theme to use
    #[arg(long, default_value = "base16-mocha.dark")]
    pub theme: String,

    /// Set the font
    #[arg(long, default_value = "monospace")]
    pub font: String,

    /// Indicates whether font should be embedded in the SVG
    #[arg(long)]
    pub embed_font: bool,

    /// Indicates whether line numbers are on/off
    #[arg(long)]
    pub line_numbers: bool,

    /// Lines to select from the input file (if not provided -> whole file).
    /// Specified as a comma seperated list and/or as a range of lines.
    /// Example: 1,2,5-20, highlights line 1, 2, and 5 to 20.
    #[arg(long, value_parser = parse_lines)]
    pub lines: Option<std::vec::Vec<u32>>,

    /// Lines to highlight in the image.
    /// Specified as a comma seperated list and/or as a range of lines.
    /// Example: 1,2,5-20, highlights line 1, 2, and 5 to 20.
    #[arg(long, value_parser = parse_lines)]
    pub highlight_lines: Option<std::vec::Vec<u32>>,

    /// Indicates whether the full width of a line should be highlighted or only the part
    /// containing code.
    #[arg(long)]
    pub highlight_full_lines: bool,

    /// Columns to highlight within a given line.
    /// Specified as a semicolon seperated list of triples (#line, #column_start, #column_end).
    /// Example: 1,5,20;10,1,15 highlights columns 5 to 20 in line 1 and columns 1 to 15 in line 10
    #[arg(long, value_parser = parse_line_columns)]
    pub highlight_columns: Option<std::vec::Vec<(u32, u32, u32)>>,

    /// Name displayed at the top of the image
    #[arg(long)]
    pub window_title: Option<String>,

    /// Choose window decorations for the image
    #[arg(long, value_enum, default_value = "mac-os")]
    pub window_decorations: Decorations,

    /// Set the corner radius of the image
    #[arg(long, short = 'r', default_value_t = 8)]
    pub corner_radius: u8,

    /// Set the shadow type
    #[arg(long, value_enum, default_value = "none")]
    pub shadow_type: ShadowType,
}

/// Tries to parse u32 from Option<&str>
fn parse_u32_from_option(next: Option<&str>) -> Result<u32, String> {
    Ok(if let Some(value_str) = next {
        if let Ok(value) = value_str.parse() {
            value
        } else {
            return Err("Invalid integer!".to_string());
        }
    } else {
        return Err("None cannot be converted to an integer!".to_string());
    })
}

/// Converts input list and ranges to vector of line numbers
pub fn parse_lines(value: &str) -> Result<Vec<u32>, String> {
    let mut lines = Vec::new();
    if value.is_empty() {
        return Ok(lines);
    }

    for line in value.split(',') {
        if line.contains('-') {
            let mut range = line.split('-');
            let start = parse_u32_from_option(range.next())?;
            let end = parse_u32_from_option(range.next())?;
            if end < start {
                return Err("Invalid range!".to_string());
            } else if start == 0 {
                return Err("Invalid line: 0!".to_string());
            }
            for line_number in start..=end {
                lines.push(line_number);
            }
        } else if let Ok(line_number) = line.parse::<u32>() {
            if line_number == 0 {
                return Err("Invalid line: 0!".to_string());
            }
            lines.push(line_number);
        } else {
            return Err("Invalid integer!".to_string());
        }
    }

    // Doesn't really make sense to have select or highlight lines multiple times
    lines.sort();
    lines.dedup();

    Ok(lines)
}

/// Converts input list to highlight-columns triples
pub fn parse_line_columns(value: &str) -> Result<Vec<(u32, u32, u32)>, String> {
    let mut line_columns = Vec::new();
    if value.is_empty() {
        return Ok(line_columns);
    }

    for triple in value.split(';') {
        let mut temp = Vec::new();
        for element in triple.split(',') {
            if let Ok(number) = element.parse() {
                temp.push(number);
            } else {
                return Err("Invalid integer!".to_string());
            }
        }
        if temp.len() != 3 {
            return Err("Invalid line-column triple!".to_string());
        }
        let first = temp[0];
        let second = temp[1];
        let third = temp[2];

        if third < second {
            return Err("Invalid column range!".to_string());
        }

        line_columns.push((first, second, third));
    }

    line_columns.sort_by(|(a, _, _), (b, _, _)| a.cmp(b));

    Ok(line_columns)
}
