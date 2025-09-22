use egui::{Color32, Context, CornerRadius, Response, RichText, Ui};
use egui_flex::Flex;
use shakmaty::{Color, Move, Position, san::San};
use ucui_engine::Score;
// use ucui_utils::ucimovelist_to_sanlist;

use crate::{
    game::GameState,
    variation::{MoveIndex, Variation, VariationTree},
};

const MARGIN: f32 = 34.0;

fn title(game_state: &GameState) -> Option<String> {
    log::info!("title");
    if let Some(outcome) = game_state.game().outcome() {
        Some(outcome.to_string())
    } else if let Score::Mate { moves } = game_state.score {
        Some(format!("Mate in {moves}"))
    // } else if let Score::CentiPawns { score, pv } = &game_state.score {
    //     let mut game = Chess::new();
    //     let moves = game_state.tree.moves();
    //     if moves.is_empty() {
    //         return None;
    //     }
    //     let n = moves.len() - 1;
    //     for m in moves.iter().take(n) {
    //         let _ = game.clone().play(m).map(|new_game| {
    //             game = new_game;
    //         });
    //     }
    //     let start = game.fullmoves();
    //     let sanlist = if game.turn() == Color::Black {
    //         let mut sanlist = vec![String::from("…")];
    //         sanlist.extend(ucimovelist_to_sanlist(game, pv));
    //         sanlist
    //     } else {
    //         ucimovelist_to_sanlist(game, pv)
    //     };
    //     let moves: Vec<String> = sanlist
    //         .chunks(2)
    //         .enumerate()
    //         .map(|(i, pair)| match (pair.get(0), pair.get(1)) {
    //             (Some(a), Some(b)) => {
    //                 format!("{}.{} {}", start.saturating_add(i as u32), a, b)
    //             }
    //             (Some(a), None) => {
    //                 format!("{}.{} …", start.saturating_add(i as u32), a)
    //             }
    //             _ => String::from("??"),
    //         })
    //         .collect();
    //     Some(format!("[{}]  {}", *score as f32 / 100.0, moves.join("  ")))
    } else {
        game_state.opening.clone().and_then(|eco| {
            if eco.moves.len() >= game_state.tree.moves().len() {
                Some(eco.name.clone())
            } else {
                None
            }
        })
    }
}

static MOVE_TEXT_LINE_HEIGHT: f32 = 22.0;

fn move_text<S>(depth: usize, current_move: bool, txt: S) -> RichText
where
    S: AsRef<str>,
{
    let rsize = 10.0 - depth as f32;
    let size = rsize * MOVE_TEXT_LINE_HEIGHT / 10.0;
    let mut base = RichText::new(txt.as_ref())
        .size(size)
        .color(Color32::BLACK)
        .line_height(Some(MOVE_TEXT_LINE_HEIGHT));
    if depth > 0 {
        base = base.italics()
    }
    if current_move {
        base = base.color(Color32::from_rgb(254, 13, 0))
    }
    base
}

fn ord_text(depth: usize, ord: std::num::NonZeroU32) -> RichText {
    let rsize = 10.0 - depth as f32;
    let size = rsize * (MOVE_TEXT_LINE_HEIGHT * 0.8) / 10.0;
    RichText::new(format!("{ord}."))
        .size(size)
        .color(Color32::BLACK)
        .line_height(Some(MOVE_TEXT_LINE_HEIGHT))
}

type MoveChunk<'a> = (Option<&'a Move>, Option<&'a Move>);

fn label(ui: &mut Ui, text: impl Into<egui::WidgetText>) -> Response {
    let c0 = ui.cursor();
    let aw = ui.available_width();
    // ui.set_max_width(aw);
    let text_widget = text.into();
    let r = ui.label(text_widget.clone());

    let c1 = ui.cursor();
    log::info!(
        "<{:?}> aw: {}, w {} tx {} ty {}",
        text_widget.clone(),
        aw,
        r.rect.width(),
        c1.min.x - c0.min.x,
        c1.min.y - c0.min.y
    );

    r
}

fn layout_moves<R>(is_main: bool, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) {
    if is_main {
        log::info!("layout_moves with layout");
        ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), add_contents);
    } else {
        log::info!("layout_moves *without* layout - {}", ui.available_width());
        add_contents(ui);
    }
}

fn render_chunk<'a>(
    ui: &mut Ui,
    tree: &VariationTree,
    chunk: MoveChunk<'a>,
    index: MoveIndex,
    depth: usize,
) -> Option<MoveIndex> {
    log::info!("render_chunk {:?}; {:?}", chunk, index);
    let mut clicked_index: Option<MoveIndex> = None;
    let is_main = depth == 0;
    layout_moves(is_main, ui, |ui| {
        match chunk {
            (Some(white), Some(black)) => {
                let game = tree.game_at(index).unwrap();
                let white_variations = tree.variations_from(index);
                let black_variations = tree.variations_from(index.incr_move());
                if white_variations.is_empty() {
                    // we print both moves

                    label(ui, ord_text(depth, game.fullmoves()));
                    if label(
                        ui,
                        move_text(
                            depth,
                            tree.is_current(index),
                            San::from_move(&game, white).to_string(),
                        ),
                    )
                    .clicked()
                    {
                        clicked_index = Some(index);
                    };

                    if let Ok(game) = game.play(black) {
                        if label(
                            ui,
                            move_text(
                                depth,
                                tree.is_current(index.incr_move()),
                                San::from_move(&game, black).to_string(),
                            ),
                        )
                        .clicked()
                        {
                            clicked_index = Some(index.incr_move());
                        };
                    }
                } else {
                    // print white move, then white variations, then black move
                    // layout_moves(is_main, ui, |ui| {
                    label(ui, ord_text(depth, game.fullmoves()));
                    if label(
                        ui,
                        move_text(
                            depth,
                            tree.is_current(index),
                            San::from_move(&game, white).to_string(),
                        ),
                    )
                    .clicked()
                    {
                        clicked_index = Some(index);
                    };
                    label(ui, move_text(depth, false, format!("…")));
                    // });
                    for var in white_variations {
                        let clicked = render_variation(ui, tree, depth + 1, var);
                        if clicked_index.is_none() {
                            clicked_index = clicked;
                        }
                    }
                    // layout_moves(is_main, ui, |ui| {
                    label(ui, ord_text(depth, game.fullmoves()));
                    label(ui, move_text(depth, false, format!("…")));

                    if let Ok(game) = game.play(black) {
                        if label(
                            ui,
                            move_text(
                                depth,
                                tree.is_current(index.incr_move()),
                                San::from_move(&game, black).to_string(),
                            ),
                        )
                        .clicked()
                        {
                            clicked_index = Some(index.incr_move());
                        };
                    }
                    // });
                }
                for var in black_variations {
                    let clicked = render_variation(ui, tree, depth + 1, var);
                    if clicked_index.is_none() {
                        clicked_index = clicked;
                    }
                }
            }
            (Some(white), None) => {
                let game = tree.game_at(index).unwrap();
                let white_variations = tree.variations_from(index);
                // layout_moves(is_main, ui, |ui| {
                label(ui, ord_text(depth, game.fullmoves()));
                if label(
                    ui,
                    move_text(
                        depth,
                        tree.is_current(index),
                        San::from_move(&game, white).to_string(),
                    ),
                )
                .clicked()
                {
                    clicked_index = Some(index);
                };
                label(ui, move_text(depth, false, format!("…")));
                // });
                for var in white_variations {
                    let clicked = render_variation(ui, tree, depth + 1, var);
                    if clicked_index.is_none() {
                        clicked_index = clicked;
                    }
                }
            }
            (None, Some(black)) => {
                let game = tree.game_at(index).unwrap();
                let black_variations = tree.variations_from(index.incr_move());
                // layout_moves(is_main, ui, |ui| {
                label(ui, ord_text(depth, game.fullmoves()));
                label(ui, move_text(depth, false, format!("…")));

                if let Ok(game) = game.play(black) {
                    if label(
                        ui,
                        move_text(
                            depth,
                            tree.is_current(index.incr_move()),
                            San::from_move(&game, black).to_string(),
                        ),
                    )
                    .clicked()
                    {
                        clicked_index = Some(index.incr_move());
                    };
                }
                // });
                for var in black_variations {
                    let clicked = render_variation(ui, tree, depth + 1, var);
                    if clicked_index.is_none() {
                        clicked_index = clicked;
                    }
                }
            }
            (None, None) => {}
        }
    });
    clicked_index
}

fn render_variation(
    ui: &mut Ui,
    tree: &VariationTree,
    depth: usize,
    var: Variation,
) -> Option<MoveIndex> {
    // assert!(depth < 10, "bad recursion");
    // ui.separator();

    let mut index = var.index;

    if tree.is_current_variation(index) {
        ui.style_mut().visuals.faint_bg_color = Color32::GRAY; // FIXME
    }

    let turn = var.game(index.move_index()).turn();
    log::info!("render_variation {} {:?} {}", depth, index, turn);

    let (first_chunk, remaining_moves) = if turn == Color::Black {
        // it looks weird, but the cursor is *on* the move
        ((var.moves.get(0), var.moves.get(1)), var.moves.get(2..))
    } else {
        ((None, var.moves.get(0)), var.moves.get(1..))
    };

    let mut clicked_index: Option<MoveIndex> = None;
    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
        *(&mut clicked_index) = render_chunk(ui, tree, first_chunk, index, depth);

        if let Some(remaining_moves) = remaining_moves {
            for chunk in remaining_moves.chunks(2) {
                index = index.incr_move().incr_move();

                let mut clicked = None;
                ui.horizontal_wrapped(|ui| {
                    log::info!(
                        "Horizotal {} {} ",
                        ui.available_size(),
                        ui.available_size_before_wrap()
                    );
                    *(&mut clicked) =
                        render_chunk(ui, tree, (chunk.get(0), chunk.get(1)), index, depth);
                });

                if clicked_index.is_none() {
                    clicked_index = clicked;
                }
            }
        }
    });

    // ui.separator();
    clicked_index
}

fn render_root_variation(
    ui: &mut Ui,
    tree: &VariationTree,
    depth: usize,
    var: Variation,
) -> Option<MoveIndex> {
    ui.separator();

    let mut index = var.index;

    if tree.is_current_variation(index) {
        ui.style_mut().visuals.faint_bg_color = Color32::GRAY; // FIXME
    }

    let turn = var.game(index.move_index()).turn();
    log::info!("render_variation {} {:?} {}", depth, index, turn);

    let (first_chunk, remaining_moves) = if turn == Color::Black {
        // it looks weird, but the cursor is *on* the move
        ((var.moves.get(0), var.moves.get(1)), var.moves.get(2..))
    } else {
        ((None, var.moves.get(0)), var.moves.get(1..))
    };

    let mut clicked_index: Option<MoveIndex> = render_chunk(ui, tree, first_chunk, index, depth);

    if let Some(remaining_moves) = remaining_moves {
        for chunk in remaining_moves.chunks(2) {
            index = index.incr_move().incr_move();
            let clicked = render_chunk(ui, tree, (chunk.get(0), chunk.get(1)), index, depth);

            if clicked_index.is_none() {
                clicked_index = clicked;
            }
        }
    }

    ui.separator();
    clicked_index
}

pub fn render_game_side(ctx: &Context, ui: &mut Ui, state: &mut GameState) {
    log::info!("render_game_side");

    let lid = ui.layer_id();
    let rect = ui.max_rect();
    let painter = egui::Painter::new(ctx.clone(), lid, rect);

    let _ = painter.rect_filled(rect, CornerRadius::ZERO, Color32::WHITE);

    title(state).map(|title| {
        ui.label(title);
        ui.separator();
    });

    ui.add_space(MARGIN);
    log::info!("start variations");

    if let Some(root) = state.tree.root_variation() {
        if let Some(move_index) = render_root_variation(ui, &mut state.tree, 0, root) {
            state.at(move_index);
        }
    }
}

pub fn render_tools_side(ctx: &Context, ui: &mut Ui) {
    let lid = ui.layer_id();
    let rect = ui.max_rect();
    let painter = egui::Painter::new(ctx.clone(), lid, rect);

    let _ = painter.rect_filled(rect, CornerRadius::ZERO, Color32::WHITE);

    ui.label("Tools");
    ui.separator();
}
