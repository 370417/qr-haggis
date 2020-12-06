use super::location::Location;
use super::player::Player;
use super::Game;

mod tests_for_qr_code {
    use image::DynamicImage;

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

        game.play_cards(vec![11, 12, 13]);

        let mut game_from_qr_code = Game::create_state(None);

        let qr_code = game.write_qr_code(200, 200);
        let dynamic_qr_code = DynamicImage::ImageRgba8(qr_code);
        game_from_qr_code.read_qr_code(dynamic_qr_code);
        game_from_qr_code.switch_perspective();

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

        game.play_cards(vec![11, 12, 13]);
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
