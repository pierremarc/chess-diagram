use egui::{Color32, Context, CornerRadius, Label, RichText, Separator, Ui};
use egui_flex::{Flex, item};
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

fn move_none() -> (Text, Option<MoveIndex>) {
    (
        Text::new("…").color(Color32::BLACK), // .line_height(Some(MOVE_TEXT_LINE_HEIGHT))
        None,
    )
}

fn ord_text(depth: usize, ord: std::num::NonZeroU32) -> RichText {
    let rsize = 10.0 - depth as f32;
    let size = rsize * (MOVE_TEXT_LINE_HEIGHT * 0.8) / 10.0;
    RichText::new(format!("{ord}."))
        .size(size)
        .color(Color32::BLACK)
        .line_height(Some(MOVE_TEXT_LINE_HEIGHT))
}

type MoveChunk = (Option<Move>, Option<Move>);

type Text = RichText;

type PrintMove = (Text, Option<MoveIndex>);
enum PrintUnit {
    FullMove {
        ord: Text,
        white: PrintMove,
        black: PrintMove,
    },
    VariationStart,
    VariationEnd,
}

fn print_move(
    ord: Text,
    white: (Text, Option<MoveIndex>),
    black: (Text, Option<MoveIndex>),
) -> PrintUnit {
    PrintUnit::FullMove { ord, white, black }
}

fn print_variation_start() -> PrintUnit {
    PrintUnit::VariationStart
}

fn print_variation_end() -> PrintUnit {
    PrintUnit::VariationEnd
}

fn append_units(
    units: &mut Vec<PrintUnit>,
    tree: &VariationTree,
    chunk: MoveChunk,
    index: MoveIndex,
    depth: usize,
) {
    log::info!("make_unit {:?}; {:?}", chunk, index);

    let game = tree.game_at(index).unwrap();
    let ord = ord_text(depth, game.fullmoves());
    let white_variations = tree.variations_from(index);
    let black_variations = tree.variations_from(index.incr_move());

    match chunk {
        (Some(white), Some(black)) => {
            if white_variations.is_empty() {
                // we print both moves
                if let Ok(game) = game.play(&black) {
                    units.push(print_move(
                        ord,
                        (
                            move_text(
                                depth,
                                tree.is_current(index),
                                San::from_move(&game, &white).to_string(),
                            ),
                            Some(index),
                        ),
                        (
                            move_text(
                                depth,
                                tree.is_current(index.incr_move()),
                                San::from_move(&game, &black).to_string(),
                            ),
                            Some(index.incr_move()),
                        ),
                    ));
                }
            } else {
                // print white move, then white variations, then black move
                units.push(print_move(
                    ord.clone(),
                    (
                        move_text(
                            depth,
                            tree.is_current(index),
                            San::from_move(&game, &white).to_string(),
                        ),
                        Some(index),
                    ),
                    move_none(),
                ));

                for var in white_variations {
                    render_variation(units, tree, depth + 1, var);
                }

                if let Ok(game) = game.play(&black) {
                    units.push(print_move(
                        ord,
                        move_none(),
                        (
                            move_text(
                                depth,
                                tree.is_current(index.incr_move()),
                                San::from_move(&game, &black).to_string(),
                            ),
                            Some(index.incr_move()),
                        ),
                    ));
                }
            }

            for var in black_variations {
                render_variation(units, tree, depth + 1, var);
            }
        }
        (Some(white), None) => {
            units.push(print_move(
                ord,
                (
                    move_text(
                        depth,
                        tree.is_current(index),
                        San::from_move(&game, &white).to_string(),
                    ),
                    Some(index),
                ),
                move_none(),
            ));

            for var in white_variations {
                render_variation(units, tree, depth + 1, var);
            }
        }
        (None, Some(black)) => {
            if let Ok(game) = game.play(&black) {
                units.push(print_move(
                    ord,
                    move_none(),
                    (
                        move_text(
                            depth,
                            tree.is_current(index.incr_move()),
                            San::from_move(&game, &black).to_string(),
                        ),
                        Some(index.incr_move()),
                    ),
                ));
            }

            for var in black_variations {
                render_variation(units, tree, depth + 1, var);
            }
        }
        (None, None) => {}
    }
}

fn render_variation(
    units: &mut Vec<PrintUnit>,
    tree: &VariationTree,
    depth: usize,
    var: Variation,
) {
    let index = var.index;
    let turn = var.game(index.move_index()).turn();
    log::info!("render_variation {} {:?} {}", depth, index, turn);

    units.push(print_variation_start());

    let (first_chunk, remaining_moves) = if turn == Color::Black {
        // it looks weird, but the cursor is *on* the move
        (
            (var.moves.get(0).cloned(), var.moves.get(1).cloned()),
            var.moves
                .get(2..)
                .map(|m| m.iter().map(|m| m.clone()).collect::<Vec<_>>()),
        )
    } else {
        (
            (None, var.moves.get(0).cloned()),
            var.moves
                .get(1..)
                .map(|m| m.iter().map(|m| m.clone()).collect::<Vec<_>>()),
        )
    };

    append_units(units, tree, first_chunk, index, depth);
    if let Some(remaining_moves) = remaining_moves {
        for chunk in remaining_moves.chunks(2) {
            append_units(
                units,
                tree,
                (chunk.get(0).cloned(), chunk.get(1).cloned()),
                index.incr_move().incr_move(),
                depth,
            );
        }
    }

    units.push(print_variation_end());
}

fn render_chunk<'a>(
    state: &mut GameState,
    container: &mut egui_flex::FlexInstance<'a>,
    (ord, (white_move, white_index), (black_move, black_index)): (Text, PrintMove, PrintMove),
) {
    container.add(item(), Label::new(ord));
    if container.add(item(), Label::new(white_move)).clicked() {
        white_index.map(|i| state.at(i));
    }
    if container.add(item(), Label::new(black_move)).clicked() {
        black_index.map(|i| state.at(i));
    }
}

fn drain_units<'a>(
    state: &mut GameState,
    root: &mut egui_flex::FlexInstance<'a>,
    accum: &mut Vec<(Text, PrintMove, PrintMove)>,
    current_depth: usize,
) {
    if !accum.is_empty() {
        let sub_flex = if current_depth > 0 {
            Flex::horizontal().wrap(true)
            // Flex::vertical()
        } else {
            Flex::vertical()
        };
        root.add_flex(item(), Flex::vertical(), |container| {
            container.add_flex(item(), sub_flex, |container| {
                for (ord, (white_move, white_index), (black_move, black_index)) in
                    accum.clone().into_iter()
                {
                    render_chunk(
                        state,
                        container,
                        (ord, (white_move, white_index), (black_move, black_index)),
                    );
                }
            });
        });
        accum.clear();
    }
}

fn render_root_variation(state: &mut GameState, ui: &mut Ui, var: Variation) {
    let mut units: Vec<PrintUnit> = Vec::new();
    render_variation(&mut units, &state.tree, 0, var);

    Flex::vertical().show(ui, |container| {
        let mut current_depth = 0;
        let mut accum: Vec<(Text, PrintMove, PrintMove)> = Vec::new();
        let accum_ref = &mut accum;

        for unit in units {
            match unit {
                PrintUnit::VariationStart => {
                    current_depth += 1;
                }
                PrintUnit::VariationEnd => {
                    drain_units(state, container, accum_ref, current_depth);
                    current_depth -= 1;
                }
                PrintUnit::FullMove { ord, white, black } => {
                    accum_ref.push((ord, white, black));
                }
            }
        }
    });
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
        render_root_variation(state, ui, root);
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
