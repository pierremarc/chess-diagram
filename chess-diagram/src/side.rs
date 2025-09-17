use egui::{Color32, Context, CornerRadius, Ui};
use shakmaty::san::San;

use crate::game::GameState;

const MARGIN: f32 = 34.0;

pub fn render_side(ctx: &Context, ui: &mut Ui, state: &GameState) {
    let lid = ui.layer_id();
    let rect = ui.max_rect();
    let painter = egui::Painter::new(ctx.clone(), lid, rect);

    let _ = painter.rect_filled(rect, CornerRadius::ZERO, Color32::WHITE);

    state.opening.as_ref().map(|opening| {
        // let _ = painter.text(
        //     pos2(rect.width() / 2.0, MARGIN / 4.0),
        //     Align2::CENTER_TOP,
        //     opening,
        //     FontId::proportional(12.0),
        //     Color32::BLACK,
        // );
        ui.label(opening.name.clone());
        ui.separator();
    });

    ui.add_space(MARGIN);
    for (i, pair) in state.moves.chunks(2).into_iter().enumerate() {
        match (pair.get(0), pair.get(1)) {
            (Some(a), Some(b)) => {
                let sana = San::from_move(&state.game, a).to_string();
                let sanb = San::from_move(&state.game, b).to_string();
                let spacing = vec![" "; 10 - sana.len()]
                    .iter()
                    .map(|s| format!("{s}"))
                    .collect::<String>();
                ui.label(format!("{}. {}{}{}", i + 1, sana, spacing, sanb));
            }
            (Some(a), None) => {
                ui.label(format!(
                    "{}. {}  ...",
                    i + 1,
                    San::from_move(&state.game, a)
                ));
            }
            _ => {}
        }
    }
}
