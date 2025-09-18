use shakmaty::{Chess, File, Move, Position, Square, san::San, uci::UciMove};

/// Sadly, `shakmaty` uses Chess960 logic for its casling moves, it breaks.
/// So we provide a little one to right their wrong.
/// https://github.com/niklasf/shakmaty/issues/13
pub fn move_classic_to(move_: &Move) -> Square {
    match move_ {
        Move::Castle { king, rook } => match (king.file(), rook.file()) {
            (File::E, File::H) => Square::from_coords(File::G, king.rank()),
            (File::E, File::A) => Square::from_coords(File::C, king.rank()),
            _ => move_.to(),
        },
        _ => move_.to(),
    }
}

pub fn ucimovelist_to_sanlist(mut game: Chess, movelist: &Vec<UciMove>) -> Vec<String> {
    let mut result = Vec::with_capacity(movelist.len());
    for uci_move in movelist {
        if let Ok(move_) = uci_move.to_move(&game) {
            result.push(San::from_move(&game, &move_).to_string());
            game = game.clone().play(&move_).expect("Illegal move?");
        } else {
            return result;
        }
    }
    result
}
