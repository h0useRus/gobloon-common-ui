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
        viewport: egui::ViewportBuilder::default().with_inner_size([760.0, 560.0]),
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
