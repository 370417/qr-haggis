use super::card::{CardValue, SuitSet};
use super::constant::{MAX_RANK, MIN_RANK};

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum CombinationType {
    Bomb(usize),
    Normal(NormalType),
}
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct NormalType {
    start_rank: usize,
    end_rank: usize,
    suit_count: usize,
    num_extra_wildcards: usize,
}

impl NormalType {
    fn rank_count(&self) -> usize {
        self.end_rank - self.start_rank + 1
    }

    fn card_count(&self) -> usize {
        self.suit_count * self.rank_count() + self.num_extra_wildcards
    }

    // Checks if self is compatible with and larger than other.
    // If that is true, returns the disambiguated type of the larger normal combination, which
    // will always be self.
    //
    // When comparing two normal combinations, we can only order them if they share
    // the same type. Some normal combinations can have ambiguous types:
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
    // (2x1 means the normal combination spans 2 ranks and 1 suit)
    // when both self and other are ambiguious, only (3x1 regular, 3 wild => 6 total)
    // and (1x3 regular, 3 wild => 6 total) don't have compatibility, but this case can be caught
    // by the existing test.
    pub fn has_higher_rank_than(&self, other: &Self) -> Option<NormalType> {
        if self.card_count() != other.card_count() {
            return None;
        }

        let combined_suit_count = self.suit_count.max(other.suit_count);
        let combined_rank_count = self.rank_count().max(other.rank_count());

        // If the combined rectangle area is larger than the number of cards, we can't
        // possibly fill it into a valid normal combination.
        if combined_suit_count * combined_rank_count <= self.card_count()
            && self.start_rank > other.start_rank
        {
            Some(NormalType {
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

/// Preassumption: card_values does not represent a bomb
pub fn is_valid_normal(card_values: &Vec<CardValue>) -> Option<NormalType> {
    if card_values.is_empty() {
        return None;
    }

    if card_values.len() == 1 {
        return Some(NormalType {
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
    let min_normal_size = number_of_ranks * suits.len();
    let num_required_wildcards = min_normal_size - num_normal_cards;
    let num_wildcards = card_values.len() - num_normal_cards;

    if card_values.len() == 2 && suits.len() == 1 {
        if num_wildcards == 1 {
            return Some(NormalType {
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
            (true, false) => Some(NormalType {
                start_rank: smallest_rank,
                end_rank: largest_rank + num_extra_wildcards / suits.len(),
                suit_count: suits.len(),
                num_extra_wildcards: 0,
            }),
            // If we can add a horizontal line (eg make the set bigger by adding another suit)
            (false, true) => Some(NormalType {
                start_rank: smallest_rank,
                end_rank: largest_rank,
                suit_count: suits.len() + num_extra_wildcards / number_of_ranks,
                num_extra_wildcards: 0,
            }),
            // We can add either type of line, so leave it ambiguous
            (true, true) => Some(NormalType {
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
pub fn is_bomb(card_values: &Vec<CardValue>) -> Option<usize> {
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

#[cfg(test)]
mod tests_for_is_valid_normal {

    use super::*;

    #[test]
    fn test_valid_normal_single() {
        let card_values: Vec<CardValue> = vec!["2♦"].iter().map(|s| s.parse().unwrap()).collect();
        assert_eq!(
            is_valid_normal(&card_values),
            Some(NormalType {
                start_rank: 2,
                end_rank: 2,
                suit_count: 1,
                num_extra_wildcards: 0
            })
        );
    }

    #[test]
    fn test_valid_wildcard_single() {
        let card_values: Vec<CardValue> = vec!["Q"].iter().map(|s| s.parse().unwrap()).collect();
        assert_eq!(
            is_valid_normal(&card_values),
            Some(NormalType {
                start_rank: 12,
                end_rank: 12,
                suit_count: 1,
                num_extra_wildcards: 0
            })
        );
    }

    #[test]
    fn test_valid_seven_of_a_kind() {
        let card_values: Vec<CardValue> = vec!["10♠", "10♥", "10♦", "10♣", "J", "Q", "K"]
            .iter()
            .map(|s| s.parse().unwrap())
            .collect();
        assert_eq!(
            is_valid_normal(&card_values),
            Some(NormalType {
                start_rank: 10,
                end_rank: 10,
                suit_count: 7,
                num_extra_wildcards: 0
            })
        );
    }

    #[test]
    fn test_valid_three_normal_three_wildcard() {
        let card_values: Vec<CardValue> = vec!["10♠", "10♥", "10♦", "J", "Q", "K"]
            .iter()
            .map(|s| s.parse().unwrap())
            .collect();
        assert_eq!(
            is_valid_normal(&card_values),
            Some(NormalType {
                start_rank: 10,
                end_rank: 10,
                suit_count: 3,
                num_extra_wildcards: 3
            })
        );
    }

    #[test]
    fn test_invalid_two_single_sequence() {
        let card_values: Vec<CardValue> = vec!["7♣", "8♣"]
            .iter()
            .map(|s| s.parse().unwrap())
            .collect();
        assert_eq!(is_valid_normal(&card_values), None);
    }

    #[test]
    fn test_valid_single_sequence() {
        let card_values: Vec<CardValue> = vec!["7♣", "8♣", "9♣"]
            .iter()
            .map(|s| s.parse().unwrap())
            .collect();
        assert_eq!(
            is_valid_normal(&card_values),
            Some(NormalType {
                start_rank: 7,
                end_rank: 9,
                suit_count: 1,
                num_extra_wildcards: 0
            })
        );
    }

    #[test]
    pub fn test_valid_sequence_wildcard() {
        let card_values: Vec<CardValue> = vec!["7♣", "8♣", "10♣", "Q", "K"]
            .iter()
            .map(|s| s.parse().unwrap())
            .collect();
        assert_eq!(
            is_valid_normal(&card_values),
            Some(NormalType {
                start_rank: 7,
                end_rank: 11,
                suit_count: 1,
                num_extra_wildcards: 0
            })
        );
    }

    #[test]
    fn test_invalid_sequence_skip() {
        let card_values: Vec<CardValue> = vec!["7♣", "8♣", "10♣"]
            .iter()
            .map(|s| s.parse().unwrap())
            .collect();
        assert_eq!(is_valid_normal(&card_values), None);
    }

    #[test]
    fn test_invalid_sequence_suit() {
        let card_values: Vec<CardValue> = vec!["7♣", "8♣", "9♠"]
            .iter()
            .map(|s| s.parse().unwrap())
            .collect();
        assert_eq!(is_valid_normal(&card_values), None);
    }

    #[test]
    fn test_valid_double_sequence() {
        let card_values: Vec<CardValue> = vec!["7♥", "7♣", "8♥", "8♣"]
            .iter()
            .map(|s| s.parse().unwrap())
            .collect();
        assert_eq!(
            is_valid_normal(&card_values),
            Some(NormalType {
                start_rank: 7,
                end_rank: 8,
                suit_count: 2,
                num_extra_wildcards: 0
            })
        );
    }

    #[test]
    fn test_valid_extra_wildcards() {
        let card_values: Vec<CardValue> = vec!["2♦", "2♣", "3♣", "J", "Q", "K"]
            .iter()
            .map(|s| s.parse().unwrap())
            .collect();
        assert_eq!(
            is_valid_normal(&card_values),
            Some(NormalType {
                start_rank: 2,
                end_rank: 3,
                suit_count: 2,
                num_extra_wildcards: 2
            })
        );
    }

    #[test]
    fn test_invalid_wildcard() {
        let card_values: Vec<CardValue> = vec!["2♠", "2♥", "3♠", "3♥", "J"]
            .iter()
            .map(|s| s.parse().unwrap())
            .collect();
        assert_eq!(is_valid_normal(&card_values), None);
    }
}

#[cfg(test)]
mod tests_for_is_bomb {
    use super::*;

    #[test]
    fn test_0_bomb() {
        let card_values: Vec<CardValue> = vec!["3♦", "5♠", "7♣", "9♥"]
            .iter()
            .map(|s| s.parse().unwrap())
            .collect();

        assert_eq!(is_bomb(&card_values), Some(0));
    }

    #[test]
    fn test_1_bomb() {
        let card_values: Vec<CardValue> =
            vec!["J", "Q"].iter().map(|s| s.parse().unwrap()).collect();

        assert_eq!(is_bomb(&card_values), Some(1));
    }

    #[test]
    fn test_2_bomb() {
        let card_values: Vec<CardValue> =
            vec!["J", "K"].iter().map(|s| s.parse().unwrap()).collect();

        assert_eq!(is_bomb(&card_values), Some(2));
    }

    #[test]
    fn test_3_bomb() {
        let card_values: Vec<CardValue> =
            vec!["Q", "K"].iter().map(|s| s.parse().unwrap()).collect();

        assert_eq!(is_bomb(&card_values), Some(3));
    }

    #[test]
    fn test_4_bomb() {
        let card_values: Vec<CardValue> = vec!["J", "Q", "K"]
            .iter()
            .map(|s| s.parse().unwrap())
            .collect();

        assert_eq!(is_bomb(&card_values), Some(4));
    }

    #[test]
    fn test_5_bomb() {
        let card_values: Vec<CardValue> = vec!["3♣", "5♣", "7♣", "9♣"]
            .iter()
            .map(|s| s.parse().unwrap())
            .collect();

        assert_eq!(is_bomb(&card_values), Some(5));
    }

    #[test]
    fn test_invalid_0_bomb() {
        let card_values: Vec<CardValue> = vec!["3♦", "5♠", "7♣", "9♣"]
            .iter()
            .map(|s| s.parse().unwrap())
            .collect();

        assert_eq!(is_bomb(&card_values), None);
    }
}

impl std::str::FromStr for CardValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let rank = match chars.next() {
            Some('2') => 2,
            Some('3') => 3,
            Some('4') => 4,
            Some('5') => 5,
            Some('6') => 6,
            Some('7') => 7,
            Some('8') => 8,
            Some('9') => 9,
            Some('1') => match chars.next() {
                Some('0') => 10,
                _ => 0,
            },
            Some('J') => 11,
            Some('Q') => 12,
            Some('K') => 13,
            Some(_) | None => 0,
        };
        let suit = match chars.next() {
            Some('♠') => 0,
            Some('♥') => 1,
            Some('♦') => 2,
            Some('♣') => 3,
            Some(_) | None => 4,
        };
        match (rank, suit) {
            (rank @ 2..=10, suit @ 0..=3) => Ok(CardValue::Normal { rank, suit }),
            (rank @ 11..=13, 4) => Ok(CardValue::Wildcard { rank }),
            _ => Err(()),
        }
    }
}
