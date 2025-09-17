use serde::{Deserialize, Serialize};
use shakmaty::{Color, Move, Role, Square};

#[rustfmt::skip]
#[derive(Serialize, Deserialize)]
#[serde(remote = "Square")]
pub enum SquareDef {
    A1 = 0, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "Role")]
pub enum RoleDef {
    Pawn = 1,
    Knight = 2,
    Bishop = 3,
    Rook = 4,
    Queen = 5,
    King = 6,
}

mod role_option {
    use super::RoleDef;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use shakmaty::Role;

    pub fn serialize<S>(value: &Option<Role>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Helper<'a>(#[serde(with = "RoleDef")] &'a Role);

        value.as_ref().map(Helper).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Role>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper(#[serde(with = "RoleDef")] Role);

        let helper = Option::deserialize(deserializer)?;
        Ok(helper.map(|Helper(external)| external))
    }
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "Move")]
#[serde(tag = "_tag")]
pub enum MoveDef {
    Normal {
        #[serde(with = "RoleDef")]
        role: Role,
        #[serde(with = "SquareDef")]
        from: Square,
        #[serde(with = "role_option")]
        capture: Option<Role>,
        #[serde(with = "SquareDef")]
        to: Square,
        #[serde(with = "role_option")]
        promotion: Option<Role>,
    },
    EnPassant {
        #[serde(with = "SquareDef")]
        from: Square,
        #[serde(with = "SquareDef")]
        to: Square,
    },
    Castle {
        #[serde(with = "SquareDef")]
        king: Square,
        #[serde(with = "SquareDef")]
        rook: Square,
    },
    Put {
        #[serde(with = "RoleDef")]
        role: Role,
        #[serde(with = "SquareDef")]
        to: Square,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MoveSerde(#[serde(with = "MoveDef")] pub Move);

impl From<Move> for MoveSerde {
    fn from(value: Move) -> Self {
        MoveSerde(value)
    }
}

impl From<MoveSerde> for Move {
    fn from(val: MoveSerde) -> Self {
        val.0
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ColorSerde {
    #[serde(rename = "white")]
    White,
    #[serde(rename = "black")]
    Black,
}

impl From<ColorSerde> for Color {
    fn from(val: ColorSerde) -> Self {
        match val {
            ColorSerde::Black => Color::Black,
            ColorSerde::White => Color::White,
        }
    }
}

impl From<Color> for ColorSerde {
    fn from(val: Color) -> Self {
        match val {
            Color::Black => ColorSerde::Black,
            Color::White => ColorSerde::White,
        }
    }
}

// impl Serialize for ColorSerde {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//         where
//             S: serde::Serializer {
//         match *self {
//             ColorSerde::White => serializer.serialize_str("white"),
//             ColorSerde::Black => serializer.serialize_str("black"),
//         }
//     }
// }

// pub trait Serialize {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer;
// }

// pub trait Deserialize<'de>: Sized {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>;
// }
