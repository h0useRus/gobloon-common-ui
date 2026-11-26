//! Every widget on one screen, to check a change to the kit by eye: `cargo run -p gob-ui --example
//! gallery`. The theme buttons switch between light and dark.

use eframe::egui;
use egui::Color32;
use gob_ui::{DialogButton, EditableText, Palette, Palettes, Spin, icon};

const PALETTES: Palettes = Palettes {
    dark: Palette {
        accent: Color32::from_rgb(0x8c, 0xc8, 0x2c),
        panel: Color32::from_rgb(0x1b, 0x1d, 0x22),
        window: Color32::from_rgb(0x23, 0x26, 0x2c),
        faint: Color32::from_rgb(0x2a, 0x2d, 0x34),
    },
    light: Palette {
        accent: Color32::from_rgb(0x6f, 0x9e, 0x1e),
        panel: Color32::from_rgb(0xf4, 0xf5, 0xf7),
        window: Color32::from_rgb(0xff, 0xff, 0xff),
        faint: Color32::from_rgb(0xe8, 0xea, 0xee),
    },
};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([860.0, 820.0]),
        ..Default::default()
    };
    eframe::run_native(
        "gob-ui gallery",
        options,
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();
            gob_ui::add_icon_font(&mut fonts);
            cc.egui_ctx.set_fonts(fonts);
            gob_ui::apply(&cc.egui_ctx, &PALETTES, egui::ThemePreference::Dark);
            Ok(Box::new(Gallery::default()))
        }),
    )
}

struct Gallery {
    dark: bool,
    page: usize,
    tab: usize,
    on: bool,
    ticked: bool,
    toggled: bool,
    count: i32,
    level: f32,
    text: String,
    note: String,
    code: String,
    dialog: bool,
    window: bool,
    keys: Vec<gob_ui::ScanCode>,
    stick: Option<egui::TextureHandle>,
    /// Cards of the device picture dragged elsewhere: side and top.
    places: std::collections::HashMap<usize, (gob_ui::CalloutSide, f32)>,
    lit: Option<usize>,
}

impl Default for Gallery {
    fn default() -> Self {
        Self {
            dark: true,
            page: 0,
            tab: 0,
            on: true,
            ticked: true,
            toggled: false,
            count: 4,
            level: 0.5,
            text: "Text field".into(),
            note: "Editable text: click to edit".into(),
            code: "-- Lua with line numbers
local speed = script.float{ name = \"Speed\", default = 1 }
if speed.value > 0.5 then log(\"fast\") end
"
            .into(),
            dialog: false,
            window: false,
            keys: Vec::new(),
            stick: None,
            places: Default::default(),
            lit: None,
        }
    }
}

impl eframe::App for Gallery {
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = root.ctx().clone();
        egui::Panel::left("rail")
            .resizable(false)
            .exact_size(60.0)
            .show(root, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(gob_ui::SPACE_S);
                    for (i, glyph) in [icon::JOYSTICK, icon::KEYBOARD, icon::GEAR]
                        .into_iter()
                        .enumerate()
                    {
                        if gob_ui::nav_toggle(ui, self.page == i, glyph, "Page").clicked() {
                            self.page = i;
                        }
                    }
                });
            });
        egui::Panel::top("toolbar").show(root, |ui| {
            ui.horizontal(|ui| {
                gob_ui::toolbar_button(ui, true, icon::FILE_PLUS, "Toolbar button");
                gob_ui::toolbar_button(ui, false, icon::FLOPPY_DISK, "Disabled");
                if gob_ui::toolbar_toggle(ui, self.toggled, icon::PLAY, "Toolbar toggle").clicked()
                {
                    self.toggled = !self.toggled;
                }
                ui.separator();
                for (dark, glyph) in [(false, icon::SUN), (true, icon::MOON)] {
                    if gob_ui::icon_toggle(ui, self.dark == dark, glyph, "Theme").clicked() {
                        self.dark = dark;
                        let theme = if dark {
                            egui::ThemePreference::Dark
                        } else {
                            egui::ThemePreference::Light
                        };
                        gob_ui::apply(&ctx, &PALETTES, theme);
                    }
                }
            });
        });
        if self.page == 1 {
            egui::CentralPanel::default().show(root, |ui| self.picture(ui));
            return;
        }
        egui::CentralPanel::default().show(root, |ui| {
            ui.horizontal(|ui| {
                for (i, (glyph, caption)) in
                    [(icon::PALETTE, "Tab one"), (icon::SLIDERS, "Tab two")]
                        .into_iter()
                        .enumerate()
                {
                    if gob_ui::tab(ui, self.tab == i, glyph, caption).clicked() {
                        self.tab = i;
                    }
                }
            });
            ui.separator();
            ui.add(gob_ui::switch(&mut self.on, "Toggle switch"));
            gob_ui::tick(ui, &mut self.ticked, "Tick", PALETTES.dark.accent);
            ui.horizontal(|ui| {
                ui.label("Spin");
                ui.add(Spin::new(&mut self.count).range(0..=128));
            });
            gob_ui::slider(ui, &mut self.level, 0.0..=1.0, 0.01, "");
            ui.add(gob_ui::text_field(&mut self.text));
            ui.add(EditableText::new(&mut self.note, "note").hint("A note"));
            ui.add(
                gob_ui::CodeEditor::new(&mut self.code, "code")
                    .desired_rows(5)
                    .error_line(Some(2)),
            );
            ui.horizontal(|ui| {
                gob_ui::icon_button(ui, true, icon::TRASH, "Icon button");
                gob_ui::icon_button_colored(ui, true, icon::WARNING, gob_ui::RED, "Colored icon");
                gob_ui::icon_badge_button(
                    ui,
                    true,
                    icon::MICROPHONE,
                    icon::PLUS,
                    "Icon with a badge",
                );
                gob_ui::icon_badge_button(
                    ui,
                    false,
                    icon::MICROPHONE,
                    icon::ARROW_UP_RIGHT,
                    "Disabled icon with a badge",
                );
                gob_ui::record_button(ui, "Record");
                gob_ui::warn_mark(ui, icon::LINK, "Warning mark");
            });
            egui::ScrollArea::horizontal().show(ui, |ui| {
                let picked = gob_ui::keyboard_picker(
                    ui,
                    "keys",
                    &self.keys,
                    |_| None,
                    |k| {
                        format!(
                            "Scan code {}{:02X}",
                            if k.extended { "E0 " } else { "" },
                            k.code
                        )
                    },
                );
                if let Some(k) = picked {
                    match self.keys.iter().position(|&x| x == k) {
                        Some(i) => {
                            self.keys.remove(i);
                        }
                        None => self.keys.push(k),
                    }
                }
            });
            ui.horizontal(|ui| {
                if ui.button("Modal dialog").clicked() {
                    self.dialog = true;
                }
                if ui.button("Dialog window").clicked() {
                    self.window = true;
                }
            });
        });
        if self.dialog {
            gob_ui::modal(&ctx, "gallery-dialog").show(&ctx, |ui| {
                gob_ui::dialog_title(ui, Some((icon::QUESTION, None)), "Modal dialog");
                ui.label("Primary first, Cancel last.");
                let buttons = [
                    DialogButton::primary("OK"),
                    DialogButton::secondary("Cancel"),
                ];
                if gob_ui::dialog_buttons(ui, &buttons).is_some() {
                    self.dialog = false;
                }
            });
        }
        let window = egui::Window::new("dialog-window").default_size([320.0, 160.0]);
        gob_ui::dialog_window(&ctx, window, "Dialog window", &mut self.window, |ui| {
            ui.label("A compact title bar with a close button.");
        });
    }
}

impl Gallery {
    /// The device picture on a made-up stick drawn here (apps pass a photo).
    fn picture(&mut self, ui: &mut egui::Ui) {
        let texture = self
            .stick
            .get_or_insert_with(|| ui.ctx().load_texture("stick", stick(), Default::default()));
        let spots = [
            (60.0, 40.0, gob_ui::CalloutSide::Left, "Hat", None),
            (130.0, 50.0, gob_ui::CalloutSide::Right, "Red button", None),
            (75.0, 120.0, gob_ui::CalloutSide::Left, "Trigger", None),
            (
                60.0,
                250.0,
                gob_ui::CalloutSide::Left,
                "B1",
                Some("Base buttons"),
            ),
            (
                100.0,
                250.0,
                gob_ui::CalloutSide::Left,
                "B2",
                Some("Base buttons"),
            ),
            (140.0, 250.0, gob_ui::CalloutSide::Right, "Wheel", None),
        ];
        let callouts: Vec<gob_ui::Callout> = spots
            .iter()
            .enumerate()
            .map(|(i, &(x, y, side, title, group))| gob_ui::Callout {
                at: egui::pos2(x, y),
                side: self.places.get(&i).map_or(side, |p| p.0),
                title: title.into(),
                lines: if i % 2 == 0 {
                    vec!["Landing gear".into(), "Lights".into()]
                } else {
                    vec!["Drop an action".into()]
                },
                group: group.map(Into::into),
                lit: self.lit == Some(i),
                place: self.places.get(&i).map(|p| p.1),
            })
            .collect();
        let size = texture.size_vec2();
        let image = egui::Image::new(&*texture);
        let response = gob_ui::DevicePicture::new("gallery-picture", image, size, &callouts)
            .caption(Some("Picture credit line"))
            .movable_cards(true)
            .max_height(ui.available_height() - 40.0)
            .show(ui);
        if let Some((i, side, top)) = response.moved {
            self.places.insert(i, (side, top));
        }
        if let Some(i) = response.clicked() {
            self.lit = (self.lit != Some(i)).then_some(i);
        }
    }
}

/// A grey stick silhouette, 200 x 300.
fn stick() -> egui::ColorImage {
    let (w, h) = (200usize, 300usize);
    let mut pixels = vec![Color32::TRANSPARENT; w * h];
    for y in 0..h {
        for x in 0..w {
            let (fx, fy) = (x as f32, y as f32);
            let grip = (fx - 100.0).abs() < 28.0 + (fy - 60.0).max(0.0) * 0.05
                && (20.0..200.0).contains(&fy);
            let head = ((fx - 100.0).powi(2) / 60.0f32.powi(2)
                + (fy - 50.0).powi(2) / 35.0f32.powi(2))
                < 1.0;
            let base = (30.0..280.0).contains(&fy) && fy > 210.0 && (fx - 100.0).abs() < 90.0;
            if grip || head || base {
                let shade = 40 + ((fy / h as f32) * 30.0) as u8;
                pixels[y * w + x] = Color32::from_gray(shade);
            }
        }
    }
    egui::ColorImage::new([w, h], pixels)
}
