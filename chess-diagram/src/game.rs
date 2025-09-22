use std::collections::HashMap;

use rand::seq::IndexedRandom;
use shakmaty::{Chess, Color, Move, Position, fen::Fen};
use ucui_eco::{
    Eco, find_eco_from_moves, get_openings_table, lookup_eco_from_code, lookup_eco_from_name,
};
use ucui_engine::Score;

use crate::{
    config::{get_eco_codes, get_opening},
    variation::{MoveIndex, VariationTree},
};

pub struct GameState {
    // pub game: Chess,
    pub tree: VariationTree,
    pub engine_color: Color,
    pub openings: Openings,
    pub opening: Option<Eco>,
    pub score: Score,
}

impl GameState {
    pub fn new(color: Color, _position: Option<String>) -> Self {
        Self {
            engine_color: color,
            tree: VariationTree::new(),
            // game: position
            //     .and_then(|fen_string| Fen::from_str(&fen_string).ok())
            //     .and_then(|fen| {
            //         Chess::from_setup(fen.into_setup(), shakmaty::CastlingMode::Standard).ok()
            //     })
            //     .unwrap_or_default(),
            openings: Openings::new(),
            opening: None,
            score: Score::None,
        }
    }

    pub fn game(&self) -> Chess {
        self.tree.game()
    }

    pub fn make_move(&mut self, move_: Move) {
        // if let Ok(new_game) = self.game.clone().play(&move_) {
        self.tree.push_move(move_);
        self.opening = find_eco_from_moves(&self.tree.moves()).cloned();
        // self.game = new_game;
        // };
    }

    pub fn at(&mut self, at: MoveIndex) {
        log::info!("Game#at {:?}", at);
        self.tree.set_current(at);
        // self.game = self.tree.game().unwrap();
    }

    pub fn clear_score(&mut self) {
        self.score = Score::None;
    }

    pub fn set_score(&mut self, score: Score) {
        self.score = score;
    }
}

pub type OpeningItem = (Vec<Move>, String);
pub struct Openings {
    // variants: Vec<Eco>,
    index: HashMap<Fen, OpeningItem>,
}

impl Openings {
    fn new() -> Self {
        log::info!("Init openings");
        let variants = if let Some(opening) = get_opening() {
            lookup_eco_from_name(&opening)
        } else if !get_eco_codes().is_empty() {
            let mut variants = Vec::new();
            for eco in get_eco_codes() {
                variants.extend(lookup_eco_from_code(&eco));
            }
            variants
        } else {
            // everything?
            get_openings_table()
        };

        let mut index = HashMap::<Fen, OpeningItem>::new();
        for variant in variants.iter() {
            let mut game = Chess::new();

            let moves_max = variant.moves.len() - 1;
            for mi in 0..moves_max {
                let fen = Fen::from_position(game.clone(), shakmaty::EnPassantMode::Legal);
                let move_: Move = variant.moves[mi].clone().into();
                if game.is_legal(&move_) {
                    game = game.play(&move_).expect("illegal move from openings table");
                    let movelist = if let Some((current, _)) = index.get(&fen) {
                        let mut updated = current.clone();
                        updated.push(move_);
                        updated
                    } else {
                        vec![move_]
                    };
                    let _ = index.insert(fen, (movelist, variant.name.clone()));
                }
            }
        }
        log::info!("Openings ready");

        Self { index }
    }

    pub fn find_move(&self, game: &Chess) -> Option<(Move, String)> {
        let fen = Fen::from_position(game.clone(), shakmaty::EnPassantMode::Legal);
        log::info!("find_move {fen}");
        self.index.get(&fen).and_then(|(moves, name)| {
            moves
                .choose(&mut rand::rng())
                .map(|move_| (move_.clone(), name.clone()))
        })
    }
}
