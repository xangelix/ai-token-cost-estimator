//! Deterministic token coloring based on token ID.

use egui::Color32;

pub struct TokenColor {
    pub bg: Color32,
    pub bg_emphasis: Color32,
    pub border: Color32,
    pub text: Color32,
    pub label_bg: Color32,
}

pub fn color_for_token(token_id: u32, is_dark: bool) -> TokenColor {
    let hue = (token_id as f32 * 55.0) % 360.0;
    let h = hue / 360.0;
    let (r, g, b) = if is_dark {
        hsl_to_rgb(h, 0.60, 0.55)
    } else {
        hsl_to_rgb(h, 0.60, 0.60)
    };

    if is_dark {
        TokenColor {
            bg: Color32::from_rgba_unmultiplied(r, g, b, 76), // 0.30 alpha
            bg_emphasis: Color32::from_rgba_unmultiplied(r, g, b, 166), // 0.65 alpha
            border: Color32::from_rgba_unmultiplied(r, g, b, 115), // 0.45 alpha
            text: Color32::from_rgb(241, 245, 249),           // slate-100
            label_bg: Color32::from_rgb(r, g, b),
        }
    } else {
        TokenColor {
            bg: Color32::from_rgba_unmultiplied(r, g, b, 64), // 0.25 alpha
            bg_emphasis: Color32::from_rgba_unmultiplied(r, g, b, 153), // 0.60 alpha
            border: Color32::from_rgba_unmultiplied(r, g, b, 102), // 0.40 alpha
            text: Color32::from_rgb(9, 9, 11),                // slate-950
            label_bg: Color32::from_rgb(r, g, b),
        }
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
