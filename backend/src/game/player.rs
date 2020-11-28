#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Player {
    Me,
    Opponent,
}

impl Player {
    pub fn other(self) -> Self {
        match self {
            Player::Me => Player::Opponent,
            Player::Opponent => Player::Me,
        }
    }
}
