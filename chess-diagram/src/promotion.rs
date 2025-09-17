use egui::{Color32, Context, CornerRadius, Rect, StrokeKind, Ui, pos2, vec2};
use shakmaty::{Color, Role};

use crate::{
    board::{board_rect, file_to_index, rank_to_index},
    gesture::Gesture,
    sources::Sources,
};

pub fn render_promotion(ctx: &Context, ui: &mut Ui, sources: &Sources<'_>, gesture: &mut Gesture) {
    use Role::*;

    if let Gesture::End(state) = *gesture {
        let lid = ui.layer_id();
        let rect = ui.max_rect();
        let painter = egui::Painter::new(ctx.clone(), lid, rect);
        let board_rect = board_rect(&rect);
        let square_size = board_rect.width() / 8.0;

        let rank_index = rank_to_index(state.to().rank());
        let file_index = file_to_index(state.to().file());

        let left = file_index as f32 * square_size + board_rect.min.x;

        let top = if state.piece().color == Color::Black {
            rank_index as f32 * square_size + board_rect.min.y - (3.0 * square_size)
        } else {
            rank_index as f32 * square_size + board_rect.min.y
        };

        // log::info!("P> {left} {top}");

        let _ = painter.rect_stroke(
            Rect::from_min_size(pos2(left, top), vec2(square_size, 4.0 * square_size)).expand(12.0),
            CornerRadius::same(12),
            (4.0, Color32::BLACK),
            StrokeKind::Middle,
        );
        let _ = painter.rect_filled(
            Rect::from_min_size(pos2(left, top), vec2(square_size, 4.0 * square_size)).expand(12.0),
            CornerRadius::same(12),
            Color32::WHITE,
        );

        let mut buttons: Vec<(Rect, Role)> = Vec::with_capacity(5);

        for (i, role) in [Queen, Rook, Bishop, Knight].iter().enumerate() {
            let piece_name = if state.piece().color == Color::Black {
                format!("{}", role.char()).to_lowercase()
            } else {
                format!("{}", role.char()).to_uppercase()
            };

            if let Some(image) = sources.get(piece_name) {
                let offset = i as f32 * square_size;
                let rect =
                    Rect::from_min_size(pos2(left, top + offset), vec2(square_size, square_size));
                // log::info!("R> {rect} {role:?}");
                image.paint_at(ui, rect);
                buttons.push((rect, role.clone()));
            }
        }

        ui.input(|input| {
            if input.pointer.primary_clicked() {
                if let Some(position) = input.pointer.interact_pos() {
                    for (rect, role) in buttons {
                        if rect.contains(position) {
                            log::info!("=> {} {}  -> {:?}", rect, position, role);
                            gesture.promote(role);
                            return;
                        }
                    }
                }
            }
        })
    }

    // if ui.label("Queen").clicked() {
    //     gesture.promote(Queen);
    // }
    // ui.separator();
    // if ui.label("Rook").clicked() {
    //     gesture.promote(Rook);
    // }
    // ui.separator();
    // if ui.label("Bishop").clicked() {
    //     gesture.promote(Bishop);
    // }
    // ui.separator();
    // if ui.label("Knight").clicked() {
    //     gesture.promote(Knight);
    // }
}
