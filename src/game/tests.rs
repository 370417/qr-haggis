use super::*;

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

#[test]
fn test_valid_normal_single() {
    let card_values: Vec<CardValue> = vec!["2♦"].iter().map(|s| s.parse().unwrap()).collect();
    assert_eq!(
        is_valid_combination(&card_values),
        Some(CombinationType {
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
        is_valid_combination(&card_values),
        Some(CombinationType {
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
        is_valid_combination(&card_values),
        Some(CombinationType {
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
        is_valid_combination(&card_values),
        Some(CombinationType {
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
    assert_eq!(is_valid_combination(&card_values), None);
}

#[test]
fn test_valid_single_sequence() {
    let card_values: Vec<CardValue> = vec!["7♣", "8♣", "9♣"]
        .iter()
        .map(|s| s.parse().unwrap())
        .collect();
    assert_eq!(
        is_valid_combination(&card_values),
        Some(CombinationType {
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
        is_valid_combination(&card_values),
        Some(CombinationType {
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
    assert_eq!(is_valid_combination(&card_values), None);
}

#[test]
fn test_invalid_sequence_suit() {
    let card_values: Vec<CardValue> = vec!["7♣", "8♣", "9♠"]
        .iter()
        .map(|s| s.parse().unwrap())
        .collect();
    assert_eq!(is_valid_combination(&card_values), None);
}

#[test]
fn test_valid_double_sequence() {
    let card_values: Vec<CardValue> = vec!["7♥", "7♣", "8♥", "8♣"]
        .iter()
        .map(|s| s.parse().unwrap())
        .collect();
    assert_eq!(
        is_valid_combination(&card_values),
        Some(CombinationType {
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
        is_valid_combination(&card_values),
        Some(CombinationType {
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
    assert_eq!(is_valid_combination(&card_values), None);
}
