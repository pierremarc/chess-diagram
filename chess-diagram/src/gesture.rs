use egui::Pos2;
use log::info;
use shakmaty::{Piece, Square};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct StateStart {
    // position: Pos2,
    from: Square,
    piece: Piece,
    // to: Option<Square>,
}

impl StateStart {
    pub fn new(from: Square, piece: Piece) -> Self {
        Self { from, piece }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct StateMoving {
    position: Pos2,
    from: Square,
    piece: Piece,
    // to: Option<Square>,
}

impl StateMoving {
    pub fn from_start(position: Pos2, StateStart { from, piece }: StateStart) -> Self {
        Self {
            position,
            from,
            piece,
        }
    }

    pub fn new_pos(mut self, position: Pos2) -> Self {
        self.position = position;
        self
    }

    pub fn position(&self) -> Pos2 {
        self.position
    }

    pub fn from(&self) -> Square {
        self.from
    }

    pub fn piece(&self) -> Piece {
        self.piece
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct StateEnd {
    // position: Pos2,
    from: Square,
    piece: Piece,
    to: Square,
}

impl StateEnd {
    pub fn from_moving(
        to: Square,
        StateMoving {
            position: _,
            from,
            piece,
        }: StateMoving,
    ) -> Self {
        Self { from, piece, to }
    }

    pub fn piece(&self) -> Piece {
        self.piece
    }
    pub fn from(&self) -> Square {
        self.from
    }

    pub fn to(&self) -> Square {
        self.to
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Gesture {
    None,
    Start(StateStart),
    Moving(StateMoving),
    End(StateEnd),
}

impl Gesture {
    pub fn new() -> Self {
        Gesture::None
    }

    pub fn start(&self, from: Square, piece: Piece) -> Self {
        Gesture::Start(StateStart::new(from, piece))
    }

    pub fn moving(&self, position: Pos2) -> Self {
        match self {
            Gesture::Start(state) => Gesture::Moving(StateMoving::from_start(position, *state)),
            Gesture::Moving(state) => Gesture::Moving(state.new_pos(position)),
            _ => *self,
        }
    }

    pub fn end(&self, to: Square) -> Self {
        info!("Gesture#end {self:?} {to:?}");
        match self {
            Gesture::Moving(state) => Gesture::End(StateEnd::from_moving(to, *state)),
            _ => *self,
        }
    }
}
