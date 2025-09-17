use std::{
    sync::{
        Arc, Mutex, RwLock,
        mpsc::{Sender, channel},
    },
    thread::spawn,
};

use chrono::Duration;
use egui::Context;
use shakmaty::{Move, Position};
use ucui_engine::{EngineCommand, EngineMessage, connect_engine};

use crate::{
    config::{get_engine, get_engine_args, get_engine_options},
    game::GameState,
};

pub struct Proxy {
    tx: Sender<EngineCommand>,
}

impl Proxy {
    fn new(tx: Sender<EngineCommand>) -> Self {
        Proxy { tx }
    }

    // fn name(&self) -> String {
    //     self.inner.name()
    // }

    pub fn new_game(&self) {
        self.tx
            .send(EngineCommand::NewGame)
            .expect("Err proxy new_game ");
    }

    pub fn stop(&self) {
        self.tx.send(EngineCommand::Stop).expect("Err proxy stop ");
    }

    pub fn play(&self, fen: String, white_time: Duration, black_time: Duration) {
        self.tx
            .send(EngineCommand::Go {
                fen,
                white_time,
                black_time,
            })
            .expect("Err proxy play ");
    }
}

pub fn start_engine(state: Arc<RwLock<GameState>>, ctx: Arc<Mutex<Context>>) -> Proxy {
    let (tx, rx) = channel::<EngineCommand>();
    let _ = spawn(move || {
        let engine = connect_engine(&get_engine(), get_engine_args(), get_engine_options());
        loop {
            if let Ok(command) = rx.recv() {
                match command {
                    EngineCommand::NewGame => engine.new_game(),
                    EngineCommand::Stop => engine.stop(),
                    EngineCommand::Go {
                        fen,
                        white_time,
                        black_time,
                    } => {
                        engine.go(fen, white_time, black_time);

                        if let Ok(EngineMessage::BestMove { move_, score: _ }) = engine.recv() {
                            let move_: Move = move_.into();
                            log::info!("Engine played {move_}");
                            let mut state =
                                state.write().expect("failed to get a writable game state");
                            state.make_move(move_);

                            if let Ok(ctx) = ctx.lock() {
                                ctx.request_repaint();
                            }
                        }
                    }
                }
            }
        }
    });

    Proxy::new(tx)
}
