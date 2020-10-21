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
    /// Empty if game just started or the previous player just passed
    last_combination: Vec<CardId>,
    /// The order of the first combination played that has not yet been captured
    /// so that we can efficiently search for the non-captured cards
    current_start_order: usize,
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

impl CardValue {
    fn point_value(&self) -> usize {
        match self {
            Self::Normal { rank, .. } => rank % 2,
            Self::Wildcard { rank: 11 } => 2,
            Self::Wildcard { rank: 12 } => 3,
            Self::Wildcard { rank: 13 } => 5,
            _ => panic!("Invalid wildcard rank"),
        }
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
            last_combination: Vec::new(),
            current_start_order: 0,
        };
        match qr_code {
            Some(data) => game.set_state(data),
            None => game.init_state(),
        };
        game
    }

    fn set_state(&mut self, qr_code: &[u8]) {
        todo!();
    }

    fn init_state(&mut self) {
        // Even though we only loop over the first 28 indices, we still
        // need to shuffle all 36 normal cards so that the Haggis gets randomized.
        let indices: Vec<_> = (0..36).collect();
        indices.shuffle(&mut rand::thread_rng());

        // Cleaner version of the for loops:
        for &i in &indices[0..14] {
            self.locations[i] = Location::Hand(Player::Me);
        }
        for &i in &indices[14..28] {
            self.locations[i] = Location::Hand(Player::Opponent);
        }

        for i in 36..39 {
            self.locations[i] = Location::Hand(Player::Me);
        }
        for i in 39..42 {
            self.locations[i] = Location::Hand(Player::Opponent);
        }
    }

    fn get_hand(&self) -> Vec<CardId> {
        let mut hand = Vec::new();
        for (i, location) in self.locations.iter().enumerate() {
            if let Location::Hand(Player::Me) = location {
                hand.push(CardId(i));
            }
        }
        hand
    }

    /// Return the non-captured cards of the table
    fn get_table(&self) -> Vec<Vec<CardId>> {
        if self.last_combination.is_empty() {
            return Vec::new();
        }

        let current_end_location = self.locations[self.last_combination[0].0];
        let current_end_order = match current_end_location {
            Location::Table { order, captured_by: None } => order,
            _ => panic!("Invalid self.last_combination"),
        };

        let mut combinations = vec![Vec::new(); 1 + current_end_order - self.current_start_order];

        for (i, location) in self.locations.iter().enumerate() {
            if let Location::Table { captured_by: None, order } = location {
                combinations[order - self.current_start_order].push(CardId(i));
            }
        }
        combinations
    }

    /// If game is over, return (my_score, opponent_score)
    fn is_game_over(&self) -> Option<(usize, usize)> {
        let mut my_card_count = 0;
        let mut opponent_card_count = 0;

        for location in self.locations.iter() {
            match location {
                Location::Hand(Player::Me) => my_card_count += 1,
                Location::Hand(Player::Opponent) => opponent_card_count += 1,
                _ => {}
            };
        }

        if my_card_count == 0 || opponent_card_count == 0 {
            let mut my_score = 0;
            let mut opponent_score = 0;

            let mut winner_of_hand_bonus = 0;

            // The winner of the hand scores 5 points for each card in her opponent's hand.
            // Remember, the wild cards count as part of the hand.
            winner_of_hand_bonus += 5 * (my_card_count + opponent_card_count);

            for (i, location) in self.locations.iter().enumerate() {
                let card_id = CardId(i);
                match location {
                    // All point  cards (i.e., any 3, 5, 7, 9, J, Q, or K), captured during trick play,
                    // are  scored by the capturing player.
                    Location::Table { captured_by: Some(Player::Me), .. } => {
                        my_score += card_id.to_value().point_value();
                    }
                    Location::Table { captured_by: Some(Player::Opponent), .. } => {
                        opponent_score += card_id.to_value().point_value();
                    }
                    // Point cards left in the opponent's hand and any point cards found in the
                    // Haggis are scored by the player who won the hand.
                    Location::Hand(..) => {
                        winner_of_hand_bonus += card_id.to_value().point_value();
                    }
                    _ => {}
                }
            }

            if my_card_count == 0 {
                my_score += winner_of_hand_bonus;
            } else {
                opponent_score += winner_of_hand_bonus;
            }

            Some((my_score, opponent_score))
        } else {
            None
        }
    }
}
