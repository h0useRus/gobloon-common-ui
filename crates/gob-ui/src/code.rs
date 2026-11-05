//! Code editor: monospace text with line numbers, Lua highlighting (keywords, strings, comments,
//! numbers, the script API's globals) and a marked error line. No wrapping: long lines scroll.

use egui::text::{LayoutJob, TextFormat};
use egui::{Color32, FontId, Ui};

use crate::theme::accent;
use crate::tokens::RED;

const KEYWORDS: &[&str] = &[
    "and", "break", "continue", "do", "else", "elseif", "end", "false", "for", "function", "if",
    "in", "local", "nil", "not", "or", "repeat", "return", "then", "true", "until", "while",
];

/// Lua's standard globals and Joystick Gobloon's script API.
const GLOBALS: &[&str] = &[
    "script",
    "vjoy",
    "gamepad",
    "keyboard",
    "mouse",
    "mode",
    "joystick",
    "speak",
    "play_sound",
    "run_action",
    "log",
    "print",
    "math",
    "string",
    "table",
    "os",
    "pairs",
    "ipairs",
    "tostring",
    "tonumber",
    "type",
    "error",
    "pcall",
    "select",
    "self",
];

#[derive(Debug, Clone, Copy, PartialEq)]
enum Kind {
    Plain,
    Keyword,
    Global,
    String,
    Comment,
    Number,
}

struct Colors {
    plain: Color32,
    keyword: Color32,
    global: Color32,
    string: Color32,
    comment: Color32,
    number: Color32,
}

impl Colors {
    fn of(ui: &Ui) -> Self {
        let v = ui.visuals();
        let dark = v.dark_mode;
        let pick = |d: (u8, u8, u8), l: (u8, u8, u8)| {
            let (r, g, b) = if dark { d } else { l };
            Color32::from_rgb(r, g, b)
        };
        Self {
            plain: v.text_color(),
            keyword: accent(ui),
            global: pick((0x4f, 0xc1, 0xff), (0x00, 0x70, 0xc1)),
            string: pick((0xce, 0x91, 0x78), (0xa3, 0x15, 0x15)),
            comment: v.weak_text_color(),
            number: pick((0xb5, 0xce, 0xa8), (0x09, 0x86, 0x58)),
        }
    }

    fn get(&self, kind: Kind) -> Color32 {
        match kind {
            Kind::Plain => self.plain,
            Kind::Keyword => self.keyword,
            Kind::Global => self.global,
            Kind::String => self.string,
            Kind::Comment => self.comment,
            Kind::Number => self.number,
        }
    }
}

/// Lua source cut into highlighted pieces: (byte range, kind).
fn tokens(code: &str) -> Vec<(std::ops::Range<usize>, Kind)> {
    let b = code.as_bytes();
    let mut out: Vec<(std::ops::Range<usize>, Kind)> = Vec::new();
    let mut i = 0;
    // `[[`, `[=[`... of a long bracket at `i`: its level.
    let long_open = |i: usize| -> Option<usize> {
        if b.get(i) != Some(&b'[') {
            return None;
        }
        let mut j = i + 1;
        while b.get(j) == Some(&b'=') {
            j += 1;
        }
        (b.get(j) == Some(&b'[')).then_some(j - i - 1)
    };
    let long_end = |from: usize, level: usize| -> usize {
        let close = format!("]{}]", "=".repeat(level));
        code[from..]
            .find(&close)
            .map_or(code.len(), |p| from + p + close.len())
    };
    while i < b.len() {
        let c = b[i];
        let start = i;
        let kind = if c == b'-' && b.get(i + 1) == Some(&b'-') {
            i += 2;
            match long_open(i) {
                Some(level) => i = long_end(i + level + 2, level),
                None => {
                    while i < b.len() && b[i] != b'\n' {
                        i += 1;
                    }
                }
            }
            Kind::Comment
        } else if let Some(level) = long_open(i) {
            i = long_end(i + level + 2, level);
            Kind::String
        } else if c == b'"' || c == b'\'' {
            i += 1;
            while i < b.len() && b[i] != c && b[i] != b'\n' {
                i += if b[i] == b'\\' { 2 } else { 1 };
            }
            i = (i + 1).min(b.len());
            Kind::String
        } else if c.is_ascii_digit() || c == b'.' && b.get(i + 1).is_some_and(u8::is_ascii_digit) {
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'.' || b[i] == b'_') {
                i += 1;
            }
            Kind::Number
        } else if c.is_ascii_alphabetic() || c == b'_' {
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1;
            }
            let word = &code[start..i];
            // A field (`x.mode`) is not the global.
            let field = start > 0 && b[start - 1] == b'.' && !code[..start].ends_with("..");
            if KEYWORDS.contains(&word) {
                Kind::Keyword
            } else if GLOBALS.contains(&word) && !field {
                Kind::Global
            } else {
                Kind::Plain
            }
        } else {
            // One character (any UTF-8 length).
            i += code[i..].chars().next().map_or(1, char::len_utf8);
            Kind::Plain
        };
        let i_end = i.min(b.len());
        match out.last_mut() {
            Some((range, k)) if *k == kind && range.end == start => range.end = i_end,
            _ => out.push((start..i_end, kind)),
        }
        i = i_end;
    }
    out
}

/// Highlighted Lua as a layout job.
pub fn lua_layout_job(ui: &Ui, code: &str, font: FontId) -> LayoutJob {
    let colors = Colors::of(ui);
    let mut job = LayoutJob::default();
    for (range, kind) in tokens(code) {
        job.append(
            &code[range],
            0.0,
            TextFormat::simple(font.clone(), colors.get(kind)),
        );
    }
    job
}

/// A Lua code editor with line numbers; `error_line` (1-based) is marked red.
pub struct CodeEditor<'a> {
    code: &'a mut String,
    id: egui::Id,
    error_line: Option<u32>,
    rows: usize,
}

impl<'a> CodeEditor<'a> {
    pub fn new(code: &'a mut String, id_salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        Self {
            code,
            id: egui::Id::unique(("code-editor", id_salt)),
            error_line: None,
            rows: 16,
        }
    }

    pub fn error_line(mut self, line: Option<u32>) -> Self {
        self.error_line = line;
        self
    }

    /// Lines shown at least.
    pub fn desired_rows(mut self, rows: usize) -> Self {
        self.rows = rows;
        self
    }
}

impl egui::Widget for CodeEditor<'_> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let font = egui::TextStyle::Monospace.resolve(ui.style());
        let lines = (self.code.matches('\n').count() + 1).max(self.rows);
        let weak = ui.visuals().weak_text_color();
        let mut gutter = LayoutJob::default();
        let width = lines.to_string().len();
        for n in 1..=lines {
            let color = if self.error_line == Some(n as u32) {
                RED
            } else {
                weak
            };
            let sep = if n < lines { "\n" } else { "" };
            gutter.append(
                &format!("{n:>width$}{sep}"),
                0.0,
                TextFormat::simple(font.clone(), color),
            );
        }
        let mut layouter = |ui: &Ui, buf: &dyn egui::TextBuffer, _wrap: f32| {
            let mut job = lua_layout_job(ui, buf.as_str(), font.clone());
            job.wrap.max_width = f32::INFINITY;
            ui.fonts_mut(|f| f.layout_job(job))
        };
        egui::ScrollArea::horizontal()
            .id_salt(self.id.with("scroll"))
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    // Lined up with the editor's text (its margin).
                    ui.add_space(2.0);
                    ui.vertical(|ui| {
                        ui.add_space(4.0);
                        ui.label(gutter);
                    });
                    ui.add(
                        egui::TextEdit::multiline(self.code)
                            .id(self.id)
                            .font(font.clone())
                            .code_editor()
                            .lock_focus(true)
                            .desired_rows(self.rows)
                            .desired_width(f32::INFINITY)
                            .layouter(&mut layouter),
                    )
                })
                .inner
            })
            .inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(code: &str) -> Vec<(&str, Kind)> {
        tokens(code)
            .into_iter()
            .filter(|(_, k)| *k != Kind::Plain)
            .map(|(r, k)| (&code[r], k))
            .collect()
    }

    #[test]
    fn lua_tokens() {
        let code = "local x = \"a\\\"b\" -- note\nif x.mode then vjoy.axis(1, 0.5e1) end --[[ long\n]] s = [==[ x ]==]";
        assert_eq!(
            kinds(code),
            [
                ("local", Kind::Keyword),
                ("\"a\\\"b\"", Kind::String),
                ("-- note", Kind::Comment),
                ("if", Kind::Keyword),
                ("then", Kind::Keyword),
                ("vjoy", Kind::Global),
                ("1", Kind::Number),
                ("0.5e1", Kind::Number),
                ("end", Kind::Keyword),
                ("--[[ long\n]]", Kind::Comment),
                ("[==[ x ]==]", Kind::String),
            ]
        );
        // Every byte is covered, also unterminated strings and non-ASCII text.
        let odd = "x = 'é unterminated\n--[[ open";
        let covered: usize = tokens(odd).iter().map(|(r, _)| r.len()).sum();
        assert_eq!(covered, odd.len());
    }
}
