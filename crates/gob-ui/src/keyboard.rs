//! A clickable full-size keyboard (ANSI 104 keys, F13-F24, the media, browser, launch and
//! power keys, and the extra keys of ISO, Japanese, Korean and Brazilian layouts) for picking keys. Keys are set 1 scan codes with the E0 flag; every key is as
//! large as an [`icon_toggle`](crate::icon_toggle), wide keys keep the same grid.
//!
//! Legends are the usual key cap abbreviations and icons; the app can pass its own (the
//! current layout's characters) and the tooltips, which name the keys in its language.

use std::sync::{Arc, OnceLock};

use egui::{CornerRadius, Rect, Sense, Stroke, Vec2, vec2};

use crate::buttons::icon_toggle_size;
use crate::icon;
use crate::theme::accent;
use crate::tokens::{ICON_SIZE, RADIUS, SPACE_XS};

/// A key by its set 1 scan code; `extended` = sent with the E0 prefix.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ScanCode {
    pub code: u16,
    pub extended: bool,
}

const fn sc(code: u16) -> ScanCode {
    ScanCode {
        code,
        extended: false,
    }
}

const fn e0(code: u16) -> ScanCode {
    ScanCode {
        code,
        extended: true,
    }
}

/// How a key is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyKind {
    /// Letters, digits, punctuation, numpad.
    Main,
    /// Modifiers, editing and navigation keys: muted.
    Modifier,
    /// Keys most keyboards lack (F13-F24, media, browser, launch, power): tinted.
    Extra,
}

/// One key of the layout; position and size in key units (1 = one
/// [`icon_toggle_size`](crate::icon_toggle_size) plus the gap).
#[derive(Clone, Copy, Debug)]
pub struct KeyCap {
    pub key: ScanCode,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub kind: KeyKind,
    /// Built-in legend: text, or a Phosphor icon when `icon` is set.
    pub legend: &'static str,
    pub icon: bool,
}

/// Every key of the full-size layout.
pub fn full_layout() -> &'static [KeyCap] {
    static LAYOUT: OnceLock<Vec<KeyCap>> = OnceLock::new();
    LAYOUT.get_or_init(build_layout)
}

fn build_layout() -> Vec<KeyCap> {
    use KeyKind::{Extra, Main, Modifier};
    let mut caps = Vec::with_capacity(130);
    let mut put = |key, x, y, w, h, kind, legend: &'static str, icon| {
        caps.push(KeyCap {
            key,
            x,
            y,
            w,
            h,
            kind,
            legend,
            icon,
        })
    };

    // Media, browser, launch and power keys.
    let top: [(f32, ScanCode, &'static str); 21] = [
        (0.0, e0(0x20), icon::SPEAKER_X),
        (1.0, e0(0x2e), icon::SPEAKER_LOW),
        (2.0, e0(0x30), icon::SPEAKER_HIGH),
        (3.5, e0(0x10), icon::SKIP_BACK),
        (4.5, e0(0x22), icon::PLAY_PAUSE),
        (5.5, e0(0x24), icon::STOP),
        (6.5, e0(0x19), icon::SKIP_FORWARD),
        (8.0, e0(0x6a), icon::ARROW_LEFT),
        (9.0, e0(0x69), icon::ARROW_RIGHT),
        (10.0, e0(0x67), icon::ARROW_CLOCKWISE),
        (11.0, e0(0x68), icon::X_CIRCLE),
        (12.0, e0(0x32), icon::HOUSE),
        (13.0, e0(0x65), icon::MAGNIFYING_GLASS),
        (14.0, e0(0x66), icon::STAR),
        (15.5, e0(0x6c), icon::ENVELOPE),
        (16.5, e0(0x21), icon::CALCULATOR),
        (17.5, e0(0x6b), icon::DESKTOP),
        (18.5, e0(0x6d), icon::MUSIC_NOTES),
        (20.0, e0(0x5f), icon::MOON),
        (21.0, e0(0x63), icon::SUN),
        (22.0, e0(0x5e), icon::POWER),
    ];
    for (x, key, glyph) in top {
        put(key, x, 0.0, 1.0, 1.0, Extra, glyph, true);
    }

    // F13-F24 above F1-F12, in groups of four.
    const F13: [u16; 12] = [
        0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x6b, 0x6c, 0x6d, 0x6e, 0x76,
    ];
    const F1: [u16; 12] = [
        0x3b, 0x3c, 0x3d, 0x3e, 0x3f, 0x40, 0x41, 0x42, 0x43, 0x44, 0x57, 0x58,
    ];
    const F13_NAMES: [&str; 12] = [
        "F13", "F14", "F15", "F16", "F17", "F18", "F19", "F20", "F21", "F22", "F23", "F24",
    ];
    const F1_NAMES: [&str; 12] = [
        "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12",
    ];
    for n in 0..12 {
        let x = 2.0 + n as f32 + (n / 4) as f32 * 0.5;
        put(sc(F13[n]), x, 1.25, 1.0, 1.0, Extra, F13_NAMES[n], false);
        put(sc(F1[n]), x, 2.25, 1.0, 1.0, Modifier, F1_NAMES[n], false);
    }
    put(sc(0x01), 0.0, 2.25, 1.0, 1.0, Modifier, "Esc", false);
    put(e0(0x37), 15.5, 2.25, 1.0, 1.0, Modifier, "PrtSc", false);
    put(sc(0x46), 16.5, 2.25, 1.0, 1.0, Modifier, "ScrLk", false);
    put(sc(0x45), 17.5, 2.25, 1.0, 1.0, Modifier, "Pause", false);

    // Main block: rows of (scan code, width, kind, legend, icon).
    type Row = &'static [(ScanCode, f32, KeyKind, &'static str, bool)];
    const ROWS: [Row; 5] = [
        &[
            (sc(0x29), 1.0, Main, "`", false),
            (sc(0x02), 1.0, Main, "1", false),
            (sc(0x03), 1.0, Main, "2", false),
            (sc(0x04), 1.0, Main, "3", false),
            (sc(0x05), 1.0, Main, "4", false),
            (sc(0x06), 1.0, Main, "5", false),
            (sc(0x07), 1.0, Main, "6", false),
            (sc(0x08), 1.0, Main, "7", false),
            (sc(0x09), 1.0, Main, "8", false),
            (sc(0x0a), 1.0, Main, "9", false),
            (sc(0x0b), 1.0, Main, "0", false),
            (sc(0x0c), 1.0, Main, "-", false),
            (sc(0x0d), 1.0, Main, "=", false),
            (sc(0x0e), 2.0, Modifier, icon::BACKSPACE, true),
        ],
        &[
            (sc(0x0f), 1.5, Modifier, icon::ARROW_LINE_RIGHT, true),
            (sc(0x10), 1.0, Main, "Q", false),
            (sc(0x11), 1.0, Main, "W", false),
            (sc(0x12), 1.0, Main, "E", false),
            (sc(0x13), 1.0, Main, "R", false),
            (sc(0x14), 1.0, Main, "T", false),
            (sc(0x15), 1.0, Main, "Y", false),
            (sc(0x16), 1.0, Main, "U", false),
            (sc(0x17), 1.0, Main, "I", false),
            (sc(0x18), 1.0, Main, "O", false),
            (sc(0x19), 1.0, Main, "P", false),
            (sc(0x1a), 1.0, Main, "[", false),
            (sc(0x1b), 1.0, Main, "]", false),
            (sc(0x2b), 1.5, Main, "\\", false),
        ],
        &[
            (sc(0x3a), 1.75, Modifier, icon::ARROW_FAT_LINE_UP, true),
            (sc(0x1e), 1.0, Main, "A", false),
            (sc(0x1f), 1.0, Main, "S", false),
            (sc(0x20), 1.0, Main, "D", false),
            (sc(0x21), 1.0, Main, "F", false),
            (sc(0x22), 1.0, Main, "G", false),
            (sc(0x23), 1.0, Main, "H", false),
            (sc(0x24), 1.0, Main, "J", false),
            (sc(0x25), 1.0, Main, "K", false),
            (sc(0x26), 1.0, Main, "L", false),
            (sc(0x27), 1.0, Main, ";", false),
            (sc(0x28), 1.0, Main, "'", false),
            (sc(0x1c), 2.25, Modifier, icon::KEY_RETURN, true),
        ],
        &[
            (sc(0x2a), 2.25, Modifier, icon::ARROW_FAT_UP, true),
            (sc(0x2c), 1.0, Main, "Z", false),
            (sc(0x2d), 1.0, Main, "X", false),
            (sc(0x2e), 1.0, Main, "C", false),
            (sc(0x2f), 1.0, Main, "V", false),
            (sc(0x30), 1.0, Main, "B", false),
            (sc(0x31), 1.0, Main, "N", false),
            (sc(0x32), 1.0, Main, "M", false),
            (sc(0x33), 1.0, Main, ",", false),
            (sc(0x34), 1.0, Main, ".", false),
            (sc(0x35), 1.0, Main, "/", false),
            (sc(0x36), 2.75, Modifier, icon::ARROW_FAT_UP, true),
        ],
        &[
            (sc(0x1d), 1.25, Modifier, "Ctrl", false),
            (e0(0x5b), 1.25, Modifier, icon::WINDOWS_LOGO, true),
            (sc(0x38), 1.25, Modifier, "Alt", false),
            (sc(0x39), 6.25, Main, "", false),
            (e0(0x38), 1.25, Modifier, "AltGr", false),
            (e0(0x5c), 1.25, Modifier, icon::WINDOWS_LOGO, true),
            (e0(0x5d), 1.25, Modifier, icon::LIST, true),
            (e0(0x1d), 1.25, Modifier, "Ctrl", false),
        ],
    ];
    const MAIN_Y: f32 = 3.5;
    for (r, row) in ROWS.iter().enumerate() {
        let mut x = 0.0;
        for &(key, w, kind, legend, icon) in *row {
            put(key, x, MAIN_Y + r as f32, w, 1.0, kind, legend, icon);
            x += w;
        }
    }

    // Navigation block.
    let nav: [(f32, f32, ScanCode, &'static str, bool); 10] = [
        (15.5, 0.0, e0(0x52), "Ins", false),
        (16.5, 0.0, e0(0x47), "Home", false),
        (17.5, 0.0, e0(0x49), "PgUp", false),
        (15.5, 1.0, e0(0x53), "Del", false),
        (16.5, 1.0, e0(0x4f), "End", false),
        (17.5, 1.0, e0(0x51), "PgDn", false),
        (16.5, 3.0, e0(0x48), icon::ARROW_UP, true),
        (15.5, 4.0, e0(0x4b), icon::ARROW_LEFT, true),
        (16.5, 4.0, e0(0x50), icon::ARROW_DOWN, true),
        (17.5, 4.0, e0(0x4d), icon::ARROW_RIGHT, true),
    ];
    for (x, y, key, legend, icon) in nav {
        put(key, x, MAIN_Y + y, 1.0, 1.0, Modifier, legend, icon);
    }

    // Numpad.
    type Pad = (f32, f32, f32, f32, ScanCode, KeyKind, &'static str, bool);
    let pad: [Pad; 17] = [
        (0.0, 0.0, 1.0, 1.0, e0(0x45), Modifier, "Num", false),
        (1.0, 0.0, 1.0, 1.0, e0(0x35), Main, "/", false),
        (2.0, 0.0, 1.0, 1.0, sc(0x37), Main, "*", false),
        (3.0, 0.0, 1.0, 1.0, sc(0x4a), Main, "-", false),
        (0.0, 1.0, 1.0, 1.0, sc(0x47), Main, "7", false),
        (1.0, 1.0, 1.0, 1.0, sc(0x48), Main, "8", false),
        (2.0, 1.0, 1.0, 1.0, sc(0x49), Main, "9", false),
        (3.0, 1.0, 1.0, 2.0, sc(0x4e), Main, "+", false),
        (0.0, 2.0, 1.0, 1.0, sc(0x4b), Main, "4", false),
        (1.0, 2.0, 1.0, 1.0, sc(0x4c), Main, "5", false),
        (2.0, 2.0, 1.0, 1.0, sc(0x4d), Main, "6", false),
        (0.0, 3.0, 1.0, 1.0, sc(0x4f), Main, "1", false),
        (1.0, 3.0, 1.0, 1.0, sc(0x50), Main, "2", false),
        (2.0, 3.0, 1.0, 1.0, sc(0x51), Main, "3", false),
        (
            3.0,
            3.0,
            1.0,
            2.0,
            e0(0x1c),
            Modifier,
            icon::KEY_RETURN,
            true,
        ),
        (0.0, 4.0, 2.0, 1.0, sc(0x52), Main, "0", false),
        (2.0, 4.0, 1.0, 1.0, sc(0x53), Main, ".", false),
    ];
    for (x, y, w, h, key, kind, legend, icon) in pad {
        put(key, 19.0 + x, MAIN_Y + y, w, h, kind, legend, icon);
    }

    // Keys of other layouts below the main block: ISO's 102nd key (between Left Shift and
    // Z there), Japanese (Ro, Yen, Muhenkan, Henkan, Kana), Korean (Hanja, Han/Yeong), and
    // the numpad's = and , (Mac, Brazilian ABNT).
    const INTL_Y: f32 = MAIN_Y + 5.25;
    // Wide enough for their names, from the left edge.
    let intl: [(f32, ScanCode, &'static str); 8] = [
        (1.0, sc(0x56), "<>"),
        (1.0, sc(0x73), "Ro"),
        (1.0, sc(0x7d), "¥"),
        (2.0, sc(0x7b), "Muhenkan"),
        (1.5, sc(0x79), "Henkan"),
        (1.25, sc(0x70), "Kana"),
        (1.5, sc(0x71), "Hanja"),
        (2.0, sc(0x72), "Han/Yeong"),
    ];
    let mut x = 0.0;
    for (w, key, legend) in intl {
        put(key, x, INTL_Y, w, 1.0, Extra, legend, false);
        x += w;
    }
    put(sc(0x59), 19.0, INTL_Y, 1.0, 1.0, Extra, "=", false);
    put(sc(0x7e), 20.0, INTL_Y, 1.0, 1.0, Extra, ",", false);
    caps
}

/// Legends the app passed, read when the picker appears (not every frame) and kept while
/// it stays shown.
#[derive(Clone)]
struct Legends {
    frame: u64,
    texts: Arc<Vec<Option<String>>>,
}

/// The full keyboard; keys in `selected` are drawn like a selected toggle. `id_salt` tells
/// two pickers in one `Ui` apart. `legend` may replace a key's built-in legend (e.g. the
/// current layout's character); it is asked once each time the picker appears, not every
/// frame. `tooltip` names a key, asked only while it is hovered or for accessibility.
/// Returns the key clicked this frame.
pub fn keyboard_picker(
    ui: &mut egui::Ui,
    id_salt: impl egui::AsIdSalt,
    selected: &[ScanCode],
    legend: impl Fn(ScanCode) -> Option<String>,
    tooltip: impl Fn(ScanCode) -> String,
) -> Option<ScanCode> {
    let id = ui.make_persistent_id(id_salt);
    let gap = SPACE_XS;
    let pitch = icon_toggle_size(ui) + Vec2::splat(gap);
    let caps = full_layout();
    let size = caps.iter().fold(Vec2::ZERO, |s, c| {
        s.max(vec2(
            (c.x + c.w) * pitch.x - gap,
            (c.y + c.h) * pitch.y - gap,
        ))
    });
    let (area, _) = ui.allocate_exact_size(size, Sense::hover());

    // Legends are asked again when the picker was not shown last frame: the keyboard layout
    // may have changed meanwhile.
    let frame = ui.ctx().cumulative_frame_nr();
    let texts = match ui.data(|d| d.get_temp::<Legends>(id)) {
        Some(l) if l.frame + 1 >= frame => l.texts,
        _ => Arc::new(caps.iter().map(|c| legend(c.key)).collect()),
    };
    let kept = Legends {
        frame,
        texts: texts.clone(),
    };
    ui.data_mut(|d| d.insert_temp(id, kept));

    let visuals = ui.visuals().clone();
    let accent = accent(ui);
    let ink = visuals.text_color();
    let body = egui::TextStyle::Body.resolve(ui.style());
    let text_font = egui::FontId::new(body.size * 0.9, body.family);
    let icon_font = egui::FontId::proportional(ICON_SIZE);
    let mut clicked = None;
    for (cap, own) in caps.iter().zip(texts.iter()) {
        let min = area.min + vec2(cap.x * pitch.x, cap.y * pitch.y);
        let rect = Rect::from_min_size(min, vec2(cap.w * pitch.x - gap, cap.h * pitch.y - gap));
        let response = ui.interact(rect, id.with(cap.key), Sense::click());
        let on = selected.contains(&cap.key);
        let hovered = response.hovered();
        // Fills mixed from the text color: they show in light and dark alike, and inside
        // menus too (egui's menus make button fills transparent).
        let (fill, text) = match (on, hovered) {
            (true, false) => (visuals.selection.bg_fill, accent),
            (true, true) => (accent.gamma_multiply(0.5), accent),
            (false, true) => (ink.gamma_multiply(0.2), visuals.strong_text_color()),
            (false, false) => match cap.kind {
                KeyKind::Main => (ink.gamma_multiply(0.11), ink),
                KeyKind::Modifier => (ink.gamma_multiply(0.05), visuals.weak_text_color()),
                KeyKind::Extra => (accent.gamma_multiply(0.14), ink),
            },
        };
        if ui.is_rect_visible(rect) {
            let stroke = if response.has_focus() {
                Stroke::new(2.0, visuals.strong_text_color())
            } else if on {
                Stroke::new(1.0, accent)
            } else {
                Stroke::NONE
            };
            let painter = ui.painter().with_clip_rect(rect);
            painter.rect(
                rect,
                CornerRadius::same(RADIUS),
                fill,
                stroke,
                egui::StrokeKind::Inside,
            );
            let (label, font) = match own {
                Some(s) => (s.as_str(), &text_font),
                None if cap.icon => (cap.legend, &icon_font),
                None => (cap.legend, &text_font),
            };
            if !label.is_empty() {
                let galley = fitted(&painter, label, font, text, rect.width() - 4.0);
                painter.galley(rect.center() - galley.size() / 2.0, galley, text);
            }
        }
        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::Role::Button, true, on, tooltip(cap.key))
        });
        let response = if hovered {
            response.on_hover_text(tooltip(cap.key))
        } else {
            response
        };
        if response.clicked() {
            clicked = Some(cap.key);
        }
    }
    clicked
}

/// `text` on one line, made smaller (down to 60 %) when it is wider than `width`.
fn fitted(
    painter: &egui::Painter,
    text: &str,
    font: &egui::FontId,
    color: egui::Color32,
    width: f32,
) -> Arc<egui::Galley> {
    let galley = painter.layout_no_wrap(text.to_owned(), font.clone(), color);
    if galley.size().x <= width {
        return galley;
    }
    let size = (font.size * width / galley.size().x).max(font.size * 0.6);
    let smaller = egui::FontId::new(size, font.family.clone());
    painter.layout_no_wrap(text.to_owned(), smaller, color)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_unique_and_do_not_overlap() {
        let caps = full_layout();
        for (i, a) in caps.iter().enumerate() {
            for b in &caps[i + 1..] {
                assert_ne!(
                    a.key, b.key,
                    "{} and {} share a scan code",
                    a.legend, b.legend
                );
                let apart =
                    a.x + a.w <= b.x || b.x + b.w <= a.x || a.y + a.h <= b.y || b.y + b.h <= a.y;
                assert!(apart, "{:?} overlaps {:?}", a.key, b.key);
            }
        }
    }

    #[test]
    fn main_rows_are_fifteen_keys_wide() {
        let caps = full_layout();
        for r in 0..5 {
            let y = 3.5 + r as f32;
            let right = caps
                .iter()
                .filter(|c| c.y == y && c.x < 15.0)
                .map(|c| c.x + c.w)
                .fold(0.0, f32::max);
            assert_eq!(right, 15.0, "row {r}");
        }
    }
}
