//! Deterministic token coloring based on token ID.

use egui::Color32;

pub struct TokenColor {
    pub bg: Color32,
    pub bg_emphasis: Color32,
    pub border: Color32,
    pub text: Color32,
    pub label_bg: Color32,
}

pub fn color_for_token(token_id: u32) -> TokenColor {
    let hue = (token_id as f32 * 55.0) % 360.0;
    let h = hue / 360.0;
    let (r, g, b) = hsl_to_rgb(h, 0.9, 0.6);

    TokenColor {
        bg: Color32::from_rgba_unmultiplied(r, g, b, 51), // 0.2 alpha
        bg_emphasis: Color32::from_rgba_unmultiplied(r, g, b, 153), // 0.6 alpha
        border: Color32::from_rgba_unmultiplied(r, g, b, 89),
        text: Color32::from_rgb(15, 23, 42), // slate-900
        label_bg: Color32::from_rgb(r, g, b),
    }
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    if s == 0.0 {
        let v = (l * 255.0) as u8;
        return (v, v, v);
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l.mul_add(-s, l + s)
    };
    let p = 2.0f32.mul_add(l, -q);

    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

fn hue_to_rgb(p: f32, q: f32, t: f32) -> f32 {
    let mut t = t;
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return ((q - p) * 6.0).mul_add(t, p);
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return ((q - p) * (2.0 / 3.0 - t)).mul_add(6.0, p);
    }
    p
}
