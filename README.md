# Tin

![Hello World Example](assets/hello_world.svg)

Create vector images of your source code! It's like a screenshot, but better. Tin is an alternative
to [Carbon](https://carbon.now.sh) and [Silicon](https://github.com/Aloxaf/silicon) that instead of generating a raster
image, outputs a vector image.
This allows for the image to be resized without losing quality.

## Installation

```bash
cargo install tin
```

You can also download the binary from the [releases](https://github.com/thejhnsn/tin/releases) page or build it
yourself.

## Usage

```bash
tin [OPTIONS] <input>
```

Add themes to the `themes` directory in the config folder. You can find the config folder by running
`tin --list-themes`, which will list the config folder along with the themes in it.
On Windows, the config folder is located at `%LOCALAPPDATA%\tin`, while on Unix systems it is located at
`~/.config/tin`.
Tin uses [TextMate](https://macromates.com/manual/en/language_grammars) themes for syntax highlighting like Sublime
Text, VSCode, bat, silicon, etc.
The theme added should be a `.tmTheme` file.

## Command-Line Options

| Option                                      | Description                                                                       | Default / Possible Values                       |
|---------------------------------------------|-----------------------------------------------------------------------------------|-------------------------------------------------|
| `-o, --output <OUTPUT>`                     | Set the path of the output file                                                   | `./out.svg`                                     |
| `--language <LANGUAGE>`                     | Set the source language, inferred by file extension or first line if not provided | *Auto*                                          |
| `--theme <THEME>`                           | Set the theme                                                                     | `base16-mocha.dark`                             |
| `--list-themes`                             | List all available themes                                                         | N/A                                             |
| `--font <FONT>`                             | Set the font                                                                      | `monospace` on Unix, `Consolas` on Windows      |
| `--embed-font`                              | Embed the font within the SVG file                                                | N/A                                             |
| `--line-spacing <LINE_SPACING>`             | Set the line spacing                                                              | `4`                                             |
| `--line-numbers`                            | Show line numbers                                                                 | N/A                                             |
| `--lines <LINES>`                           | Select specific lines to include in the output (e.g., `1,2,5-20`)                 | *Whole file*                                    |
| `--highlight-color <HIGHLIGHT_COLOR>`       | Set the color used for highlighting lines/columns (hex or CSS color)              | *Based on theme*                                |
| `--highlight-lines <HIGHLIGHT_LINES>`       | Select lines to highlight (e.g., `1,2,5-20`)                                      | N/A                                             |
| `--highlight-mode <HIGHLIGHT_MODE>`         | Choose how highlighted lines are displayed                                        | `fit` (Options: `full`, `fit`, `align-right`)   |
| `--highlight-columns <HIGHLIGHT_COLUMNS>`   | Highlight specific columns in a line (e.g., `1,5,20;10,1,15`)                     | N/A                                             |
| `--window-title <WINDOW_TITLE>`             | Set the window title displayed at the top                                         | N/A                                             |
| `--window-decorations <WINDOW_DECORATIONS>` | Set window decorations                                                            | `mac-os` (Options: `mac-os`, `windows`, `none`) |
| `-r, --corner-radius <CORNER_RADIUS>`       | Set the corner radius of the image                                                | `8`                                             |
| `--min-width <MIN_WIDTH>`                   | Set the minimum width of the code rectangle                                       | `800`                                           |
| `--no-shadow`                               | Disable the background shadow                                                     | N/A                                             |
| `--composite-shadow`                        | Enable composite shadow rendering (for compatibility with Inkscape/PowerPoint)    | N/A                                             |
| `--shadow-blur <SHADOW_BLUR>`               | Set shadow bluriness (`stdDeviation`)                                             | `1`                                             |
| `--shadow-color <SHADOW_COLOR>`             | Set shadow color                                                                  | `#444444`                                       |
| `--shadow-opacity <SHADOW_OPACITY>`         | Set shadow opacity                                                                | `0.5`                                           |
| `-x, --shadow-offset-x <SHADOW_OFFSET_X>`   | Set horizontal shadow offset                                                      | `4` (Negative values: `-x=-5`)                  |
| `-y, --shadow-offset-y <SHADOW_OFFSET_Y>`   | Set vertical shadow offset                                                        | `4` (Negative values: `-y=-5`)                  |
| `-h, --help`                                | Print help information                                                            | N/A                                             |
| `-V, --version`                             | Print version information                                                         | N/A                                             |

## Building

Clone the repository and build the project with Cargo:

```bash
cargo build --release
```

You may need to install the following dependencies:

* `Debian/Ubuntu`: `sudo apt install pkg-config libfreetype6-dev libfontconfig1-dev`
* `Fedora/RHEL`: `sudo dnf install pkg-config freetype-devel fontconfig-devel`

## Exporting to PNG

Converting the svg using InkScape or ImageMagick will most likely not work as expected because both InkScape and
ImageMagick have certain limitations when it comes to rendering SVGs. For converting the svg to png, you can use
[SVG to PNG](https://vincerubinetti.github.io/svg-to-png/), which is a simple open source tool that converts vector
images to PNGs using your browser rendering engine.