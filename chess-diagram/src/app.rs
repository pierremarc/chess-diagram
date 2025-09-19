use std::cell::RefCell;
use std::fmt::format;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};

use egui::Key;
use egui_extras::install_image_loaders;
use log::info;
use shakmaty::fen::Fen;
use shakmaty::san::San;
use shakmaty::{Chess, Color, Move, Position};
use ucui_engine::Score;
use ucui_utils::ucimovelist_to_sanlist;

use crate::board::{render_board, square_at};
use crate::config::get_engine_color;
use crate::game::GameState;
use crate::gesture::{Gesture, StateStart};
use crate::promotion::render_promotion;
use crate::proxy::{Proxy, start_engine};
use crate::sources::Sources;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum BoardMode {
    Play,
    Setup,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum PointerMode {
    Drag,
    Click,
}

pub struct DiagramApp<'a> {
    gesture: Rc<RefCell<Gesture>>,
    game: Arc<RwLock<GameState>>,
    engine: Arc<Proxy>,
    sources: Sources<'a>,
    board_mode: BoardMode,
    pointer_mode: PointerMode,
}

impl<'a> DiagramApp<'a> {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        // } else {
        //     Default::default()
        // }

        install_image_loaders(&cc.egui_ctx);
        let ctx = Arc::new(Mutex::new(cc.egui_ctx.clone()));

        let game_state = Arc::new(RwLock::new(GameState::new(get_engine_color(), None)));
        DiagramApp {
            gesture: Rc::new(RefCell::new(Gesture::new())),
            game: game_state.clone(),
            sources: Sources::new(),
            engine: Arc::new(start_engine(game_state.clone(), ctx)),
            board_mode: BoardMode::Play,
            pointer_mode: PointerMode::Drag,
        }
    }
}

impl<'a> DiagramApp<'a> {
    fn new_game(&mut self) {
        if let Ok(mut game_state) = self.game.write() {
            game_state.moves = Vec::new();
            game_state.game = Chess::new();
            self.engine.new_game();
        }
    }

    fn set_board_mode(&mut self, mode: BoardMode) {
        use BoardMode::*;
        match mode {
            Setup => self.board_mode = Setup,
            Play => {
                self.board_mode = Play;
                if let Ok(game_state) = self.game.read() {
                    self.engine.play(
                        Fen::from_position(game_state.game.clone(), shakmaty::EnPassantMode::Legal)
                            .to_string(),
                    );
                }
            }
        }
    }
    fn toggle_pointer_mode(&mut self) {
        use PointerMode::*;
        let _ = self.gesture.try_borrow_mut().map(|mut gesture| {
            *gesture = Gesture::None;
        });
        match self.pointer_mode {
            Drag => self.pointer_mode = Click,
            Click => self.pointer_mode = Drag,
        }
    }
}

impl<'a> eframe::App for DiagramApp<'a> {
    /// Called by the framework to save state before shutdown.

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // let gesture = &mut self.gesture;
        let gesture = self.gesture.clone();
        let game_state = self.game.clone();
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            let toggle_pointer = format!(
                "Toggle {}",
                if self.pointer_mode == PointerMode::Click {
                    "drag"
                } else {
                    "click"
                }
            );
            ui.horizontal(|ui| {
                let keys = [
                    ("Q", "Quit"),
                    ("F", "Fullscreen"),
                    ("N", "New game"),
                    ("S", "Setup"),
                    ("P", "Engine Play"),
                    ("I", toggle_pointer.as_str()),
                ];
                for (key, label) in keys {
                    ui.label(format!("[{key}] {label}"));
                    ui.separator();
                }
            });
        });

        // {
        //     egui::SidePanel::right("side")
        //         .resizable(false)
        //         .show_separator_line(false)
        //         .min_width(ctx.screen_rect().width() * 0.1)
        //         .max_width(ctx.screen_rect().width() * 0.2)
        //         .frame(egui::Frame::NONE)
        //         .show(ctx, |ui| {
        //             ui.label("Info");
        //             let game_state = game_state.read().unwrap();
        //             render_side(ctx, ui, &game_state);
        //         });
        // }

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                {
                    let gesture = gesture.borrow();
                    let game_state = game_state.read().unwrap();
                    let title = if let Some(outcome) = game_state.game.outcome() {
                        Some(outcome.to_string())
                    } else if let Score::Mate { moves } = game_state.score {
                        Some(format!("Mate in {moves}"))
                    } else if let Score::CentiPawns { score, pv } = &game_state.score {
                        let mut game = Chess::new();
                        let n = game_state.moves.len() - 1;
                        for m in game_state.moves.iter().take(n) {
                            let _ = game.clone().play(m).map(|new_game| {
                                game = new_game;
                            });
                        }
                        let start = game.fullmoves();
                        let sanlist = if game.turn() == Color::Black {
                            let mut sanlist = vec![String::from("…")];
                            sanlist.extend(ucimovelist_to_sanlist(game, pv));
                            sanlist
                        } else {
                            ucimovelist_to_sanlist(game, pv)
                        };
                        let moves: Vec<String> = sanlist
                            .chunks(2)
                            .enumerate()
                            .map(|(i, pair)| match (pair.get(0), pair.get(1)) {
                                (Some(a), Some(b)) => {
                                    format!("{}.{} {}", start.saturating_add(i as u32), a, b)
                                }
                                (Some(a), None) => {
                                    format!("{}.{} …", i + 1, a)
                                }
                                _ => String::from("??"),
                            })
                            .collect();
                        Some(format!("[{}]  {}", *score as f32 / 100.0, moves.join("  ")))
                    } else {
                        game_state.opening.clone().and_then(|eco| {
                            if eco.moves.len() >= game_state.moves.len() {
                                Some(eco.name.clone())
                            } else {
                                None
                            }
                        })
                    };
                    let highlight_square = if self.pointer_mode == PointerMode::Click {
                        if let Gesture::Start(StateStart { from, .. }) = *gesture {
                            Some(from)
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    render_board(
                        ctx,
                        ui,
                        &self.sources,
                        &gesture,
                        &game_state.game,
                        game_state.moves.last(),
                        title,
                        highlight_square,
                    );
                }
                {
                    let mut gesture = gesture.borrow_mut();
                    if gesture.need_promotion() {
                        render_promotion(ctx, ui, &self.sources, &mut gesture);
                        return;
                    }
                }

                let (move_, is_end) = {
                    let gesture = self.gesture.borrow();
                    let game_state = self.game.read().unwrap();
                    let turn = game_state.game.turn();

                    if let Gesture::End(state) = *gesture {
                        if turn != state.piece().color {
                            (None, false)
                        } else {
                            let moves_: Vec<Move> = game_state
                                .game
                                .legal_moves()
                                .iter()
                                .filter(|m| {
                                    ucui_utils::move_classic_to(m) == state.to()
                                        && m.from() == Some(state.from())
                                        && state.promotion().comp_move(m.promotion())
                                })
                                .cloned()
                                .collect();

                            (moves_.first().cloned(), true)
                        }
                    } else {
                        (None, false)
                    }
                };

                match (move_, is_end) {
                    (None, true) => {
                        let mut gesture = self.gesture.borrow_mut();
                        *gesture = Gesture::new();
                    }
                    (Some(move_), true) => {
                        info!("We got a move {}", move_);
                        let mut gesture = self.gesture.borrow_mut();
                        let mut game_state = self.game.write().unwrap();
                        *gesture = Gesture::new();
                        game_state.clear_score();
                        game_state.make_move(move_);
                        if self.board_mode == BoardMode::Play {
                            if let Some((move_, _)) =
                                game_state.openings.find_move(&game_state.game)
                            {
                                game_state.make_move(move_);
                            } else {
                                self.engine.play(
                                    Fen::from_position(
                                        game_state.game.clone(),
                                        shakmaty::EnPassantMode::Legal,
                                    )
                                    .to_string(),
                                );
                            }
                        }
                    }
                    _ => {}
                }

                if self.pointer_mode == PointerMode::Click {
                    ui.input(|input| {
                        if let Some(position) = input.pointer.interact_pos()
                            && input.pointer.primary_clicked()
                        {
                            let _ = gesture.try_borrow_mut().map(|mut gesture| {
                                log::info!("CLICk {:?}", gesture);
                                match *gesture {
                                    Gesture::None => {
                                        if let Some(from) = square_at(&ui.max_rect(), position) {
                                            let _ = game_state.read().map(|game_state| {
                                                game_state.game.board().piece_at(from).map(
                                                    |piece| {
                                                        info!(
                                                            "start with {:?} from {}",
                                                            &piece, &from
                                                        );
                                                        *gesture = gesture.start(from, piece);
                                                    },
                                                );
                                            });
                                        }
                                    }
                                    Gesture::Start(StateStart { from, .. }) => {
                                        square_at(&ui.max_rect(), position).map(|to| {
                                            info!("ON  {to}",);
                                            if from != to {
                                                *gesture = gesture.moving(position).end(to);
                                            } else {
                                                *gesture = Gesture::None;
                                            }
                                        });
                                    }
                                    _ => {}
                                }
                            });
                        }
                    });
                } else {
                    ui.input(|input| {
                        if let Some(position) = input.pointer.interact_pos() {
                            if input.pointer.button_pressed(egui::PointerButton::Primary) {
                                let _ = gesture.try_borrow_mut().map(|mut gesture| {
                                    log::info!("PRESS {:?}", gesture);
                                    if let Gesture::None = *gesture
                                        && let Some(from) = square_at(&ui.max_rect(), position)
                                    {
                                        let _ = game_state.read().map(|game_state| {
                                            game_state.game.board().piece_at(from).map(|piece| {
                                                info!("start with {:?} from {}", &piece, &from);
                                                *gesture = gesture.start(from, piece);
                                            });
                                        });
                                    }
                                });
                            } else if input.pointer.button_down(egui::PointerButton::Primary) {
                                let _ = gesture.try_borrow_mut().map(|mut gesture| {
                                    *gesture = gesture.moving(position);
                                });
                            } else if input.pointer.button_released(egui::PointerButton::Primary) {
                                log::info!("RELEASE");
                                if let Some(to) = square_at(&ui.max_rect(), position) {
                                    let _ = gesture.try_borrow_mut().map(|mut gesture| {
                                        info!("end to {}", &to);
                                        *gesture = gesture.end(to);
                                    });
                                }
                            }
                        }
                    });
                }

                // if processed inside ui.input, it deadlocks on rwlock<context> acquisisition
                let mut viewport_commands: Vec<egui::ViewportCommand> = Vec::new();
                let vcr = &mut viewport_commands;

                ui.input(|input| {
                    if input.key_released(Key::F) {
                        let current = egui::ViewportInfo::default().fullscreen.unwrap_or(false);
                        // ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!current));
                        vcr.push(egui::ViewportCommand::Fullscreen(!current));
                    }

                    if input.key_released(Key::Q) {
                        // ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        vcr.push(egui::ViewportCommand::Close);
                    }

                    if input.key_released(Key::N) {
                        self.new_game();
                    }

                    if input.key_released(Key::S) {
                        self.set_board_mode(BoardMode::Setup);
                    }

                    if input.key_released(Key::P) {
                        self.set_board_mode(BoardMode::Play);
                    }

                    if input.key_released(Key::I) {
                        self.toggle_pointer_mode();
                    }
                });

                for command in viewport_commands {
                    ctx.send_viewport_cmd(command);
                }
            });
    }
}

// impl DiagramApp {
//     fn process_events_board(
//         &mut self,
//         ctx: &egui::Context,
//         frame: &mut eframe::Frame,
//     ) {
//         let gesture = self.gesture;
//         ctx.input_for(board_id, |input| {
//             let pointer = &input.pointer;
//         })
//     }
// }

// fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
//     ui.horizontal(|ui| {
//         ui.spacing_mut().item_spacing.x = 0.0;
//         ui.label("Powered by ");
//         ui.hyperlink_to("egui", "https://github.com/emilk/egui");
//         ui.label(" and ");
//         ui.hyperlink_to(
//             "eframe",
//             "https://github.com/emilk/egui/tree/master/crates/eframe",
//         );
//         ui.label(".");
//     });
// }
