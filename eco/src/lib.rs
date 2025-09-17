use serde::{Deserialize, Serialize};
use shakmaty::Move;
use std::cmp;
use std::collections::HashMap;
use std::sync::OnceLock;
use ucui_utils::MoveSerde;

#[derive(Serialize, Deserialize, Clone)]
pub struct Eco {
    pub code: String,
    pub name: String,
    pub fen: String,
    pub moves: Vec<MoveSerde>,
    pub pgn: String,
}

const MAX_MOVES: usize = 36;

static ECO_JSON: &str = include_str!("../eco-table.json");

static ECO_TABLE: OnceLock<HashMap<String, Eco>> = OnceLock::new();

fn init_table() -> HashMap<String, Eco> {
    serde_json::from_str::<HashMap<String, Eco>>(ECO_JSON)
        .expect("Getting eco data should go smoothly")
}

pub fn find_eco_from_moves(mlist: &[Move]) -> Option<&Eco> {
    let slen = cmp::min(MAX_MOVES, mlist.len());
    let range = 0..=slen;
    let ucis: Vec<String> = mlist
        .iter()
        .map(|m| format!("{}", m.to_uci(shakmaty::CastlingMode::Standard)))
        .collect();

    let table = ECO_TABLE.get_or_init(init_table);
    // make keys from longest to shortest
    let keys = range
        .rev()
        .map(|len| {
            (0..len)
                .map(|i| ucis.get(i).unwrap_or(&"----".to_string()).clone())
                .collect::<Vec<_>>()
                .join("")
        })
        .collect::<Vec<_>>();

    for key in keys {
        if let Some(eco) = table.get(&key) {
            return Some(eco);
        }
    }
    None
}

pub fn lookup_eco_from_name(pat: &str) -> Vec<Eco> {
    let table = ECO_TABLE.get_or_init(init_table);
    let pat_list: Vec<String> = pat
        .to_lowercase()
        .split(" ")
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .collect();
    table
        .values()
        // .filter(|eco| eco.name.to_lowercase().contains(&lower_pat))
        .filter(|eco| {
            let lowered = eco.name.to_lowercase();
            pat_list.iter().all(|pat| lowered.contains(pat))
        })
        .cloned()
        .collect()
}

pub fn lookup_eco_from_code(pat: &str) -> Vec<Eco> {
    let table = ECO_TABLE.get_or_init(init_table);
    let pat_list: Vec<String> = pat
        .to_lowercase()
        .split(" ")
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .collect();
    table
        .values()
        // .filter(|eco| eco.name.to_lowercase().contains(&lower_pat))
        .filter(|eco| {
            let lowered = eco.code.to_lowercase();
            pat_list.iter().all(|pat| lowered.contains(pat))
        })
        .cloned()
        .collect()
}
