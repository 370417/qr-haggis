use crate::compression::{compress_hand, decode_game, encode_game};
use card::*;
use combination_type::*;
use constant::*;
use image::{load_from_memory_with_format, DynamicImage, ImageBuffer, ImageFormat::Png, Rgba};
use location::Location;
use player::Player;
use qrcode::QrCode;
use rand::prelude::*;
use wasm_bindgen::prelude::*;

pub mod card;
mod combination_type;
pub mod constant;
pub mod location;
pub mod player;

#[cfg(test)]
mod tests;

// The game has three levels:
// - Combination: I play a combination, you play a combination
// - CombinationGroup: I pass, you pass
// - Game (called hand in the rulebook): I empty my hand, you empty your hand

#[wasm_bindgen]
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Game {
    /// The location of a card with id x is locations[x].
    #[wasm_bindgen(skip)]
    pub locations: Vec<Location>,
    #[wasm_bindgen(skip)]
    pub current_player: Player,
    #[wasm_bindgen(skip)]
    pub me_went_first: bool,
    /// Type (including disambiguations) of the last combination played
    #[wasm_bindgen(skip)]
    pub last_combination_type: Option<CombinationType>,
    /// The order that the next card combination will have
    #[wasm_bindgen(skip)]
    pub next_order: usize,
}
#[wasm_bindgen]
pub enum CardFrontendState {
    Haggis,
    InMyHand,
    JustPlayed,
    ThisCombinationGroup,
    CapturedByMe,
    CapturedByOpponent,
    InOpponentHand,
}

#[wasm_bindgen]
pub enum GameStage {
    BeforeGame,
    Play,
    Wait,
    GameOver,
}

#[wasm_bindgen]
impl Game {
    pub fn new() -> Self {
        let mut game = Game {
            locations: Vec::new(),
            current_player: Player::Me,
            me_went_first: true,
            last_combination_type: None,
            next_order: 0,
        };
        for _ in 0..DECK_SIZE {
            game.locations.push(Location::Haggis);
        }
        // Even though we only loop over the first 28 indices, we still
        // need to shuffle all 36 normal cards so that the Haggis gets randomized.
        let mut indices: Vec<_> = (0..NUM_NORMAL).collect();
        indices.shuffle(&mut rand::thread_rng());

        // Cleaner version of the for loops:
        for &i in &indices[0..INIT_HAND_SIZE_WO_WILDCARD] {
            game.locations[i] = Location::Hand(Player::Me);
        }
        for &i in &indices[INIT_HAND_SIZE_WO_WILDCARD..(INIT_HAND_SIZE_WO_WILDCARD * 2)] {
            game.locations[i] = Location::Hand(Player::Opponent);
        }

        for i in NUM_NORMAL..(NUM_NORMAL + NUM_WILDCARDS_PER_PLAYER) {
            game.locations[i] = Location::Hand(Player::Me);
        }
        for i in (NUM_NORMAL + NUM_WILDCARDS_PER_PLAYER)..DECK_SIZE {
            game.locations[i] = Location::Hand(Player::Opponent);
        }

        game
    }

    pub fn from_qr_code(&mut self, image_data: &[u8]) -> bool {
        match load_from_memory_with_format(image_data, Png) {
            Ok(image) => self.read_qr_code(image).is_ok(),
            Err(_) => false,
        }
    }

    pub fn to_qr_code(&self, width: usize, height: usize) -> js_sys::Uint8ClampedArray {
        let image = self.write_qr_code(width, height);
        let pixels = image.into_raw();

        // unsafe because wasm's memory might change after dynamic allocation
        // unsafe { js_sys::Uint8ClampedArray::view(&image.into_raw()) }

        let array = js_sys::Uint8ClampedArray::new_with_length(pixels.len() as u32);
        for (i, pixel) in pixels.into_iter().enumerate() {
            array.set_index(i as u32, pixel);
        }
        array
    }

    pub fn from_compressed(&mut self, data: &[u8]) -> bool {
        if let Some(game) = decode_game(data) {
            *self = game;
            self.switch_perspective();
            true
        } else {
            false
        }
    }

    pub fn to_compressed(&self) -> js_sys::Uint8Array {
        // unsafe { js_sys::Uint8Array::view(&encode_game(self)) }

        let bytes = encode_game(self);
        let array = js_sys::Uint8Array::new_with_length(bytes.len() as u32);
        for (i, byte) in bytes.into_iter().enumerate() {
            array.set_index(i as u32, byte);
        }

        array
    }

    /// Returns the client id: 4 bytes for my hand followed by 4 bytes for
    /// opponent's hand. Assumes that at most one combination has been played.
    /// Cards on the table are counted as cards in the opponent's hand because
    /// they must have been played by the opponent.
    pub fn get_client_id(&self) -> js_sys::Uint8Array {
        let mut my_sorted_hand = Vec::with_capacity(INIT_HAND_SIZE_WO_WILDCARD);
        let mut opponent_sorted_hand = Vec::with_capacity(INIT_HAND_SIZE_WO_WILDCARD);

        for (card_id, location) in self.locations[0..NUM_NORMAL].iter().enumerate() {
            match location {
                Location::Hand(Player::Me) => my_sorted_hand.push(card_id),
                Location::Hand(Player::Opponent) | Location::Table { .. } => {
                    opponent_sorted_hand.push(card_id)
                }
                _ => {}
            }
        }

        let my_compressed_hand = compress_hand(&my_sorted_hand).to_le_bytes();
        let opponent_compressed_hand = compress_hand(&opponent_sorted_hand).to_le_bytes();

        let array = js_sys::Uint8Array::new_with_length(8);
        for i in 0..4 {
            array.set_index(i, my_compressed_hand[i as usize]);
        }
        for i in 0..4 {
            array.set_index(4 + i, opponent_compressed_hand[i as usize]);
        }

        array
    }

    // card_ids can be empty
    // Returns true on success, false on failure
    // Assumption: current_player == Player::Me
    pub fn can_play_cards(&mut self, card_ids: &[usize]) -> bool {
        if card_ids.is_empty() {
            // We can't pass before the first combination of a combination group is played
            return self.last_combination_type.is_some();
        }

        let card_values = card_ids
            .into_iter()
            .map(|&id| CardId(id).to_value())
            .collect();

        let current_combination_type = if let Some(bomb_rank) = is_bomb(&card_values) {
            Some(CombinationType::Bomb(bomb_rank))
        } else {
            is_valid_normal(&card_values).map(|normal_type| CombinationType::Normal(normal_type))
        };

        use CombinationType::*;
        match (&self.last_combination_type, &current_combination_type) {
            (_, None) => false,
            (Some(Bomb(last_bomb)), Some(Bomb(current_bomb))) => {
                if current_bomb <= last_bomb {
                    false
                } else {
                    true
                }
            }
            (Some(Bomb(_)), Some(Normal(_))) => false,
            (Some(Normal(last_normal)), Some(Normal(current_normal))) => {
                if let Some(_) = current_normal.has_higher_rank_than(last_normal) {
                    true
                } else {
                    false
                }
            }
            _ => true,
        }
    }

    // We pass if card_ids is empty
    pub fn play_cards(&mut self, card_ids: &[usize]) {
        if card_ids.is_empty() {
            self.capture_table();
        } else {
            let card_values = card_ids.iter().map(|&id| CardId(id).to_value()).collect();

            let current_combination_type = if let Some(bomb_rank) = is_bomb(&card_values) {
                Some(CombinationType::Bomb(bomb_rank))
            } else {
                is_valid_normal(&card_values)
                    .map(|normal_type| CombinationType::Normal(normal_type))
            };

            use CombinationType::*;
            self.last_combination_type =
                match (&self.last_combination_type, &current_combination_type) {
                    (_, None) => panic!("play_cards: No current combination type"),
                    (Some(Bomb(last_bomb)), Some(Bomb(current_bomb))) => {
                        if current_bomb <= last_bomb {
                            panic!("play_cards: Bomb rank too low");
                        } else {
                            current_combination_type
                        }
                    }
                    (Some(Bomb(_)), Some(Normal(_))) => {
                        panic!("play_cards: Tried to play normal combination after a bomb")
                    }
                    (Some(Normal(last_normal)), Some(Normal(current_normal))) => {
                        if let Some(new_normal_type) =
                            current_normal.has_higher_rank_than(last_normal)
                        {
                            Some(Normal(new_normal_type))
                        } else {
                            panic!(
                                "play_cards: Tried to play {:?} after {:?}",
                                current_normal, last_normal
                            );
                        }
                    }
                    _ => current_combination_type,
                };

            // move the cards to table
            for &card_id in card_ids {
                self.locations[card_id] = Location::Table {
                    order: self.next_order,
                    captured_by: None,
                    in_last_combination_before_pass: false,
                };
            }
            self.next_order += 1;
        }

        // change the current player
        self.current_player = self.current_player.other();
    }

    pub fn game_stage(&self) -> GameStage {
        if self.is_game_over() {
            return GameStage::GameOver;
        }
        if self.current_player == Player::Me {
            return GameStage::Play;
        } else {
            return GameStage::Wait;
        }
    }

    pub fn card_frontend_state(&self, card_id: usize) -> CardFrontendState {
        match self.locations[card_id] {
            Location::Haggis => CardFrontendState::Haggis,
            Location::Hand(Player::Opponent) => CardFrontendState::InOpponentHand,
            Location::Hand(Player::Me) => CardFrontendState::InMyHand,
            Location::Table {
                captured_by: None,
                order,
                ..
            } => {
                if order + 1 == self.next_order {
                    CardFrontendState::JustPlayed
                } else {
                    CardFrontendState::ThisCombinationGroup
                }
            }
            Location::Table {
                captured_by: Some(Player::Me),
                ..
            } => CardFrontendState::CapturedByMe,
            Location::Table {
                captured_by: Some(Player::Opponent),
                ..
            } => CardFrontendState::CapturedByOpponent,
        }
    }

    /// return (my_hand_size, opponent_hand_size)
    pub fn hand_sizes(&mut self) -> Box<[usize]> {
        let mut my_card_count = 0;
        let mut opponent_card_count = 0;

        for location in self.locations.iter() {
            match location {
                Location::Hand(Player::Me) => my_card_count += 1,
                Location::Hand(Player::Opponent) => opponent_card_count += 1,
                _ => {}
            };
        }

        Box::new([my_card_count, opponent_card_count])
    }

    /// return (my_score, opponent_score) based on the scores so far
    pub fn calculate_score(&mut self) -> Box<[usize]> {
        let hand_sizes = self.hand_sizes();
        let my_card_count = hand_sizes[0];
        let opponent_card_count = hand_sizes[1];

        if my_card_count == 0 || opponent_card_count == 0 {
            // Capture the last combination group
            // Because the winner of the last combination group was the last
            // person to play, self.play_cards will switch the current player
            // to their opponent, so self.capture_table will assign cards correctly
            self.capture_table();
        }
        let mut my_score = 0;
        let mut opponent_score = 0;

        let mut winner_of_hand_bonus = 0;

        // The winner of the hand scores 5 points for each card in her opponent's hand.
        // Remember, the wild cards count as part of the hand.
        winner_of_hand_bonus += 5 * (my_card_count + opponent_card_count);

        for (i, location) in self.locations.iter().enumerate() {
            let card_id = CardId(i);
            match location {
                // All point  cards (i.e., any 3, 5, 7, 9, J, Q, or K) captured
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
                Location::Hand(..) | Location::Haggis => {
                    winner_of_hand_bonus += card_id.to_value().point_value();
                }
                Location::Table {
                    captured_by: None, ..
                } => {
                    // This can happen if the game is still going
                }
            }
        }

        if my_card_count == 0 {
            my_score += winner_of_hand_bonus;
        } else if opponent_card_count == 0 {
            opponent_score += winner_of_hand_bonus;
        }

        Box::new([my_score, opponent_score])
    }

    pub fn am_player_1(&self) -> bool {
        self.me_went_first
    }
}

impl Game {
    pub fn read_qr_code(&mut self, image: DynamicImage) -> Result<(), &str> {
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

        let code = match codes.next() {
            Some(Ok(code)) => code,
            Some(_) => return Err("Cannot identify qr code"),
            _ => return Err("No qr code in image"),
        };
        let decoded = match code.decode() {
            Ok(decoded) => decoded,
            _ => return Err("Cannot decode qr code into bytes"),
        };

        *self = match decode_game(&decoded.payload) {
            Some(game) => game,
            None => return Err("Qr data is not a valid game"),
        };
        self.switch_perspective();

        Ok(())
    }

    pub fn write_qr_code(&self, width: usize, height: usize) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let encoded_game = encode_game(self);

        // Encode some data into bits.
        let code = QrCode::new(&encoded_game).unwrap();

        // Render the bits into an image.
        code.render()
            .max_dimensions(width as u32, height as u32)
            .dark_color(Rgba([0, 0, 0, 255]))
            .light_color(Rgba([217, 217, 217, 255]))
            .build()
    }

    /// Setup the location of each card at the beginning of a game
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

    pub fn is_game_over(&self) -> bool {
        let mut my_card_count = 0;
        let mut opponent_card_count = 0;

        for location in self.locations.iter() {
            match location {
                Location::Hand(Player::Me) => my_card_count += 1,
                Location::Hand(Player::Opponent) => opponent_card_count += 1,
                _ => {}
            };
        }

        my_card_count == 0 || opponent_card_count == 0
    }

    /// Capture the cards on the table without changing whose turn it is
    fn capture_table(&mut self) {
        // capture the cards on the table
        let player_who_captures = if let Some(CombinationType::Bomb(_)) = self.last_combination_type
        {
            self.current_player
        } else {
            self.current_player.other()
        };
        for location in &mut self.locations {
            if let Location::Table {
                captured_by,
                order,
                in_last_combination_before_pass,
            } = location
            {
                if captured_by.is_none() {
                    *captured_by = Some(player_who_captures);
                    if *order + 1 == self.next_order {
                        *in_last_combination_before_pass = true;
                    }
                }
            }
        }

        self.last_combination_type = None;
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
}
