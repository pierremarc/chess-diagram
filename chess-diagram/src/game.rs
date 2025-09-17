use std::{collections::HashMap, str::FromStr};

use rand::seq::IndexedRandom;
use shakmaty::{Chess, Color, FromSetup, Move, Position, fen::Fen};
use ucui_eco::{
    find_eco_from_moves, get_openings_table, lookup_eco_from_code, lookup_eco_from_name,
};

use crate::config::{get_eco_codes, get_opening};

pub struct GameState {
    pub game: Chess,
    pub moves: Vec<Move>,
    pub engine_color: Color,
    pub openings: Openings,
    pub opening: Option<String>,
}

impl GameState {
    pub fn new(color: Color, position: Option<String>) -> Self {
        Self {
            engine_color: color,
            moves: Vec::new(),
            game: position
                .and_then(|fen_string| Fen::from_str(&fen_string).ok())
                .and_then(|fen| {
                    Chess::from_setup(fen.into_setup(), shakmaty::CastlingMode::Standard).ok()
                })
                .unwrap_or_default(),
            openings: Openings::new(),
            opening: None,
        }
    }

    pub fn make_move(&mut self, move_: Move) {
        if let Ok(new_game) = self.game.clone().play(&move_) {
            self.moves.push(move_.clone());
            self.opening = find_eco_from_moves(&self.moves).map(|eco| eco.name.clone());
            self.game = new_game;
        };
    }
}

pub type OpeningItem = (Vec<Move>, String);
pub struct Openings {
    // variants: Vec<Eco>,
    index: HashMap<Fen, OpeningItem>,
}

impl Openings {
    fn new() -> Self {
        let variants = if let Some(opening) = get_opening() {
            lookup_eco_from_name(&opening)
        } else if get_eco_codes().len() > 0 {
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

        Self { index }
    }

    pub fn find_move(&self, game: &Chess) -> Option<(Move, String)> {
        let fen = Fen::from_position(game.clone(), shakmaty::EnPassantMode::Legal);
        self.index.get(&fen).and_then(|(moves, name)| {
            moves
                .choose(&mut rand::rng())
                .map(|move_| (move_.clone(), name.clone()))
        })
    }
}
