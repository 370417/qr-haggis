use super::constant::{NUM_NORMAL, NUM_RANKS, NUM_SUITS};

pub enum CardValue {
    Normal {
        /// Rank is in the range 2..=10
        rank: usize,
        /// Suit is in the range 0..4
        suit: usize,
    },
    Wildcard {
        /// Wildcard rank is in the range 11..=13
        rank: usize,
    },
}

impl CardValue {
    /// How many points this card value scores at the end of a game.
    pub fn point_value(&self) -> usize {
        match self {
            Self::Normal { rank, .. } => rank % 2,
            Self::Wildcard { rank: 11 } => 2,
            Self::Wildcard { rank: 12 } => 3,
            Self::Wildcard { rank: 13 } => 5,
            _ => panic!("Invalid wildcard rank"),
        }
    }

    pub fn rank(&self) -> usize {
        match self {
            CardValue::Normal { rank, .. } => *rank,
            CardValue::Wildcard { rank } => *rank,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct CardId(pub usize);

impl CardId {
    /// CardIds: 0  1  2  ...   8  9  ...  35 36 37 38 39 40 41
    /// Ranks:   2  3  4  ...  10  2  ...  10  J  Q  K  J  Q  K
    /// Suits:   0  0  0  ...   0  1  ...   3
    pub fn to_value(self) -> CardValue {
        if self.0 < NUM_NORMAL {
            CardValue::Normal {
                rank: 2 + (self.0 % NUM_RANKS),
                suit: self.0 / NUM_RANKS,
            }
        } else {
            CardValue::Wildcard {
                rank: 11 + (self.0 % 3),
            }
        }
    }
}

pub struct SuitSet([usize; NUM_SUITS]);

impl SuitSet {
    pub fn new() -> Self {
        SuitSet([0; NUM_SUITS])
    }

    pub fn insert(&mut self, suit: usize) {
        self.0[suit] = 1;
    }

    pub fn len(&self) -> usize {
        self.0.iter().sum()
    }
}
