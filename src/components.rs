use super::arguments::Decorations;
use super::util::rgb_to_hex;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use font_kit::font::Font;
use svg::node::element::{Circle, Group, Line, Rectangle, Style, Text};
use svg::node::element::{
    Filter, FilterEffectComposite, FilterEffectDropShadow, FilterEffectFlood,
    FilterEffectGaussianBlur, FilterEffectOffset,
};
use syntect::highlighting::Color;
pub fn add_window_buttons(window_decorations: Decorations, width: f32, font_color: Color) -> Group {
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
        Decorations::None => Group::new(),
    }
}

pub fn add_window_title(
    window_title: &str,
    font: &str,
    font_color: Color,
    rect_width: f32,
) -> Text {
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

pub fn embed_font(font: Font, font_name: &str) -> Style {
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

pub fn get_shadow(
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

pub fn get_bounding_box(
    shadow: bool,
    shadow_offset_x: f32,
    shadow_offset_y: f32,
    mut current_x: f32,
    mut current_y: f32,
) -> (f32, f32, f32, f32) {
    // FIXME: I don't really how to calculate this properly, seems to be clipping/cutoff no matter
    // how large the viewbox is (e.g. using a value of 10 for the shadow blur)
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

pub fn get_text_width(font: Font, font_scale: f32, text: &str) -> f32 {
    text.chars()
        .filter_map(|ch| {
            font.glyph_for_char(ch)
                .map(|glyph_id| {
                    let advance = font.advance(glyph_id).ok()?;
                    Some(advance.x() * font_scale)
                })
                .flatten()
        })
        .sum()
}
