use std::{
    str::FromStr,
    sync::mpsc::{channel, Receiver, RecvError, Sender},
    thread,
};

use chrono::Duration;
use shakmaty::{fen::Fen, Chess, Color, FromSetup, Position};
use shakmaty_uci::{UciInfo, UciInfoScore, UciMessage, UciMove};

use crate::Score;

use super::{Engine, EngineCommand, EngineMessage};

struct UciEngine {
    rx: Receiver<EngineCommand>,
    tx: Sender<EngineMessage>,

    engine: uci::Engine,
    options: Vec<(String, Option<String>)>,
}

impl UciEngine {
    fn new(
        path: &str,
        rx: Receiver<EngineCommand>,
        tx: Sender<EngineMessage>,
        args: Option<Vec<String>>,
        options: Vec<(String, Option<String>)>,
    ) -> Self {
        let engine = match args {
            None => uci::Engine::new(path).expect("engine should be OK"),
            Some(args) => uci::Engine::with_args(path, args).expect("engine should be OK"),
        };
        UciEngine {
            rx,
            tx,
            engine,
            options,
        }
    }

    fn send_id(&self) {
        let name = self
            .engine
            .command_and_wait_for("uci", "id name")
            .map(|lines| {
                for line in lines.split("\n") {
                    if let Ok(UciMessage::Id { name, .. }) = UciMessage::from_str(line) {
                        if let Some(name) = name {
                            return name;
                        }
                    } else {
                        log::debug!("<engine> {line}");
                    }
                }
                "UCI Engine".to_string()
            })
            .unwrap_or("UCI Engine".to_string());

        let _ = self.tx.send(EngineMessage::Id(name));
    }

    fn set_options(&self) {
        for (id, value) in self.options.iter() {
            let _ = self
                .engine
                .set_option(id, &value.clone().unwrap_or(String::new()));
        }
    }

    fn start(&self) {
        self.set_options();

        loop {
            match self.rx.recv() {
                Err(err) => {
                    log::error!("Engine channel error: {}", err);
                    break;
                }
                Ok(msg) => match msg {
                    EngineCommand::NewGame => self.new_game(),
                    EngineCommand::Go {
                        fen,
                        white_time,
                        black_time,
                    } => self.go(fen, white_time, black_time),
                    EngineCommand::Stop => break,
                },
            }
        }
    }

    pub fn update_move(&self, best_move_uci: UciMove, game: Chess, score: Score) {
        match best_move_uci.to_move(&game) {
            Err(e) => log::error!(
                "<uci-engine> Failed to produce a bestmove from {best_move_uci}: {} ",
                e,
            ),
            Ok(m) => {
                let _ = self.tx.send(EngineMessage::BestMove {
                    move_: m.into(),
                    score,
                });
            }
        }
    }

    fn new_game(&self) {
        let _ = self
            .engine
            .command_with_duration("ucinewgame", std::time::Duration::from_millis(100));
    }

    fn go(&self, fen_string: String, white_time: Duration, black_time: Duration) {
        if let Ok(fen) = Fen::from_str(&fen_string) {
            let setpos = shakmaty_uci::UciMessage::Position {
                startpos: false,
                fen: Some(fen.clone()),
                moves: Vec::new(),
            };
            let goc = shakmaty_uci::UciMessage::Go {
                time_control: Some(shakmaty_uci::UciTimeControl::TimeLeft {
                    white_time: Some(white_time.to_std().expect("positive duration")),
                    black_time: Some(black_time.to_std().expect("positive duration")),
                    white_increment: None,
                    black_increment: None,
                    moves_to_go: None,
                }),
                search_control: None,
            };
            if self.engine.command(&setpos.to_string()).is_ok() {
                let _ = self
                    .engine
                    .command_and_wait_for(&goc.to_string(), "bestmove")
                    .map(|lines| {
                        let mut infos: Vec<UciInfo> = Vec::new();
                        for line in lines.split("\n") {
                            if let Ok(UciMessage::BestMove { best_move, .. }) =
                                UciMessage::from_str(line)
                            {
                                let game = Chess::from_setup(
                                    fen.as_setup().clone(),
                                    shakmaty::CastlingMode::Standard,
                                )
                                .expect("argh!");

                                let score = get_score(&infos, game.turn(), &best_move);

                                self.update_move(best_move, game, score);
                            } else if let Ok(UciMessage::Info(info)) = UciMessage::from_str(line) {
                                infos.push(info);
                            }
                        }
                        "OK".to_string()
                    });
            };
        } else {
            log::error!("<uci-engine> failed to produce a `Fen` from fen string:  '{fen_string}'");
        }
    }
}

/// lookup a possible score in infos list
fn get_score(infos: &Vec<UciInfo>, color: Color, best_move: &UciMove) -> Score {
    let mut candidates = infos
        .iter()
        .filter(|info| {
            info.score.is_some() // info score will be unwrapable later
                && info
                    .pv
                    .first()
                    .map(|m| *m == *best_move)
                    .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|info| info.pv.len());
    let max_len = candidates
        .iter()
        .map(|info| info.pv.len())
        .max()
        .unwrap_or(0);

    let score = candidates
        .into_iter()
        .filter(|c| c.pv.len() == max_len)
        .reduce(|acc, info| {
            match comp_score(
                acc.score.clone().unwrap(),
                info.score.clone().unwrap(),
                color,
            ) {
                CompScore::Right => info,
                _ => acc,
            }
        })
        .map(|info| Score::from(info.clone()))
        .unwrap_or(Score::None);
    score
}

enum CompScore {
    Left,
    Right,
    Equal,
}

fn comp_score(a: UciInfoScore, b: UciInfoScore, color: Color) -> CompScore {
    use CompScore::*;
    let gt = if color == Color::Black {
        |a: i32, b: i32| a < b
    } else {
        |a: i32, b: i32| a > b
    };

    match (a, b) {
        (UciInfoScore { mate: Some(_), .. }, UciInfoScore { mate: None, .. }) => Left,
        (UciInfoScore { mate: None, .. }, UciInfoScore { mate: Some(_), .. }) => Right,
        (UciInfoScore { mate: Some(na), .. }, UciInfoScore { mate: Some(nb), .. }) => {
            if na > nb {
                Left
            } else if na < nb {
                Right
            } else {
                Equal
            }
        }
        (UciInfoScore { cp: Some(_), .. }, UciInfoScore { cp: None, .. }) => Left,
        (UciInfoScore { cp: None, .. }, UciInfoScore { cp: Some(_), .. }) => Right,
        (UciInfoScore { cp: Some(cpa), .. }, UciInfoScore { cp: Some(cpb), .. }) => {
            if gt(cpa, cpb) {
                Left
            } else if !gt(cpa, cpb) {
                Right
            } else {
                Equal
            }
        }
        // TODO: consider bounds
        _ => Equal,
    }
}

pub struct EngineConnection {
    tx: Sender<EngineCommand>,
    receiver: Receiver<EngineMessage>,
    engine_id: Option<String>,
}

impl EngineConnection {
    fn new(
        tx: Sender<EngineCommand>,
        rx: Receiver<EngineMessage>,
        engine_id: Option<String>,
    ) -> Self {
        Self {
            tx,
            receiver: rx,
            engine_id,
        }
    }
}

impl Engine for EngineConnection {
    fn name(&self) -> String {
        self.engine_id.clone().unwrap_or(String::from("-"))
    }

    fn new_game(&self) {
        let _ = self.tx.send(EngineCommand::NewGame);
    }

    fn stop(&self) {
        let _ = self.tx.send(EngineCommand::Stop);
    }

    fn go(&self, fen_string: String, white_time: Duration, black_time: Duration) {
        let _ = self.tx.send(EngineCommand::Go {
            fen: fen_string,
            white_time,
            black_time,
        });
    }

    fn recv(&self) -> Result<EngineMessage, RecvError> {
        self.receiver.recv()
    }
}

pub fn connect_engine(
    path: &str,
    args: Option<Vec<String>>,
    options: Vec<(String, Option<String>)>,
) -> EngineConnection {
    let (sender_to, receiver_to) = channel::<EngineCommand>();
    let (sender_from, receiver_from) = channel::<EngineMessage>();
    let cloned_path = String::from(path);
    thread::spawn(move || {
        let engine = UciEngine::new(&cloned_path, receiver_to, sender_from, args, options);
        engine.send_id();
        engine.start();
    });

    let id = receiver_from
        .recv()
        .map(|msg| {
            if let EngineMessage::Id(id) = msg {
                id
            } else {
                String::from("NN")
            }
        })
        .ok();

    EngineConnection::new(sender_to, receiver_from, id)
}
