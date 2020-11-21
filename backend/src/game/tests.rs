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

mod tests_for_qr_code {
    use super::*;

    #[test]
    fn test_write_and_read_qr_code() {
        use Location::*;
        use Player::*;
        let mut game = Game {
            locations: vec![
                Hand(Opponent),
                Haggis,
                Hand(Opponent),
                Hand(Opponent),
                Haggis,
                Haggis,
                Hand(Opponent),
                Hand(Me),
                Hand(Me),
                Hand(Opponent),
                Hand(Me),
                Hand(Me),
                Hand(Me),
                Hand(Me),
                Hand(Opponent),
                Hand(Me),
                Haggis,
                Hand(Opponent),
                Hand(Me),
                Hand(Opponent),
                Haggis,
                Hand(Opponent),
                Hand(Me),
                Hand(Me),
                Hand(Opponent),
                Haggis,
                Hand(Me),
                Hand(Me),
                Hand(Opponent),
                Haggis,
                Hand(Me),
                Hand(Opponent),
                Hand(Opponent),
                Haggis,
                Hand(Opponent),
                Hand(Me),
                Hand(Opponent),
                Hand(Opponent),
                Hand(Opponent),
                Hand(Me),
                Hand(Me),
                Hand(Me),
            ],
            current_player: Me,
            me_went_first: true,
            last_combination_type: None,
            next_order: 0,
        };

        game.play_cards(vec![CardId(11), CardId(12), CardId(13)]);

        let mut game_from_qr_code = Game::create_state(None);

        let qr_code = game.write_qr_code(200, 200);
        let dynamic_qr_code = DynamicImage::ImageLuma8(qr_code);
        game_from_qr_code.read_qr_code(dynamic_qr_code);

        assert_eq!(game, game_from_qr_code);
    }

    #[test]
    fn test_switch_perspective() {
        use Location::*;
        use Player::*;
        let mut game = Game {
            locations: vec![
                Hand(Opponent),
                Haggis,
                Hand(Opponent),
                Hand(Opponent),
                Haggis,
                Haggis,
                Hand(Opponent),
                Hand(Me),
                Hand(Me),
                Hand(Opponent),
                Hand(Me),
                Hand(Me),
                Hand(Me),
                Hand(Me),
                Hand(Opponent),
                Hand(Me),
                Haggis,
                Hand(Opponent),
                Hand(Me),
                Hand(Opponent),
                Haggis,
                Hand(Opponent),
                Hand(Me),
                Hand(Me),
                Hand(Opponent),
                Haggis,
                Hand(Me),
                Hand(Me),
                Hand(Opponent),
                Haggis,
                Hand(Me),
                Hand(Opponent),
                Hand(Opponent),
                Haggis,
                Hand(Opponent),
                Hand(Me),
                Hand(Opponent),
                Hand(Opponent),
                Hand(Opponent),
                Hand(Me),
                Hand(Me),
                Hand(Me),
            ],
            current_player: Me,
            me_went_first: true,
            last_combination_type: None,
            next_order: 0,
        };

        game.play_cards(vec![CardId(11), CardId(12), CardId(13)]);
        let game_copy = game.clone();

        println!(
            "game size: {}, location size: {}",
            std::mem::size_of::<Game>(),
            std::mem::size_of::<Location>()
        );

        assert_eq!(game.current_player, Player::Opponent);
        game.switch_perspective();
        assert_eq!(game.current_player, Player::Me);
        game.switch_perspective();
        assert_eq!(game, game_copy);
    }
}
