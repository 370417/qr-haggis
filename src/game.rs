use std::collections::HashMap;

use crate::compression::{decode_game, encode_game};
use constant::*;
use image::{DynamicImage, ImageBuffer, Luma};
use qrcode::QrCode;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

pub mod constant {
    pub const HAGGIS_SIZE: usize = 8;
    pub const NUM_WILDCARDS_PER_PLAYER: usize = 3;
    pub const NUM_PLAYERS: usize = 2;
    pub const MIN_RANK: usize = 2;
    pub const MAX_RANK: usize = 10;
    pub const NUM_RANKS: usize = (MAX_RANK - MIN_RANK + 1);
    pub const NUM_SUITS: usize = 4;
    pub const NUM_NORMAL: usize = NUM_RANKS * NUM_SUITS;
    pub const DECK_SIZE: usize = (NUM_SUITS * NUM_RANKS) + (NUM_WILDCARDS_PER_PLAYER * NUM_PLAYERS);
    pub const INIT_HAND_SIZE_WO_WILDCARD: usize = (NUM_NORMAL - HAGGIS_SIZE) / 2;
}

#[cfg(test)]
mod tests;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq)]
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

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub enum Location {
    Haggis,
    Hand(Player),
    /// Table is the location of all cards that players have played.
    /// Order is the number of combinations played (across all tricks) before this card.
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

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct CombinationType {
    start_rank: usize,
    end_rank: usize,
    suit_count: usize,
    num_extra_wildcards: usize,
}

impl CombinationType {
    fn rank_count(&self) -> usize {
        self.end_rank - self.start_rank + 1
    }

    fn card_count(&self) -> usize {
        self.suit_count * self.rank_count() + self.num_extra_wildcards
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct Game {
    /// The location of a card with id x is locations[x].
    pub locations: Vec<Location>,
    pub current_player: Player,
    pub me_went_first: bool,
    /// Empty if game just started or the previous player just passed
    pub last_trick: Vec<CardId>,
    /// Type (including disambiguations) of the last trick played
    pub last_trick_type: Option<TrickType>,
    /// The order of the first combination played that has not yet been captured
    /// so that we can efficiently search for the non-captured cards
    pub current_start_order: usize,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub enum TrickType {
    Bomb(usize),
    Combination(CombinationType),
}

#[derive(Serialize, Deserialize)]
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
    },
}

impl CardValue {
    /// How many points this card value scores at the end of a game.
    fn point_value(&self) -> usize {
        match self {
            Self::Normal { rank, .. } => rank % 2,
            Self::Wildcard { rank: 11 } => 2,
            Self::Wildcard { rank: 12 } => 3,
            Self::Wildcard { rank: 13 } => 5,
            _ => panic!("Invalid wildcard rank"),
        }
    }

    fn rank(&self) -> usize {
        match self {
            CardValue::Normal { rank, .. } => *rank,
            CardValue::Wildcard { rank } => *rank,
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct CardId(pub usize);

impl CardId {
    /// CardIds: 0  1  2  ...   8  9  ...  35 36 37 38 39 40 41
    /// Ranks:   2  3  4  ...  10  2  ...  10  J  Q  K  J  Q  K
    /// Suits:   0  0  0  ...   0  1  ...   3
    fn to_value(self) -> CardValue {
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

impl Game {
    /// Create and initialize a new game state.
    pub fn create_state(qr_code: Option<DynamicImage>) -> Self {
        let mut game = Game {
            locations: Vec::new(),
            current_player: Player::Me,
            me_went_first: true,
            last_trick: Vec::new(),
            last_trick_type: None,
            current_start_order: 0,
        };
        for _ in 0..DECK_SIZE {
            game.locations.push(Location::Haggis);
        }
        match qr_code {
            Some(data) => game.read_qr_code(data),
            None => game.init_state(),
        };
        game
    }

    pub fn read_qr_code(&mut self, image: DynamicImage) {
        // convert to gray scale
        let img_gray = image.into_luma();

        // create a decoder
        let mut decoder = quircs::Quirc::default();

        // identify all qr codes
        let mut codes = decoder.identify(
            img_gray.width() as usize,
            img_gray.height() as usize,
            &img_gray,
        );

        let code = codes
            .next()
            .expect("found no qr codes")
            .expect("failed to extract qr code");
        let decoded = code.decode().expect("failed to decode qr code");

        *self = decode_game(&decoded.payload);
    }

    pub fn write_qr_code(&self, width: usize, height: usize) -> ImageBuffer<Luma<u8>, Vec<u8>> {
        let encoded_game = encode_game(self);

        // Encode some data into bits.
        let code = QrCode::new(&encoded_game).unwrap();

        // Render the bits into an image.
        code.render::<Luma<u8>>().build()
    }

    pub fn init_state(&mut self) {
        // Even though we only loop over the first 28 indices, we still
        // need to shuffle all 36 normal cards so that the Haggis gets randomized.
        let mut indices: Vec<_> = (0..NUM_NORMAL).collect();
        indices.shuffle(&mut rand::thread_rng());

        // Cleaner version of the for loops:
        for &i in &indices[0..INIT_HAND_SIZE_WO_WILDCARD] {
            self.locations[i] = Location::Hand(Player::Me);
        }
        for &i in &indices[INIT_HAND_SIZE_WO_WILDCARD..(INIT_HAND_SIZE_WO_WILDCARD * 2)] {
            self.locations[i] = Location::Hand(Player::Opponent);
        }

        for i in NUM_NORMAL..(NUM_NORMAL + NUM_WILDCARDS_PER_PLAYER) {
            self.locations[i] = Location::Hand(Player::Me);
        }
        for i in (NUM_NORMAL + NUM_WILDCARDS_PER_PLAYER)..DECK_SIZE {
            self.locations[i] = Location::Hand(Player::Opponent);
        }
    }

    pub fn get_hand(&self, player: Player) -> Vec<CardId> {
        let mut hand = Vec::new();
        for (i, location) in self.locations.iter().enumerate() {
            if let Location::Hand(owner) = location {
                if *owner == player {
                    hand.push(CardId(i));
                }
            }
        }
        hand
    }

    /// Return the non-captured cards of the table
    pub fn get_table(&self) -> HashMap<usize, Vec<CardId>> {
        if self.last_trick.is_empty() {
            return HashMap::new();
        }

        let mut combinations: HashMap<usize, Vec<CardId>> = HashMap::new();

        for (i, location) in self.locations.iter().enumerate() {
            if let Location::Table {
                captured_by: None,
                order,
            } = location
            {
                combinations.entry(*order).or_default().push(CardId(i));
            }
        }
        combinations
    }

    /// If game is over, return (my_score, opponent_score)
    pub fn is_game_over(&self) -> Option<(usize, usize)> {
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
                    Location::Table {
                        captured_by: Some(Player::Me),
                        ..
                    } => {
                        my_score += card_id.to_value().point_value();
                    }
                    Location::Table {
                        captured_by: Some(Player::Opponent),
                        ..
                    } => {
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

    pub fn get_opponent_num_of_card(&self) -> usize {
        let mut opponent_num_of_card: usize = 0;
        for location in self.locations.iter() {
            match location {
                Location::Hand(Player::Opponent) => opponent_num_of_card += 1,
                _ => {}
            }
        }
        opponent_num_of_card
    }

    pub fn pass(&mut self) {
        // capture the cards on the table
        let winner_of_the_trick = if let Some(TrickType::Bomb(_)) = self.last_trick_type {
            self.current_player
        } else {
            self.current_player.other()
        };
        for location in &mut self.locations {
            if let Location::Table { captured_by, .. } = location {
                if captured_by.is_none() {
                    *captured_by = Some(winner_of_the_trick);
                }
            }
        }

        // store the new current start order
        self.current_start_order = self.get_last_trick_order() + 1;

        // update last combination and its type
        self.last_trick = Vec::new();
        self.last_trick_type = None;

        // change player
        self.current_player = self.current_player.other();
    }

    // we assume card_ids is not empty
    pub fn play_cards(&mut self, card_ids: Vec<CardId>) -> bool {
        let card_values = card_ids.iter().map(|id| id.to_value()).collect();

        let current_trick_type = if let Some(bomb_rank) = is_bomb(&card_values) {
            Some(TrickType::Bomb(bomb_rank))
        } else {
            is_valid_combination(&card_values)
                .map(|combination_type| TrickType::Combination(combination_type))
        };

        use TrickType::*;
        self.last_trick_type = match (&self.last_trick_type, &current_trick_type) {
            (_, None) => return false,
            (Some(Bomb(last_bomb)), Some(Bomb(current_bomb))) => {
                if current_bomb <= last_bomb {
                    return false;
                } else {
                    current_trick_type
                }
            }
            (Some(Bomb(_)), Some(Combination(_))) => return false,
            (Some(Combination(last_combination)), Some(Combination(current_combination))) => {
                if let Some(new_combination_type) =
                    current_combination.has_higher_rank_than(last_combination)
                {
                    Some(Combination(new_combination_type))
                } else {
                    return false;
                }
            }
            _ => current_trick_type,
        };

        // get the current order
        let current_order = if self.last_trick.is_empty() {
            self.current_start_order
        } else {
            self.get_last_trick_order() + 1
        };
        // move the cards to table
        for card_id in card_ids.iter() {
            self.locations[card_id.0] = Location::Table {
                order: current_order,
                captured_by: None,
            };
        }

        // update the last trick
        self.last_trick = card_ids;

        // change the current player
        self.current_player = self.current_player.other();
        return true;
    }

    // Swap Me and Opponent
    pub fn switch_perspective(&mut self) {
        for location in &mut self.locations {
            match location {
                Location::Hand(player) => *player = player.other(),
                Location::Table {
                    captured_by: Some(player),
                    ..
                } => *player = player.other(),
                _ => {}
            }
        }
        self.current_player = self.current_player.other();
        self.me_went_first = !self.me_went_first;
    }

    // Preassumption: last_trick is not empty
    fn get_last_trick_order(&self) -> usize {
        let current_end_location = &self.locations[self.last_trick[0].0];
        match current_end_location {
            Location::Table { order, .. } => *order,
            _ => panic!("Invalid self.last_trick"),
        }
    }
}

pub struct SuitSet([usize; NUM_SUITS]);

impl SuitSet {
    fn new() -> Self {
        SuitSet([0; NUM_SUITS])
    }

    fn insert(&mut self, suit: usize) {
        self.0[suit] = 1;
    }

    fn len(&self) -> usize {
        self.0.iter().sum()
    }
}

// fn find_trick_type(card_values: &Vec<CardValue>) -> TrickType {}

/// Preassumption: card_values does not represent a bomb
fn is_valid_combination(card_values: &Vec<CardValue>) -> Option<CombinationType> {
    if card_values.is_empty() {
        return None;
    }

    if card_values.len() == 1 {
        return Some(CombinationType {
            start_rank: card_values[0].rank(),
            end_rank: card_values[0].rank(),
            suit_count: 1,
            num_extra_wildcards: 0,
        });
    }

    let mut smallest_rank = MAX_RANK + 1;
    let mut largest_rank = MIN_RANK - 1;
    let mut suits = SuitSet::new();
    let mut num_normal_cards: usize = 0;
    for value in card_values {
        if let CardValue::Normal { rank, suit } = value {
            num_normal_cards += 1;
            smallest_rank = smallest_rank.min(*rank);
            largest_rank = largest_rank.max(*rank);
            suits.insert(*suit);
        }
    }

    // smallest_rank > largest_rank will happen if all the cards were wildcards.
    // Without this check, number_of_ranks can underflow.
    assert!(smallest_rank <= largest_rank);

    let number_of_ranks = largest_rank - smallest_rank + 1;
    let min_combination_size = number_of_ranks * suits.len();
    let num_required_wildcards = min_combination_size - num_normal_cards;
    let num_wildcards = card_values.len() - num_normal_cards;

    if card_values.len() == 2 && suits.len() == 1 {
        if num_wildcards == 1 {
            return Some(CombinationType {
                start_rank: smallest_rank,
                end_rank: largest_rank,
                suit_count: 2,
                num_extra_wildcards: 0,
            });
        } else {
            return None;
        }
    }

    if num_required_wildcards > num_wildcards {
        None
    } else {
        let num_extra_wildcards = num_wildcards - num_required_wildcards;

        match (
            num_extra_wildcards % suits.len() == 0,
            num_extra_wildcards % number_of_ranks == 0,
        ) {
            // The number of extra wildcards doesn't fit an edge
            (false, false) => None,
            // If we can add a vertical line (eg make the sequence longer by adding another rank)
            (true, false) => Some(CombinationType {
                start_rank: smallest_rank,
                end_rank: largest_rank + num_extra_wildcards / suits.len(),
                suit_count: suits.len(),
                num_extra_wildcards: 0,
            }),
            // If we can add a horizontal line (eg make the set bigger by adding another suit)
            (false, true) => Some(CombinationType {
                start_rank: smallest_rank,
                end_rank: largest_rank,
                suit_count: suits.len() + num_extra_wildcards / number_of_ranks,
                num_extra_wildcards: 0,
            }),
            // We can add either type of line, so leave it ambiguous
            (true, true) => Some(CombinationType {
                start_rank: smallest_rank,
                end_rank: largest_rank,
                suit_count: suits.len(),
                num_extra_wildcards,
            }),
        }
    }
}

// 0:    3-5-7-9 (these 4 ranks in 4 different suits, no wild cards)
// 1:    J-Q
// 2:    J-K
// 3:    Q-K
// 4:    J-Q-K
// 5:    3-5-7-9 (these 4 ranks in one suit, no wild cards)
// None: not a bomb
fn is_bomb(card_values: &Vec<CardValue>) -> Option<usize> {
    if card_values.len() == 4 {
        let mut rank_bit_mask = 0;
        for card_value in card_values {
            let rank = card_value.rank();
            rank_bit_mask |= 1 << rank;
        }
        if rank_bit_mask == 0b1010101000 {
            // is 3-5-7-9
            let mut suit_bit_mask = 0;
            for i in 0..4 {
                let suit = match card_values[i] {
                    CardValue::Normal { suit, .. } => suit,
                    _ => panic!("Should never reach this line"),
                };
                suit_bit_mask |= 1 << suit;
            }
            if suit_bit_mask == 0b1111 {
                return Some(0);
            } else if suit_bit_mask == 0b1
                || suit_bit_mask == 0b10
                || suit_bit_mask == 0b100
                || suit_bit_mask == 0b1000
            {
                return Some(5);
            } else {
                return None;
            }
        } else {
            return None;
        }
    } else {
        let mut rank_bit_mask = 0;
        for card_value in card_values.iter() {
            let rank = card_value.rank();
            rank_bit_mask |= 1 << rank;
        }
        println!("{}", rank_bit_mask);
        return match rank_bit_mask {
            0b01100000000000 => Some(1),
            0b10100000000000 => Some(2),
            0b11000000000000 => Some(3),
            0b11100000000000 => Some(4),
            _ => None,
        };
    }
}

impl CombinationType {
    // Checks if self is compatible with and larger than other.
    // If that is true, returns the disambiguated type of the larger combination, which
    // will always be self.
    //
    // When comparing two combinations, we can only order them if they share
    // the same type. Some combinations can have ambiguous types:
    //
    // 1x1 regular, 2 wild => 3 total
    // 1x1 regular, 3 wild => 4 total
    // 2x1 regular, 2 wild => 4 total
    // 1x2 regular, 2 wild => 4 total
    // 3x1 regular, 3 wild => 6 total
    // 1x3 regular, 3 wild => 6 total
    // 2x2 regular, 2 wild => 6 total
    // 3x3 regular, 3 wild => 12 total
    //
    // (2x1 means the combination spans 2 ranks and 1 suit)
    // when both self and other are ambiguious, only (3x1 regular, 3 wild => 6 total)
    // and (1x3 regular, 3 wild => 6 total) don't have compatibility, but this case can be caught
    // by the existing test.
    fn has_higher_rank_than(&self, other: &Self) -> Option<CombinationType> {
        if self.card_count() != other.card_count() {
            return None;
        }

        let combined_suit_count = self.suit_count.max(other.suit_count);
        let combined_rank_count = self.rank_count().max(other.rank_count());

        // If the combined rectangle area is larger than the number of cards, we can't
        // possibly fill it into a valid combination.
        if combined_suit_count * combined_rank_count <= self.card_count()
            && self.start_rank > other.start_rank
        {
            Some(CombinationType {
                start_rank: self.start_rank,
                end_rank: self.start_rank + combined_rank_count - 1,
                suit_count: combined_suit_count,
                num_extra_wildcards: self.card_count() - combined_suit_count * combined_rank_count,
            })
        } else {
            None
        }
    }
}
