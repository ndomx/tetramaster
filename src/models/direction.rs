#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

impl Direction {
    pub fn opposite(&self) -> Self {
        match self {
            Self::N => Self::S,
            Self::NE => Self::SW,
            Self::E => Self::W,
            Self::SE => Self::NW,
            Self::S => Self::N,
            Self::SW => Self::NE,
            Self::W => Self::E,
            Self::NW => Self::SE,
        }
    }
}

impl TryFrom<u8> for Direction {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::N),
            1 => Ok(Self::NE),
            2 => Ok(Self::E),
            3 => Ok(Self::SE),
            4 => Ok(Self::S),
            5 => Ok(Self::SW),
            6 => Ok(Self::W),
            7 => Ok(Self::NW),
            _ => Err(()),
        }
    }
}
