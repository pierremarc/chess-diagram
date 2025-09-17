use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::{Arc, Mutex, RwLock};

use chrono::Duration;
use egui::Key;
use egui_extras::install_image_loaders;
use log::info;
use shakmaty::fen::Fen;
use shakmaty::{Chess, Move, Position};
use shakmaty_uci::{ParseUciMoveError, UciMove};

use crate::board::{render_board, square_at};
use crate::config::get_engine_color;
use crate::game::GameState;
use crate::gesture::Gesture;
use crate::promotion::render_promotion;
use crate::proxy::{Proxy, start_engine};
use crate::side::render_side;
use crate::sources::Sources;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Mode {
    Play,
    Setup,
}

pub struct DiagramApp<'a> {
    gesture: Rc<RefCell<Gesture>>,
    game: Arc<RwLock<GameState>>,
    engine: Arc<Proxy>,
    sources: Sources<'a>,
    mode: Mode,
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
            mode: Mode::Play,
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

    fn set_mode(&mut self, mode: Mode) {
        use Mode::*;
        match mode {
            Setup => self.mode = Setup,
            Play => {
                self.mode = Play;
                if let Ok(game_state) = self.game.read() {
                    self.engine.play(
                        Fen::from_position(game_state.game.clone(), shakmaty::EnPassantMode::Legal)
                            .to_string(),
                        Duration::milliseconds(3000),
                        Duration::milliseconds(3000),
                    );
                }
            }
        }
    }
}

impl<'a> eframe::App for DiagramApp<'a> {
    /// Called by the framework to save state before shutdown.
    // fn save(&mut self, storage: &mut dyn eframe::Storage) {
    //     eframe::set_value(storage, eframe::APP_KEY, self);
    // }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // let gesture = &mut self.gesture;
        let gesture = self.gesture.clone();
        let game_state = self.game.clone();

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
                    render_board(
                        ctx,
                        ui,
                        &self.sources,
                        &gesture,
                        &game_state.game,
                        game_state.moves.last(),
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
                                .map(|m| m.clone())
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
                        game_state.make_move(move_);
                        if self.mode == Mode::Play {
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
                                    Duration::milliseconds(3000),
                                    Duration::milliseconds(3000),
                                );
                            }
                        }
                    }
                    _ => {}
                }

                ui.input(|input| {
                    if let Some(position) = input.pointer.interact_pos() {
                        if input.pointer.button_pressed(egui::PointerButton::Primary) {
                            square_at(&ui.max_rect(), position).map(|from| {
                                let _ = game_state.read().map(|game_state| {
                                    game_state.game.board().piece_at(from).map(|piece| {
                                        let _ = gesture.try_borrow_mut().map(|mut gesture| {
                                            info!("start with {:?} from {}", &piece, &from);
                                            *gesture = gesture.start(from, piece);
                                        });
                                    });
                                });
                            });
                        } else if input.pointer.button_down(egui::PointerButton::Primary) {
                            let _ = gesture.try_borrow_mut().map(|mut gesture| {
                                *gesture = gesture.moving(position);
                            });
                        } else if input.pointer.button_released(egui::PointerButton::Primary) {
                            square_at(&ui.max_rect(), position).map(|to| {
                                let _ = gesture.try_borrow_mut().map(|mut gesture| {
                                    info!("end to {}", &to);
                                    *gesture = gesture.end(to);
                                });
                            });
                        }
                    }
                });

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
                        self.set_mode(Mode::Setup);
                    }

                    if input.key_released(Key::P) {
                        self.set_mode(Mode::Play);
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
