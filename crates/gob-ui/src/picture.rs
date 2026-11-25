//! A picture of a device with callouts: the picture in the middle, a column of cards on each
//! side, and an elbow leader line from every card to its spot on the picture. Each card can be
//! hovered, clicked and dropped on (`Response::dnd_release_payload`).
//!
//! The app brings the picture (any [`egui::Image`]: a photo or a rasterized drawing), its size in
//! the units the spots use (pixels of a photo, the viewBox of a drawing) and the cards' texts.

use egui::{Align2, Color32, CornerRadius, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};

use crate::theme::accent;
use crate::tokens::{RADIUS, SPACE_S, SPACE_XS};

/// Which column a card goes to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalloutSide {
    Left,
    Right,
}

/// One card and its spot.
#[derive(Debug, Clone)]
pub struct Callout {
    /// The spot, in picture units.
    pub at: Pos2,
    pub side: CalloutSide,
    pub title: String,
    /// One line each under the title (what the control does, its inputs).
    pub lines: Vec<String>,
    /// A heading over a run of cards on the same side ("Base buttons"); cards of one group
    /// should be next to each other in the list.
    pub group: Option<String>,
    /// Drawn in the accent color (the control is held now, or selected).
    pub lit: bool,
}

/// What the picture did this frame.
pub struct PictureResponse {
    pub response: egui::Response,
    /// One per callout, in the order given: the card and its spot together.
    pub callouts: Vec<egui::Response>,
    /// The callout under the pointer.
    pub hovered: Option<usize>,
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

const COLUMN_MIN: f32 = 130.0;
const COLUMN_MAX: f32 = 230.0;
const SPOT_RADIUS: f32 = 6.0;

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

    /// The picture is never taller than this (the cards may still reach further).
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
        let column = (width * 0.26).clamp(COLUMN_MIN, COLUMN_MAX);
        let middle = (width - 2.0 * (column + SPACE_S)).max(80.0);
        let scale = (middle / self.size.x)
            .min(self.max_height / self.size.y)
            .max(0.01);
        let shown = self.size * scale;

        let small = FontId::proportional(ui.text_style_height(&egui::TextStyle::Small) + 1.0);
        let body = egui::TextStyle::Body.resolve(ui.style());
        let line_h = ui.fonts_mut(|f| f.row_height(&small));
        let title_h = ui.fonts_mut(|f| f.row_height(&body));
        let pad = SPACE_XS + 2.0;
        let card_h = |c: &Callout| title_h + c.lines.len() as f32 * line_h + 2.0 * pad;
        let group_h = line_h + SPACE_XS;

        // Cards stacked per side in the order of their spots, each as near its spot's height
        // as the cards above allow.
        let mut tops = vec![0.0; self.callouts.len()];
        let mut heading = vec![false; self.callouts.len()];
        let mut bottom = shown.y;
        for side in [CalloutSide::Left, CalloutSide::Right] {
            let mut order: Vec<usize> = (0..self.callouts.len())
                .filter(|&i| self.callouts[i].side == side)
                .collect();
            order.sort_by(|&a, &b| {
                let (a, b) = (&self.callouts[a], &self.callouts[b]);
                a.at.y.total_cmp(&b.at.y)
            });
            group_runs(&mut order, self.callouts);
            let mut next = 0.0f32;
            let mut last_group: Option<&str> = None;
            for &i in &order {
                let c = &self.callouts[i];
                let group = c.group.as_deref();
                if group.is_some() && group != last_group {
                    heading[i] = true;
                    next += group_h;
                }
                last_group = group;
                let want = c.at.y * scale - card_h(c) / 2.0;
                tops[i] = want.max(next);
                next = tops[i] + card_h(c) + SPACE_XS;
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
        self.image.paint_at(ui, picture);
        let painter = ui.painter_at(rect.expand(SPOT_RADIUS));
        if let Some(caption) = self.caption {
            painter.text(
                Pos2::new(picture.center().x, picture.bottom() + SPACE_XS),
                Align2::CENTER_TOP,
                caption,
                small.clone(),
                ui.visuals().weak_text_color(),
            );
        }

        let visuals = ui.visuals();
        let accent = accent(ui);
        let line = visuals
            .widgets
            .noninteractive
            .fg_stroke
            .color
            .gamma_multiply(0.55);
        let card_fill = visuals.faint_bg_color;
        let text = visuals.text_color();
        let weak = visuals.weak_text_color();
        let pointer = ui.ctx().pointer_hover_pos();

        let mut callouts = Vec::with_capacity(self.callouts.len());
        let mut hovered = None;
        for (i, c) in self.callouts.iter().enumerate() {
            let x = match c.side {
                CalloutSide::Left => rect.left(),
                CalloutSide::Right => rect.right() - column,
            };
            let card = Rect::from_min_size(
                Pos2::new(x, rect.top() + tops[i]),
                Vec2::new(column, card_h(c)),
            );
            let spot = picture.min + c.at.to_vec2() * scale;
            let spot_rect = Rect::from_center_size(spot, Vec2::splat(SPOT_RADIUS * 3.0));
            let id = self.id.with(i);
            let card_response = ui.interact(card, id, Sense::click());
            let spot_sense = if self.draggable {
                Sense::click_and_drag()
            } else {
                Sense::click()
            };
            let spot_response = ui.interact(spot_rect, id.with("spot"), spot_sense);
            let both = card_response.union(spot_response);
            let hot = both.hovered()
                || pointer.is_some_and(|p| card.contains(p) || spot_rect.contains(p));
            if hot {
                hovered = Some(i);
            }
            let color = if c.lit || hot { accent } else { line };

            if heading[i] {
                painter.text(
                    Pos2::new(card.left() + pad, card.top() - group_h),
                    Align2::LEFT_TOP,
                    c.group.as_deref().unwrap_or_default(),
                    small.clone(),
                    weak,
                );
            }
            // Leader: out of the card's inner edge to an elbow beside the picture, then to the spot.
            let (start, elbow_x) = match c.side {
                CalloutSide::Left => (
                    Pos2::new(card.right(), card.top() + pad + title_h / 2.0),
                    picture.left().min(card.right() + SPACE_S) - SPACE_XS,
                ),
                CalloutSide::Right => (
                    Pos2::new(card.left(), card.top() + pad + title_h / 2.0),
                    picture.right().max(card.left() - SPACE_S) + SPACE_XS,
                ),
            };
            let elbow = Pos2::new(elbow_x, start.y);
            let width = if c.lit || hot { 1.6 } else { 1.0 };
            painter.line(vec![start, elbow, spot], Stroke::new(width, color));
            painter.circle(
                spot,
                SPOT_RADIUS,
                if c.lit {
                    accent.gamma_multiply(0.45)
                } else {
                    Color32::TRANSPARENT
                },
                Stroke::new(
                    2.0,
                    if c.lit || hot {
                        accent
                    } else {
                        Color32::WHITE.gamma_multiply(0.85)
                    },
                ),
            );
            painter.circle_stroke(
                spot,
                SPOT_RADIUS + 1.5,
                Stroke::new(1.0, Color32::BLACK.gamma_multiply(0.5)),
            );

            painter.rect(
                card,
                CornerRadius::same(RADIUS),
                card_fill,
                Stroke::new(
                    1.0,
                    if c.lit || hot {
                        accent
                    } else {
                        line.gamma_multiply(0.6)
                    },
                ),
                StrokeKind::Inside,
            );
            let clip = painter.with_clip_rect(card.shrink(2.0));
            let mut y = card.top() + pad;
            clip.text(
                Pos2::new(card.left() + pad + 2.0, y),
                Align2::LEFT_TOP,
                &c.title,
                body.clone(),
                if c.lit { accent } else { text },
            );
            y += title_h;
            for l in &c.lines {
                clip.text(
                    Pos2::new(card.left() + pad + 2.0, y),
                    Align2::LEFT_TOP,
                    l,
                    small.clone(),
                    weak,
                );
                y += line_h;
            }
            callouts.push(both);
        }
        PictureResponse {
            response,
            callouts,
            hovered,
            picture,
            scale,
        }
    }
}

/// Keeps every group's cards together: a run starts where the group's first card would go.
fn group_runs(order: &mut Vec<usize>, callouts: &[Callout]) {
    let mut out: Vec<usize> = Vec::with_capacity(order.len());
    for &i in order.iter() {
        if out.contains(&i) {
            continue;
        }
        out.push(i);
        if let Some(g) = &callouts[i].group {
            out.extend(
                order
                    .iter()
                    .copied()
                    .filter(|&j| j != i && callouts[j].group.as_ref() == Some(g)),
            );
        }
    }
    *order = out;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn callout(y: f32, group: Option<&str>) -> Callout {
        Callout {
            at: Pos2::new(0.0, y),
            side: CalloutSide::Left,
            title: String::new(),
            lines: Vec::new(),
            group: group.map(Into::into),
            lit: false,
        }
    }

    #[test]
    fn groups_stay_together() {
        let c = [
            callout(10.0, Some("base")),
            callout(20.0, None),
            callout(30.0, Some("base")),
        ];
        let mut order = vec![0, 1, 2];
        group_runs(&mut order, &c);
        assert_eq!(order, [0, 2, 1]);
    }
}
