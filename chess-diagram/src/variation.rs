use std::cmp::Ordering;

use shakmaty::{Chess, Move, Position};
use shakmaty_uci::UciMove;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoveIndex {
    var_index: usize,
    move_index: usize,
}

impl MoveIndex {
    fn root() -> Self {
        MoveIndex {
            var_index: 0,
            move_index: 0,
        }
    }

    pub fn incr_variation(&self) -> Self {
        MoveIndex {
            var_index: self.var_index + 1,
            move_index: self.move_index,
        }
    }

    pub fn incr_move(&self) -> Self {
        MoveIndex {
            var_index: self.var_index,
            move_index: self.move_index + 1,
        }
    }

    pub fn move_index(&self) -> usize {
        self.move_index
    }

    pub fn var_index(&self) -> usize {
        self.var_index
    }
}

impl Ord for MoveIndex {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.var_index == other.var_index {
            self.move_index.cmp(&other.move_index)
        } else {
            self.var_index.cmp(&other.var_index)
        }
    }
}

impl PartialOrd for MoveIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
pub struct Variation {
    pub from: Option<MoveIndex>,
    pub index: MoveIndex,
    pub moves: Vec<Move>,
    position: Chess,
}

impl Variation {
    fn new(from: MoveIndex, var_index: usize, move_: Move, starting_position: Chess) -> Self {
        Variation {
            from: Some(from),
            index: MoveIndex {
                var_index,
                move_index: 0,
            },
            moves: vec![move_],
            position: starting_position,
        }
    }

    pub fn is_root(&self) -> bool {
        self.from.is_none()
    }

    pub fn starting_position(&self) -> Chess {
        self.position.clone()
    }

    pub fn game(&self, move_index: usize) -> Chess {
        let mut source_game = self.position.clone();
        let game = &mut source_game;
        for index in 0..=move_index {
            if let Some(move_) = self.moves.get(index) {
                if let Ok(new_game) = game.clone().play(move_) {
                    *game = new_game;
                }
            }
        }
        source_game
    }
}

#[derive(Debug)]
pub struct VariationTree {
    variations: Vec<Variation>,
    cursor: Option<MoveIndex>,
}

impl VariationTree {
    pub fn new() -> Self {
        VariationTree {
            variations: Vec::new(),
            cursor: None,
        }
    }

    pub fn current(&self) -> Option<MoveIndex> {
        self.cursor
    }

    pub fn is_current(&self, index: MoveIndex) -> bool {
        self.cursor.map(|c| c == index).unwrap_or(false)
    }

    pub fn is_current_variation(&self, index: MoveIndex) -> bool {
        self.cursor
            .map(|c| c.var_index == index.var_index)
            .unwrap_or(false)
    }

    pub fn set_current(&mut self, index: MoveIndex) {
        self.cursor = Some(index);
    }

    pub fn game(&self) -> Chess {
        self.cursor
            .and_then(|cursor| self.game_at(cursor))
            .unwrap_or(Chess::new())
    }

    pub fn game_at(
        &self,
        MoveIndex {
            move_index,
            var_index,
        }: MoveIndex,
    ) -> Option<Chess> {
        self.variations.get(var_index).map(|variation| {
            let mut source_game = variation.position.clone();
            let game = &mut source_game;
            for index in 0..=move_index {
                if let Some(move_) = variation.moves.get(index) {
                    if let Ok(new_game) = game.clone().play(move_) {
                        *game = new_game;
                    }
                }
            }
            source_game
        })
    }

    pub fn start_variation(&mut self, start: MoveIndex, move_: Move) -> Option<MoveIndex> {
        self.game_at(start).map(|game| {
            let variation = Variation::new(start, self.variations.len(), move_, game);
            let new_index = variation.index;
            self.variations.push(variation);
            self.cursor = Some(new_index);
            new_index
        })
    }
    /// Push a move to the tip of the current variation
    /// Or just follow the variation
    /// Or create a new branch
    /// Or create the main branch
    pub fn push_move(&mut self, move_: Move) {
        log::info!("push_move {move_}");
        if let Some(index) = self.current() {
            let variation = &mut self.variations[index.var_index];
            if index.move_index + 1 == variation.moves.len() {
                log::info!("On tip of {:?}", index.var_index);
                variation.moves.push(move_);
                self.set_current(index.incr_move());
            } else {
                if let Some(movat) = variation.moves.get(index.move_index + 1) {
                    if move_ == *movat {
                        log::info!("Follow line {:?}", index.var_index);
                        self.set_current(index.incr_move());
                    } else {
                        if let Some(new_index) = self.start_variation(index, move_) {
                            log::info!("Started variation{new_index:?}");
                            self.set_current(new_index);
                        }
                    }
                }
            }
        } else {
            log::info!("Create main variation");

            self.variations.push(Variation {
                from: None,
                index: MoveIndex::root(),
                position: Chess::new(),
                moves: vec![move_],
            });
            self.cursor = Some(MoveIndex::root());
        }
    }

    pub fn push_uci_move(&mut self, uci: &str) -> Result<(), shakmaty_uci::ParseUciMoveError> {
        UciMove::from_ascii(uci.as_bytes()).map(|uci_move| {
            if let Some(index) = self.current() {
                if let Some(game) = self.game_at(index) {
                    if let Ok(move_) = uci_move.to_move(&game) {
                        let variation = &mut self.variations[index.var_index];
                        if index.move_index + 1 == variation.moves.len() {
                            log::info!("On tip of {:?}", index.var_index);
                            variation.moves.push(move_);
                            self.set_current(index.incr_move());
                        } else {
                            if let Some(movat) = variation.moves.get(index.move_index + 1) {
                                if move_ == *movat {
                                    log::info!("Follow line {:?}", index.var_index);
                                    self.set_current(index.incr_move());
                                } else {
                                    if let Some(new_index) = self.start_variation(index, move_) {
                                        log::info!("Started variation{new_index:?}");
                                        self.set_current(new_index);
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                log::info!("Create main variation");
                let _ = uci_move.to_move(&Chess::new()).map(|move_| {
                    self.variations.push(Variation {
                        from: None,
                        index: MoveIndex::root(),
                        position: Chess::new(),
                        moves: vec![move_],
                    });
                    self.cursor = Some(MoveIndex::root());
                });
            }
        })
    }

    pub fn moves(&self) -> Vec<Move> {
        self.current()
            .map(
                |MoveIndex {
                     var_index,
                     move_index,
                 }| {
                    log::info!("moves {var_index} {move_index}");

                    let mut variation = &self.variations[var_index];
                    let mut chunks = vec![&variation.moves[0..move_index]];
                    let mut it = 0;
                    loop {
                        assert!(it < 1200, "looping a tad too much, me think");

                        if let Some(from) = variation.from {
                            log::info!("from {}", from.var_index);
                            variation = &self.variations[from.var_index];
                            chunks.push(&variation.moves);
                        } else {
                            break;
                        }
                        it += 1;
                    }
                    chunks
                        .into_iter()
                        .rev()
                        .flatten()
                        .map(|m| m.clone())
                        .collect()
                },
            )
            .unwrap_or(Vec::new())
    }

    fn next(
        &self,
        MoveIndex {
            move_index,
            var_index,
        }: MoveIndex,
    ) -> Option<MoveIndex> {
        let mut candidate_variation_index = var_index;
        // let sorted = self.sorted_variations(); // ?
        let sorted = &self.variations;
        loop {
            // log::info!("loop next");
            if candidate_variation_index >= sorted.len() {
                return None;
            } else {
                let variation = sorted.get(var_index).unwrap();
                if move_index + 1 >= variation.moves.len() {
                    candidate_variation_index += 1;
                    continue;
                } else {
                    return Some(MoveIndex {
                        move_index: move_index + 1,
                        var_index: candidate_variation_index,
                    });
                }
            }
        }
    }

    pub fn move_at(
        &self,
        MoveIndex {
            move_index,
            var_index,
        }: MoveIndex,
    ) -> Option<&'_ Move> {
        self.variations
            .get(var_index)
            .and_then(|variation| variation.moves.get(move_index))
    }

    pub fn iter(&self) -> VariationTreeIterator<'_> {
        VariationTreeIterator {
            root: self,
            cur_index: Some(MoveIndex::root()),
        }
    }

    pub fn root_index(&self) -> Option<MoveIndex> {
        if self.cursor.is_some() {
            Some(MoveIndex::root())
        } else {
            None
        }
    }

    pub fn root_variation(&self) -> Option<Variation> {
        // log::info!("root_variation {:?}", self.root_index());
        self.root_index()
            .and_then(|_| self.variations.first().map(|v| v.clone()))
    }

    pub fn variations_from(&self, index: MoveIndex) -> Vec<Variation> {
        self.variations
            .iter()
            .filter_map(|v| {
                v.from
                    .and_then(|from| if from == index { Some(v.clone()) } else { None })
            })
            .collect()
    }

    fn sorted_variations(&self) -> Vec<Variation> {
        // TODO
        if self.variations.is_empty() {
            Vec::new()
        } else {
            let without_root = self
                .variations
                .iter()
                .filter(|v| !v.is_root())
                .collect::<Vec<_>>();

            if without_root.is_empty() {
                self.variations.first().map(|v| vec![v.clone()]).unwrap()
            } else {
                let mut vars = self.variations.clone();
                vars.sort_by_key(|v| v.index);
                vars
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariationTreeItem {
    pub index: MoveIndex,
    pub move_: Move,
}

pub struct VariationTreeIterator<'a> {
    root: &'a VariationTree,
    cur_index: Option<MoveIndex>,
}

impl<'a> Iterator for VariationTreeIterator<'a> {
    type Item = VariationTreeItem;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.cur_index {
            if let Some(m) = self.root.move_at(current) {
                self.cur_index = self.root.next(current);
                Some(VariationTreeItem {
                    index: current,
                    move_: m.clone(),
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use std::num::NonZero;

    use super::*;

    #[test]
    fn move_at() {
        let mut tree = VariationTree::new();
        let move_ = Move::Normal {
            role: shakmaty::Role::Pawn,
            from: shakmaty::Square::E2,
            capture: None,
            to: shakmaty::Square::E4,
            promotion: None,
        };
        tree.push_move(move_.clone());
        let pos = tree.current().unwrap();
        let m = tree.move_at(pos).unwrap();
        assert_eq!(*m, move_);
    }

    #[test]
    fn push_on_main_variation() {
        let mut tree = VariationTree::new();
        let _ = tree.push_uci_move("e2e4").unwrap();
        let _ = tree.push_uci_move("e7e5").unwrap();
        let pos = tree.current().unwrap();
        let game = tree.game_at(pos).unwrap();

        assert_eq!(Some(game.fullmoves()), NonZero::new(2));
    }

    #[test]
    fn push_on_new_variation() {
        let mut tree = VariationTree::new();
        let _ = tree.push_uci_move("e2e4").unwrap();
        let _ = tree.push_uci_move("e7e5").unwrap();
        tree.set_current(MoveIndex::root());
        let _ = tree.push_uci_move("c7c5").unwrap();
        let _ = tree.push_uci_move("g1f3").unwrap();
        let expected_current = MoveIndex::root().incr_variation().incr_move();

        assert_eq!(tree.current(), Some(expected_current));
    }

    #[test]
    fn walk_variations() {
        fn govar(tree: &VariationTree, var: Variation, path: &mut Vec<MoveIndex>) {
            let mut index = var.index;
            for _ in var.moves.iter() {
                path.push(index);

                for sub in tree.variations_from(index) {
                    govar(tree, sub, path);
                }
                index = index.incr_move();
            }
        }

        let mut tree = VariationTree::new();
        let _ = tree.push_uci_move("e2e4").unwrap();
        let _ = tree.push_uci_move("e7e5").unwrap();
        tree.set_current(MoveIndex::root());
        let _ = tree.push_uci_move("c7c5").unwrap();
        let _ = tree.push_uci_move("g1f3").unwrap();

        let mut path: Vec<MoveIndex> = Vec::new();
        govar(&tree, tree.root_variation().unwrap(), &mut path);
        let expected = vec![
            MoveIndex {
                var_index: 0,
                move_index: 0,
            },
            MoveIndex {
                var_index: 1,
                move_index: 0,
            },
            MoveIndex {
                var_index: 1,
                move_index: 1,
            },
            MoveIndex {
                var_index: 0,
                move_index: 1,
            },
        ];

        assert_eq!(expected, path);
    }
}
