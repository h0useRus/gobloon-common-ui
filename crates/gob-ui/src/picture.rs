//! A picture of a device with callouts: the picture in the middle and a column of callouts on
//! each side, close to it and centered together with it. A callout is a title, one line per
//! part of the control and a glyph between them and the picture (the text runs toward the
//! glyph, so the two read as one): a compass for a hat, a ring for stick axes, a gauge for a single axis
//! and small badges for the rest (one per part). A leader runs from the glyph to the control's
//! spot on the picture. Axis glyphs show where the axes are now when the app passes the values.
//!
//! The app brings the picture (any [`egui::Image`]: a photo or a rasterized drawing), its size in
//! the units the spots use (pixels of a photo, the viewBox of a drawing) and the texts. Every
//! callout can be hovered, clicked and dropped on (`Response::dnd_release_payload`); each of its
//! lines, badges and compass arrows is a target of its own ([`PictureResponse::line`]).

use std::sync::Arc;

use egui::text::{LayoutJob, TextWrapping};
use egui::{
    Align2, Color32, CornerRadius, FontId, Galley, Pos2, Rect, Sense, Stroke, TextFormat, Vec2,
};

use crate::theme::accent;
use crate::tokens::{SPACE_M, SPACE_S, SPACE_XS};

/// Which column a callout goes to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CalloutSide {
    #[default]
    Left,
    Right,
}

/// How a callout's parts are drawn beside its text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CalloutGlyph {
    /// One badge per line, showing the line's `badge` (a dot when empty).
    #[default]
    Badges,
    /// A hat: the lines are up, right, down, left and optionally push, drawn as a compass.
    Hat,
    /// Stick axes: a ring with the stick's position (`axes` = x, y and optionally twist).
    Stick,
    /// One axis (a wheel, a slider): a bar with its position (`axes` = the value).
    Gauge,
}

/// One line of a callout: one part of the control (a hat direction, a trigger stage) and what
/// it does.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CalloutLine {
    /// Short name of the part before the text ("1st", "X", "F2"); may be empty.
    pub label: String,
    /// What the part's badge shows (badge glyphs only): a letter, a number or an icon.
    pub badge: String,
    pub text: String,
    /// The part is held now.
    pub lit: bool,
    /// Nothing on it yet: drawn faint and slanted.
    pub empty: bool,
    /// The part's own spot (callouts gathering several controls): its badge gets a leader.
    pub at: Option<Pos2>,
}

impl From<String> for CalloutLine {
    fn from(text: String) -> Self {
        Self {
            text,
            ..Default::default()
        }
    }
}

impl From<&str> for CalloutLine {
    fn from(text: &str) -> Self {
        text.to_string().into()
    }
}

/// One callout and its spot.
#[derive(Debug, Clone, Default)]
pub struct Callout {
    /// The spot, in picture units.
    pub at: Pos2,
    pub side: CalloutSide,
    pub title: String,
    pub lines: Vec<CalloutLine>,
    pub glyph: CalloutGlyph,
    /// Where the axes of a stick or gauge glyph are now, -1 to 1 (empty = not known).
    pub axes: Vec<f32>,
    /// Drawn in the accent color (the control is held now, or selected).
    pub lit: bool,
}

/// What the picture did this frame.
pub struct PictureResponse {
    pub response: egui::Response,
    /// One per callout, in the order given: its text, glyph and spot together.
    pub callouts: Vec<egui::Response>,
    /// The callout under the pointer.
    pub hovered: Option<usize>,
    /// The line under the pointer: (callout, line), also over its badge or compass arrow.
    pub line: Option<(usize, usize)>,
    /// Where the picture is drawn and how picture units map onto the screen.
    pub picture: Rect,
    pub scale: f32,
}

impl PictureResponse {
    /// The first callout clicked this frame.
    pub fn clicked(&self) -> Option<usize> {
        self.callouts.iter().position(egui::Response::clicked)
    }

    /// A screen position as picture units.
    pub fn to_picture(&self, pos: Pos2) -> Pos2 {
        Pos2::ZERO + (pos - self.picture.min) / self.scale
    }
}

/// The widget. Fills the available width; the height follows the picture.
pub struct DevicePicture<'a> {
    id: egui::Id,
    image: egui::Image<'a>,
    size: Vec2,
    callouts: &'a [Callout],
    caption: Option<&'a str>,
    max_height: f32,
    draggable: bool,
}

const COLUMN_MIN: f32 = 200.0;
const COLUMN_MAX: f32 = 330.0;
const SPOT_RADIUS: f32 = 4.5;
const HAT: f32 = 50.0;
const STICK: f32 = 66.0;
const BADGE: f32 = 22.0;
const BADGE_GAP: f32 = 5.0;
const GAUGE: Vec2 = Vec2::new(16.0, 46.0);

fn glyph_size(c: &Callout) -> Vec2 {
    match c.glyph {
        CalloutGlyph::Hat => Vec2::splat(HAT),
        CalloutGlyph::Stick => Vec2::splat(STICK),
        CalloutGlyph::Gauge => GAUGE,
        CalloutGlyph::Badges => {
            let n = c.lines.len().max(1) as f32;
            Vec2::new(n * BADGE + (n - 1.0) * BADGE_GAP, BADGE)
        }
    }
}

impl<'a> DevicePicture<'a> {
    /// `size` = the picture's size in the units of the callouts' spots.
    pub fn new(
        id_salt: impl std::hash::Hash + std::fmt::Debug,
        image: egui::Image<'a>,
        size: Vec2,
        callouts: &'a [Callout],
    ) -> Self {
        Self {
            id: egui::Id::unique(("device-picture", id_salt)),
            image,
            size,
            callouts,
            caption: None,
            max_height: f32::INFINITY,
            draggable: false,
        }
    }

    /// A small line under the picture (where the picture comes from).
    pub fn caption(mut self, caption: Option<&'a str>) -> Self {
        self.caption = caption;
        self
    }

    /// The picture is never taller than this (the callouts may still reach further).
    pub fn max_height(mut self, height: f32) -> Self {
        self.max_height = height;
        self
    }

    /// The spots can be dragged (`callouts[i].dragged()`, the pointer through
    /// [`PictureResponse::to_picture`]): for placing them.
    pub fn draggable(mut self, draggable: bool) -> Self {
        self.draggable = draggable;
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> PictureResponse {
        let width = ui.available_width();
        let column = (width * 0.3).clamp(COLUMN_MIN, COLUMN_MAX);
        let middle = (width - 2.0 * (column + SPACE_M)).max(80.0);
        let scale = (middle / self.size.x)
            .min(self.max_height / self.size.y)
            .max(0.01);
        let shown = self.size * scale;
        // The columns hug the picture; picture and columns are centered together.
        let total = shown.x + 2.0 * (column + SPACE_M);

        let small = FontId::proportional(ui.text_style_height(&egui::TextStyle::Small) + 1.5);
        let title_font = FontId::proportional(small.size + 1.0);
        let line_h = ui.fonts_mut(|f| f.row_height(&small)) + 1.0;
        let title_h = ui.fonts_mut(|f| f.row_height(&title_font)) + 3.0;
        let text_h = |c: &Callout| title_h + c.lines.len() as f32 * line_h;
        let item_h = |c: &Callout| text_h(c).max(glyph_size(c).y);

        // Callouts stacked per side in the order of their spots, each as near its spot's
        // height as the ones above allow.
        let mut tops = vec![0.0; self.callouts.len()];
        let mut bottom = shown.y;
        for side in [CalloutSide::Left, CalloutSide::Right] {
            let mut order: Vec<usize> = (0..self.callouts.len())
                .filter(|&i| self.callouts[i].side == side)
                .collect();
            order.sort_by(|&a, &b| self.callouts[a].at.y.total_cmp(&self.callouts[b].at.y));
            let mut next = 0.0f32;
            for &i in &order {
                let c = &self.callouts[i];
                let want = c.at.y * scale - item_h(c) / 2.0;
                tops[i] = want.max(next);
                next = tops[i] + item_h(c) + SPACE_M;
            }
            bottom = bottom.max(next);
        }
        let caption_h = if self.caption.is_some() {
            line_h + SPACE_S
        } else {
            0.0
        };
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(width, bottom.max(shown.y + caption_h)),
            Sense::hover(),
        );
        let picture = Rect::from_min_size(
            Pos2::new(rect.center().x - shown.x / 2.0, rect.top()),
            shown,
        );
        let (left, right) = if total < width {
            (rect.center().x - total / 2.0, rect.center().x + total / 2.0)
        } else {
            (rect.left(), rect.right())
        };
        self.image.paint_at(ui, picture);
        let painter = ui.painter_at(rect.expand(SPOT_RADIUS * 2.0));
        let visuals = ui.visuals().clone();
        if let Some(caption) = self.caption {
            painter.text(
                Pos2::new(picture.center().x, picture.bottom() + SPACE_XS),
                Align2::CENTER_TOP,
                caption,
                small.clone(),
                visuals.weak_text_color(),
            );
        }

        let pal = Pal {
            accent: accent(ui),
            text: visuals.strong_text_color(),
            body: visuals.text_color(),
            weak: visuals.weak_text_color(),
            fill: visuals.extreme_bg_color.gamma_multiply(0.85),
            line: visuals
                .widgets
                .noninteractive
                .fg_stroke
                .color
                .gamma_multiply(0.5),
        };
        let pointer = ui.ctx().pointer_hover_pos();
        let dragging = egui::DragAndDrop::has_any_payload(ui.ctx());
        let mut line_under = None;
        let mut callouts = Vec::with_capacity(self.callouts.len());
        let mut hovered = None;

        for (i, c) in self.callouts.iter().enumerate() {
            let glyph = glyph_size(c);
            let h = item_h(c);
            let top = rect.top() + tops[i];
            let (outer, inner) = match c.side {
                CalloutSide::Left => (left, left + column),
                CalloutSide::Right => (right, right - column),
            };
            let item = Rect::from_x_y_ranges(outer.min(inner)..=outer.max(inner), top..=top + h);
            // Glyphs up to a stick's size share one upright axis per column; wider rows of
            // badges reach out from the inner edge.
            let lane = glyph.x.max(STICK);
            let center_x = match c.side {
                CalloutSide::Left => inner - lane / 2.0,
                CalloutSide::Right => inner + lane / 2.0,
            };
            let glyph_rect = Rect::from_center_size(Pos2::new(center_x, top + h / 2.0), glyph);
            let text_width = column - glyph.x.max(STICK) - SPACE_S;
            let spot = picture.min + c.at.to_vec2() * scale;
            let spot_rect = Rect::from_center_size(spot, Vec2::splat(SPOT_RADIUS * 4.0));

            let id = self.id.with(i);
            let item_response = ui.interact(item, id, Sense::click());
            let spot_sense = if self.draggable {
                Sense::click_and_drag()
            } else {
                Sense::click()
            };
            let spot_response = ui.interact(spot_rect, id.with("spot"), spot_sense);
            let both = item_response.union(spot_response);
            let hot = both.hovered()
                || pointer.is_some_and(|p| item.contains(p) || spot_rect.contains(p));
            if hot {
                hovered = Some(i);
                painter.rect_filled(
                    item.expand(SPACE_XS),
                    CornerRadius::same(6),
                    pal.accent
                        .gamma_multiply(if dragging { 0.14 } else { 0.06 }),
                );
            }
            let strong = c.lit || hot;

            // The leader from the glyph to the spot, unless the badges have spots of their own.
            let per_line = c.lines.iter().any(|l| l.at.is_some());
            if !per_line {
                let from = match c.side {
                    CalloutSide::Left => glyph_rect.right_center(),
                    CalloutSide::Right => glyph_rect.left_center(),
                };
                let elbow_x = match c.side {
                    CalloutSide::Left => (from.x + SPACE_M).min(picture.left()).max(from.x),
                    CalloutSide::Right => (from.x - SPACE_M).max(picture.right()).min(from.x),
                };
                painter.line(
                    vec![from, Pos2::new(elbow_x, from.y), spot],
                    Stroke::new(
                        if strong { 1.6 } else { 1.0 },
                        if strong { pal.accent } else { pal.line },
                    ),
                );
                draw_spot(&painter, spot, c.lit, strong, &pal);
            }

            // The part under the pointer: a badge or a compass arrow.
            let mut part_under = None;
            match c.glyph {
                CalloutGlyph::Hat => {
                    if let Some(p) = pointer.filter(|p| glyph_rect.contains(*p)) {
                        part_under = Some(compass_at(
                            p - glyph_rect.center(),
                            glyph.x,
                            c.lines.len() > 4,
                        ));
                    }
                    draw_hat(&painter, glyph_rect, &c.lines, part_under, dragging, &pal);
                }
                CalloutGlyph::Stick => draw_stick(&painter, glyph_rect, c, strong, &pal),
                CalloutGlyph::Gauge => draw_gauge(&painter, glyph_rect, c, strong, &pal),
                CalloutGlyph::Badges => {
                    for (n, l) in c.lines.iter().enumerate() {
                        let badge = Rect::from_min_size(
                            glyph_rect.min + Vec2::new(n as f32 * (BADGE + BADGE_GAP), 0.0),
                            Vec2::splat(BADGE),
                        );
                        let under = pointer.is_some_and(|p| badge.contains(p));
                        if under {
                            part_under = Some(n);
                        }
                        if let Some(at) = l.at {
                            let to = picture.min + at.to_vec2() * scale;
                            let start =
                                badge.center() + (to - badge.center()).normalized() * BADGE * 0.5;
                            let lit = l.lit || (under && dragging) || hot;
                            painter.line_segment(
                                [start, to],
                                Stroke::new(
                                    if lit { 1.6 } else { 1.0 },
                                    if lit { pal.accent } else { pal.line },
                                ),
                            );
                            draw_spot(&painter, to, l.lit, lit, &pal);
                        }
                        draw_badge(&painter, badge, l, under && dragging, &pal, &small);
                    }
                }
            }
            if let Some(n) = part_under {
                line_under = Some((i, n));
            }

            // Title and lines beside the glyph, running toward it.
            let (text_x, flush_right) = match c.side {
                CalloutSide::Left => (center_x - lane / 2.0 - SPACE_S, true),
                CalloutSide::Right => (center_x + lane / 2.0 + SPACE_S, false),
            };
            let mut y = top + (h - text_h(c)) / 2.0;
            let title = elided(
                ui,
                None,
                &c.title.to_uppercase(),
                title_font.clone(),
                if strong { pal.accent } else { pal.text },
                false,
                text_width,
            );
            paint_row(&painter, &title, text_x, y, flush_right);
            y += title_h;
            for (n, l) in c.lines.iter().enumerate() {
                let row = Rect::from_x_y_ranges(item.x_range(), y..=y + line_h);
                if part_under.is_none() && pointer.is_some_and(|p| row.contains(p)) {
                    line_under = Some((i, n));
                }
                if dragging && line_under == Some((i, n)) {
                    painter.rect_filled(
                        row,
                        CornerRadius::same(3),
                        pal.accent.gamma_multiply(0.22),
                    );
                }
                let color = if l.lit {
                    pal.accent
                } else if l.empty {
                    pal.weak
                } else {
                    pal.body
                };
                let label = (!l.label.is_empty()).then_some((l.label.as_str(), pal.weak));
                let galley = elided(
                    ui,
                    label,
                    &l.text,
                    small.clone(),
                    color,
                    l.empty,
                    text_width,
                );
                paint_row(&painter, &galley, text_x, y, flush_right);
                y += line_h;
            }
            callouts.push(both);
        }
        PictureResponse {
            response,
            callouts,
            hovered,
            line: line_under,
            picture,
            scale,
        }
    }
}

struct Pal {
    accent: Color32,
    text: Color32,
    body: Color32,
    weak: Color32,
    fill: Color32,
    line: Color32,
}

fn draw_spot(painter: &egui::Painter, at: Pos2, lit: bool, strong: bool, pal: &Pal) {
    let r = if lit { SPOT_RADIUS + 1.5 } else { SPOT_RADIUS };
    painter.circle_filled(at, r + 2.0, Color32::BLACK.gamma_multiply(0.55));
    painter.circle(
        at,
        r,
        if strong {
            pal.accent
        } else {
            pal.accent.gamma_multiply(0.75)
        },
        Stroke::new(1.0, Color32::BLACK.gamma_multiply(0.6)),
    );
}

/// A row's galley at `x`: its left edge, or its right edge when `right`.
fn paint_row(painter: &egui::Painter, galley: &Arc<Galley>, x: f32, y: f32, right: bool) {
    let x = if right { x - galley.size().x } else { x };
    painter.galley(Pos2::new(x, y), galley.clone(), Color32::WHITE);
}

/// One row of text: a faint label, then the text, cut to `width` with an ellipsis.
fn elided(
    ui: &egui::Ui,
    label: Option<(&str, Color32)>,
    text: &str,
    font: FontId,
    color: Color32,
    italics: bool,
    width: f32,
) -> Arc<Galley> {
    let mut job = LayoutJob::default();
    if let Some((label, label_color)) = label {
        job.append(
            label,
            0.0,
            TextFormat {
                font_id: font.clone(),
                color: label_color,
                ..Default::default()
            },
        );
    }
    job.append(
        text,
        if label.is_some() { 5.0 } else { 0.0 },
        TextFormat {
            font_id: font,
            color,
            italics,
            ..Default::default()
        },
    );
    job.wrap = TextWrapping {
        max_width: width.max(20.0),
        max_rows: 1,
        break_anywhere: true,
        overflow_character: Some('…'),
    };
    ui.fonts_mut(|f| f.layout_job(job))
}

/// The disc of a glyph: dark, with a ring that is the accent when `strong`.
fn disc(painter: &egui::Painter, center: Pos2, radius: f32, strong: bool, pal: &Pal) {
    painter.circle(
        center,
        radius,
        pal.fill,
        Stroke::new(
            if strong { 1.8 } else { 1.2 },
            if strong {
                pal.accent
            } else {
                pal.accent.gamma_multiply(0.55)
            },
        ),
    );
}

fn draw_hat(
    painter: &egui::Painter,
    rect: Rect,
    lines: &[CalloutLine],
    under: Option<usize>,
    dragging: bool,
    pal: &Pal,
) {
    let size = rect.width();
    let center = rect.center();
    let any = lines.iter().any(|l| l.lit);
    disc(painter, center, size * 0.5, any, pal);
    for (n, line) in lines.iter().enumerate().take(5) {
        let hot = line.lit || (dragging && under == Some(n));
        let color = if hot {
            pal.accent
        } else if line.empty {
            pal.weak.gamma_multiply(0.7)
        } else {
            pal.accent.gamma_multiply(0.8)
        };
        if n < 4 {
            painter.add(egui::Shape::convex_polygon(
                arrow_points(center, size, n),
                color,
                Stroke::NONE,
            ));
        } else if hot {
            painter.circle_filled(center, size * 0.12, color);
        } else {
            painter.circle_stroke(center, size * 0.12, Stroke::new(1.3, color));
        }
    }
}

/// Stick axes: a ring with an inner ring, arrows, the stick's position as a dot (x, y) and its
/// twist as a mark running along the rim.
fn draw_stick(painter: &egui::Painter, rect: Rect, c: &Callout, strong: bool, pal: &Pal) {
    let center = rect.center();
    let r = rect.width() * 0.5;
    let lit = c.lines.iter().any(|l| l.lit);
    disc(painter, center, r, strong || lit, pal);
    painter.circle_stroke(
        center,
        r * 0.58,
        Stroke::new(1.0, pal.accent.gamma_multiply(0.4)),
    );
    let ink = |on: bool| {
        if on {
            pal.accent
        } else {
            pal.accent.gamma_multiply(0.7)
        }
    };
    for n in 0..4 {
        painter.add(egui::Shape::convex_polygon(
            arrow_points(center, r * 1.3, n),
            ink(lit),
            Stroke::NONE,
        ));
    }
    // Twist: a short arc on the rim, from the top toward where the grip is turned.
    if let Some(twist) = c.axes.get(2).copied() {
        let turn = twist.clamp(-1.0, 1.0) * std::f32::consts::FRAC_PI_2;
        let steps = 12;
        let points: Vec<Pos2> = (0..=steps)
            .map(|k| {
                let a = -std::f32::consts::FRAC_PI_2 + turn * k as f32 / steps as f32;
                center + Vec2::angled(a) * (r - 3.0)
            })
            .collect();
        painter.line(points, Stroke::new(3.0, ink(twist.abs() > 0.15)));
        painter.circle_filled(
            center + Vec2::angled(-std::f32::consts::FRAC_PI_2 + turn) * (r - 3.0),
            2.5,
            ink(true),
        );
    }
    // Position: a dot inside the inner ring.
    let (x, y) = (
        c.axes.first().copied().unwrap_or(0.0).clamp(-1.0, 1.0),
        c.axes.get(1).copied().unwrap_or(0.0).clamp(-1.0, 1.0),
    );
    let at = center + Vec2::new(x, y) * r * 0.58;
    if c.axes.len() >= 2 {
        painter.line_segment(
            [center, at],
            Stroke::new(1.0, pal.accent.gamma_multiply(0.5)),
        );
    }
    painter.circle(
        at,
        r * 0.13,
        ink(x.hypot(y) > 0.15),
        Stroke::new(1.0, Color32::BLACK.gamma_multiply(0.5)),
    );
}

/// One axis: an upright bar filled from its middle to the value.
fn draw_gauge(painter: &egui::Painter, rect: Rect, c: &Callout, strong: bool, pal: &Pal) {
    let lit = c.lines.iter().any(|l| l.lit);
    painter.rect(
        rect,
        CornerRadius::same((rect.width() / 2.0) as u8),
        pal.fill,
        Stroke::new(
            if strong || lit { 1.8 } else { 1.2 },
            if strong || lit {
                pal.accent
            } else {
                pal.accent.gamma_multiply(0.55)
            },
        ),
        egui::StrokeKind::Inside,
    );
    let inner = rect.shrink(3.5);
    painter.line_segment(
        [inner.left_center(), inner.right_center()],
        Stroke::new(1.0, pal.accent.gamma_multiply(0.4)),
    );
    if let Some(v) = c.axes.first().copied() {
        let v = v.clamp(-1.0, 1.0);
        let y = inner.center().y - v * inner.height() / 2.0;
        let fill = Rect::from_x_y_ranges(
            inner.x_range(),
            y.min(inner.center().y)..=y.max(inner.center().y),
        );
        painter.rect_filled(
            fill,
            CornerRadius::same(2),
            pal.accent.gamma_multiply(if lit { 1.0 } else { 0.7 }),
        );
        painter.line_segment(
            [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
            Stroke::new(2.0, pal.accent),
        );
    }
}

fn draw_badge(
    painter: &egui::Painter,
    rect: Rect,
    line: &CalloutLine,
    under: bool,
    pal: &Pal,
    font: &FontId,
) {
    let hot = line.lit || under;
    painter.rect(
        rect,
        CornerRadius::same((BADGE / 2.0) as u8),
        if line.lit {
            pal.accent.gamma_multiply(0.85)
        } else {
            pal.fill
        },
        Stroke::new(
            if hot { 1.8 } else { 1.2 },
            if hot {
                pal.accent
            } else if line.empty {
                pal.weak.gamma_multiply(0.8)
            } else {
                pal.accent.gamma_multiply(0.6)
            },
        ),
        egui::StrokeKind::Inside,
    );
    let ink = if line.lit { pal.fill } else { pal.body };
    if line.badge.is_empty() {
        painter.circle_filled(rect.center(), 2.0, ink);
    } else {
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            &line.badge,
            FontId::proportional(font.size - 0.5),
            ink,
        );
    }
}

/// The compass part at `offset` from its center: the push dot in the middle (when the hat has
/// one), else the direction pointed at (0 up, 1 right, 2 down, 3 left).
fn compass_at(offset: Vec2, size: f32, push: bool) -> usize {
    if push && offset.length() < size * 0.18 {
        return 4;
    }
    if offset.y.abs() >= offset.x.abs() {
        if offset.y < 0.0 { 0 } else { 2 }
    } else if offset.x > 0.0 {
        1
    } else {
        3
    }
}

/// A small triangle pointing direction `n` (0 up, 1 right, 2 down, 3 left), near the rim of a
/// glyph `size` across.
fn arrow_points(center: Pos2, size: f32, n: usize) -> Vec<Pos2> {
    let (dir, side) = match n {
        0 => (Vec2::new(0.0, -1.0), Vec2::new(1.0, 0.0)),
        1 => (Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0)),
        2 => (Vec2::new(0.0, 1.0), Vec2::new(-1.0, 0.0)),
        _ => (Vec2::new(-1.0, 0.0), Vec2::new(0.0, -1.0)),
    };
    let tip = center + dir * size * 0.40;
    let base = center + dir * size * 0.27;
    vec![tip, base + side * size * 0.10, base - side * size * 0.10]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compass_parts() {
        let size = 50.0;
        assert_eq!(compass_at(Vec2::new(0.0, -20.0), size, true), 0);
        assert_eq!(compass_at(Vec2::new(20.0, 3.0), size, true), 1);
        assert_eq!(compass_at(Vec2::new(-2.0, 20.0), size, true), 2);
        assert_eq!(compass_at(Vec2::new(-20.0, 0.0), size, true), 3);
        assert_eq!(compass_at(Vec2::new(2.0, 2.0), size, true), 4);
        assert_eq!(compass_at(Vec2::new(0.0, -2.0), size, false), 0);
    }

    #[test]
    fn glyph_sizes() {
        let mut c = Callout {
            lines: vec!["a".into(), "b".into(), "c".into()],
            ..Default::default()
        };
        assert_eq!(
            glyph_size(&c),
            Vec2::new(3.0 * BADGE + 2.0 * BADGE_GAP, BADGE)
        );
        c.glyph = CalloutGlyph::Hat;
        assert_eq!(glyph_size(&c), Vec2::splat(HAT));
    }
}
