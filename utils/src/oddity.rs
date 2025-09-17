use shakmaty::{File, Move, Square};

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
