use super::player::Player;

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Location {
    Haggis,
    Hand(Player),
    /// Table is the location of all cards that players have played.
    /// Order is the number of combinations played (across all combination groups) before this card.
    /// Order will have the same value for all the cards in a combination.
    Table {
        captured_by: Option<Player>,
        order: usize,
    },
}

impl Location {
    pub fn captured_by(&self) -> Option<Player> {
        match self {
            Location::Table { captured_by, .. } => *captured_by,
            _ => panic!(),
        }
    }
}
