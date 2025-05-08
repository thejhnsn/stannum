use syntect::highlighting::Color;

// Utility function to convert a syntect Color to a HEX string.
pub fn rgb_to_hex(color: Color) -> String {
    format!(
        "#{:02X}{:02X}{:02X}{:02X}",
        color.r, color.g, color.b, color.a
    )
}

pub fn rgb_to_yuv(color: Color) -> (f64, f64, f64) {
    let red = color.r as f64 / 255.0;
    let green = color.g as f64 / 255.0;
    let blue = color.b as f64 / 255.0;
    let y = 0.299 * red + 0.587 * green + 0.114 * blue;
    let u = -0.14713 * red - 0.28886 * green + 0.436 * blue;
    let v = 0.615 * red - 0.51499 * green - 0.10001 * blue;
    (y, u, v)
}

pub fn yuv_to_rgb(y: f64, u: f64, v: f64) -> Color {
    let red = ((y + 1.13983 * v) * 255.0) as u8;
    let green = ((y - 0.39465 * u - 0.5806 * v) * 255.0) as u8;
    let blue = ((y + 2.03211 * u) * 255.0) as u8;
    Color {
        r: red,
        g: green,
        b: blue,
        a: 255,
    }
}
