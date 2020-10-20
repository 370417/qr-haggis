use rand::prelude::*;

#[derive(Debug, Copy, Clone)]
enum Player {
    Me,
    Opponent,
}

enum Location {
    Haggis,
    Hand(Player),
    /// Table is the location of all cards that players have played.
    /// Order is the number of combinations played (across all tricks) before this card.
    /// Order will have the same value for all the cards in a combination.
    Table {
        captured_by: Option<Player>,
        order: usize,
    }
}

struct Game {
    /// The location of a card with id x is locations[x].
    locations: [Location; 42],
    current_player: Player,
}

enum CardValue {
    Normal {
        /// Rank is in the range 2..=10
        rank: usize,
        /// Suit is in the range 0..4
        suit: usize,
    },
    Wildcard {
        /// Wildcard rank is in the range 11..=13
        rank: usize,
    }
}

#[derive(Copy, Clone)]
struct CardId(usize);

impl CardId {
    /// CardIds: 0  1  2  ...   8  9  ...  35 36 37 38 39 40 41
    /// Ranks:   2  3  4  ...  10  2  ...  10  J  Q  K  J  Q  K
    /// Suits:   0  0  0  ...   0  1  ...   3
    fn to_value(self) -> CardValue {
        if self.0 < 36 {
            CardValue::Normal {
                rank: 2 + (self.0 % 9),
                suit: self.0 / 9,
            }
        } else {
            CardValue::Wildcard {
                rank: 11 + (self.0 % 3),
            }
        }
    }
}

impl Game {
    /// Create and initialize a new game state.
    fn create_state(qr_code: Option<&[u8]>) -> Self {
        let game = Game {
            locations: [Location::Haggis; 42],
            current_player: Player::Me,
        };
        game.set_state(qr_code);
        game
    }

    fn set_state(&mut self, qr_code: Option<&[u8]>) {
        match qr_code {
            Some(_) => todo!(),
            None => {
                // Even though we only loop over the first 28 indices, we still
                // need to shuffle all 36 normal cards so that the Haggis gets randomized.
                let indices: Vec<_> = (0..36).collect();
                indices.shuffle(&mut rand::thread_rng());

                for i in 0..14 {
                    let rand_i = indices[i];
                    self.locations[rand_i] = Location::Hand(Player::Me);
                }
                for i in 14..28 {
                    let rand_i = indices[i];
                    self.locations[rand_i] = Location::Hand(Player::Opponent);
                }

                // Cleaner version of the for loops:
                for &i in &indices[0..14] {
                    self.locations[i] = Location::Hand(Player::Me);
                }
                for &i in &indices[14..28] {
                    self.locations[i] = Location::Hand(Player::Opponent);
                }

                // Also I added the locations of the wild cards
                for i in 36..39 {
                    self.locations[i] = Location::Hand(Player::Me);
                }
                for i in 39..42 {
                    self.locations[i] = Location::Hand(Player::Opponent);
                }
            }
        }
    }
}
