use egui::Pos2;
use log::info;
use shakmaty::{Piece, Rank, Role, Square};

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
pub enum Promotion {
    NotApplicable,
    None,
    Role(Role),
}

impl Promotion {
    pub fn comp_move(&self, opt_role: Option<Role>) -> bool {
        match (self, opt_role) {
            (Promotion::NotApplicable, _) => true,
            (Promotion::None, None) => true,
            (Promotion::None, Some(_)) => false,
            (Promotion::Role(_), None) => false,
            (Promotion::Role(self_role), Some(role)) => *self_role == role,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct StateEnd {
    // position: Pos2,
    from: Square,
    piece: Piece,
    to: Square,
    promotion: Promotion,
}

impl StateEnd {
    pub fn from_moving(
        to: Square,
        StateMoving {
            position: _,
            from,
            piece,
        }: StateMoving,
        promotion: Promotion,
    ) -> Self {
        Self {
            from,
            piece,
            to,
            promotion,
        }
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

    pub fn promotion(&self) -> Promotion {
        self.promotion
    }

    pub fn promote(&mut self, role: Role) {
        self.promotion = Promotion::Role(role);
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
        info!("Gesture#end {:?} {}", self, to);
        match self {
            Gesture::Moving(state) => {
                if state.piece().role == Role::Pawn
                    && (to.rank() == Rank::First || to.rank() == Rank::Eighth)
                {
                    Gesture::End(StateEnd::from_moving(to, *state, Promotion::None))
                } else {
                    Gesture::End(StateEnd::from_moving(to, *state, Promotion::NotApplicable))
                }
            }
            _ => *self,
        }
    }

    pub fn promote(&mut self, role: Role) {
        if let Gesture::End(state) = self {
            state.promotion = Promotion::Role(role);
        }
    }

    pub fn need_promotion(&self) -> bool {
        if let Gesture::End(state) = self {
            state.promotion == Promotion::None
        } else {
            false
        }
    }
}
