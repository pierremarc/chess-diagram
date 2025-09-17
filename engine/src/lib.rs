use std::sync::mpsc::RecvError;

use serde::{Deserialize, Serialize};
use shakmaty::Move;
use shakmaty_uci::UciInfo;
mod uci;

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub enum EngineState {
    #[default]
    Idle,
    Computing,
    PendingMove(Move),
    Move(Move),
}

// #[derive(Serialize, Deserialize)]
// pub enum EngineGoArg {
//     MoveTime(Duration),
//     LeftTime(Duration),
//     Depth(usize),
// }

#[derive(Serialize, Deserialize)]
#[serde(tag = "_tag")]
pub enum EngineCommand {
    Go { fen: String, depth: u8 },
    NewGame,
    Stop,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "_tag")]
pub enum Score {
    CentiPawns { score: i32 },
    Mate { moves: i8 },
    None,
}

impl From<UciInfo> for Score {
    fn from(value: UciInfo) -> Self {
        if let UciInfo {
            score: Some(score), ..
        } = value
        {
            match (score.cp, score.mate) {
                (None, None) => Score::None,
                (Some(score), None) => Score::CentiPawns { score },
                (_, Some(moves)) => Score::Mate { moves },
            }
        } else {
            Score::None
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "_tag")]
pub enum EngineMessage {
    Id(String),
    BestMove {
        move_: ucui_utils::MoveSerde,
        score: Score,
    },
}

pub trait Engine {
    fn name(&self) -> String;
    fn new_game(&self) {}
    fn stop(&self) {}
    fn go(&self, fen: String, depth: u8);
    fn recv(&self) -> Result<EngineMessage, RecvError>;
}

pub fn connect_engine(
    engine_path: &str,
    args: Option<Vec<String>>,
    options: Vec<(String, Option<String>)>,
) -> Box<dyn Engine + Send> {
    Box::new(uci::connect_engine(engine_path, args, options))
}
