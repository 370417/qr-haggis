use std::collections::HashMap;

use crate::game::{constant::*, CardId};
use crate::game::{Game, Location, Player};
use num_bigint::BigUint;

const GROUPING_ARRAY_BYTE_LEN: usize = (2 * (DECK_SIZE - HAGGIS_SIZE) + 7) / 8;
const CARD_ORDER_BYTE_LEN: usize = 20;

fn compress_card_order(card_order_goal: &[usize]) -> BigUint {
    assert!(card_order_goal.len() == DECK_SIZE - HAGGIS_SIZE);
    println!("from compress_card_order: {:?}", card_order_goal);

    let mut compressed = BigUint::new(vec![0]);

    let mut curr_card_order: Vec<usize> = (0..DECK_SIZE).collect(); // can be removed
    let mut card_value_to_index: Vec<usize> = (0..DECK_SIZE).collect();

    let mut distances = Vec::new();

    for i in 0..(DECK_SIZE - HAGGIS_SIZE) {
        let card_value_goal = card_order_goal[i];
        // We want to swap two cards: (let x = curr_card_order[i]) and card_order_goal[i]
        // So we need to find card_order_goal[i] in curr_card_order
        // And then when we update card_value_to_index, we need to update the index x
        let j = card_value_to_index[card_value_goal];
        curr_card_order.swap(i, j); // can be removed
        card_value_to_index[curr_card_order[j]] = j;
        distances.push(j - i);
    }

    while let Some(distance) = distances.pop() {
        let card_possibilities = DECK_SIZE - distances.len();
        compressed = BigUint::new(vec![distance as u32]) + (card_possibilities * compressed);
    }

    compressed
}

fn decompress_card_order(mut compressed: BigUint) -> Vec<usize> {
    let mut card_possibilities: u32 = DECK_SIZE as u32;

    let mut curr_card_order: Vec<usize> = (0..DECK_SIZE).collect();

    for i in 0..(DECK_SIZE - HAGGIS_SIZE) {
        let distance = (&compressed % card_possibilities).to_u32_digits();
        let distance = if distance.len() == 0 {
            0
        } else {
            distance[0] as usize
        };
        compressed /= card_possibilities;
        card_possibilities -= 1;

        curr_card_order.swap(i, i + distance);
    }

    curr_card_order.truncate(DECK_SIZE - HAGGIS_SIZE);
    curr_card_order
}

// bit == 0 means the first bit (head of the combination),
// bit == 1 means the second bit (head of the group of combinations)
// grouping_array_idx is relative to the cards on table, not including the cards in hand
fn set_1_for_grouping_array(grouping_array: &mut u128, grouping_array_idx: usize, bit: usize) {
    let bit_idx = 2 * grouping_array_idx + bit;
    *grouping_array = *grouping_array | 1 << bit_idx;
}

fn read_bit_from_grouping_array(
    grouping_array: &u128,
    grouping_array_idx: usize,
    bit: usize,
) -> bool {
    let bit_idx = 2 * grouping_array_idx + bit;
    (*grouping_array & 1 << bit_idx) > 0
}

// Standard card order when sending a qr code:
// - my hand
// - opponent's hand
// - group of combinations
// - next group of combinations, after a player passed
// - ...
pub(crate) fn encode_game(game: &Game) -> Vec<u8> {
    println!("In encode_game. ");
    let mut my_hand = Vec::new();
    let mut opponent_hand = Vec::new();
    let mut cards_on_table: HashMap<usize, Vec<usize>> = HashMap::new();
    for (card_id, location) in game.locations.iter().enumerate() {
        match location {
            Location::Hand(Player::Me) => my_hand.push(card_id),
            Location::Hand(Player::Opponent) => opponent_hand.push(card_id),
            Location::Table { order, .. } => {
                cards_on_table.entry(*order).or_default().push(card_id)
            }
            Location::Haggis => {}
        }
    }

    let my_hand_size = my_hand.len();
    let opponent_hand_size = opponent_hand.len();

    // Each card gets 2 bits
    // The first bit is 1 iff the card is the last card of its combination
    // The second bit is 1 iff the card is the last card of its combination group
    let mut grouping_array = 0_u128;
    let mut card_order = [0; DECK_SIZE - HAGGIS_SIZE];

    //Card order: cards_in_my_hand, cards_in_opponents_hand, cards_on_the_table_in_order

    for (i, card_id) in my_hand.iter().enumerate() {
        card_order[i] = *card_id;
    }

    for (i, card_id) in opponent_hand.iter().enumerate() {
        card_order[my_hand_size + i] = *card_id;
    }

    // u128 => u8[16]
    let mut prev_captured_by = None;
    let mut i: usize = 0;
    for combination_idx in 0.. {
        match cards_on_table.get(&combination_idx) {
            Some(combination) => {
                let curr_captured_by = game.locations[combination[0]].captured_by();
                if curr_captured_by != prev_captured_by {
                    if i > 0 {
                        // mark the end of a group of combinations
                        set_1_for_grouping_array(&mut grouping_array, i - 1, 1);
                    }
                    prev_captured_by = curr_captured_by;
                }

                // mark the end of a combination
                set_1_for_grouping_array(&mut grouping_array, i + combination.len() - 1, 0);
                for card_id in combination {
                    card_order[i + my_hand_size + opponent_hand_size] = *card_id;
                    i += 1;
                }
            }
            None => {
                let last_combination_idx = combination_idx - 1;
                let combination = &cards_on_table[&last_combination_idx];
                let curr_captured_by = game.locations[combination[0]].captured_by();
                if curr_captured_by.is_some() {
                    set_1_for_grouping_array(&mut grouping_array, i + combination.len() - 1, 1);
                }
                break;
            }
        }
    }

    let size_of_u128 = std::mem::size_of::<u128>();
    let grouping_array_bytes =
        &grouping_array.to_be_bytes()[size_of_u128 - GROUPING_ARRAY_BYTE_LEN..size_of_u128];

    let compressed_card_order = compress_card_order(&card_order);
    let mut compressed_card_order_bytes = compressed_card_order.to_bytes_be();
    while compressed_card_order_bytes.len() < CARD_ORDER_BYTE_LEN {
        compressed_card_order_bytes.insert(0, 0);
    }

    let mut compressed_game = Vec::new();

    // output vec:
    // compressed_card_order as bytes (20 bytes),
    // hand sizes (2 bytes),
    // grouping array (33 elements, 9 bytes)
    // Player::Me went first bool (1 byte)
    compressed_game.append(&mut compressed_card_order_bytes);
    compressed_game.push(my_hand_size as u8);
    compressed_game.push(opponent_hand_size as u8);
    compressed_game.append(&mut grouping_array_bytes.to_vec());
    compressed_game.push(game.me_went_first as u8);

    println!("Card order for compression: {:?}", card_order);
    println!("Compressed card order: {:?}", compressed_card_order);
    println!("Compressed grouping array: 0b{:b}", grouping_array);

    compressed_game
}

pub(crate) fn decode_game(compressed_game: &[u8]) -> Game {
    println!("In decode_game. ");
    // Separate compressed game into sections
    let card_order_bytes = &compressed_game[0..CARD_ORDER_BYTE_LEN];
    let my_hand_size = compressed_game[CARD_ORDER_BYTE_LEN] as usize;
    let opponent_hand_size = compressed_game[CARD_ORDER_BYTE_LEN + 1] as usize;
    let grouping_array_bytes = &compressed_game
        [CARD_ORDER_BYTE_LEN + 2..CARD_ORDER_BYTE_LEN + 2 + GROUPING_ARRAY_BYTE_LEN];
    let me_went_first = *compressed_game.last().unwrap() != 0;

    // Recover the big int from the slice and decompress it
    let compressed_card_order = BigUint::from_bytes_be(card_order_bytes);
    println!("Compressed card order: {:?}", compressed_card_order);
    let card_order = decompress_card_order(compressed_card_order);
    println!("Card order after decompress: {:?}", card_order);

    let net_hand_size = my_hand_size + opponent_hand_size;

    // Store grouping_array_bytes into a fixed size array so that we can pad the left with zeros
    let mut fixed_grouping_array_bytes = [0; 16];
    let num_zeroes = 16 - grouping_array_bytes.len();
    for (i, grouping_byte) in grouping_array_bytes.iter().enumerate() {
        fixed_grouping_array_bytes[i + num_zeroes] = *grouping_byte;
    }
    let grouping_array = u128::from_be_bytes(fixed_grouping_array_bytes);
    println!("Decompressed Grouping array: 0b{:b}", grouping_array);

    //create the game struct with the informations given above
    let mut game = Game {
        locations: Vec::new(),
        current_player: if me_went_first {
            Player::Me
        } else {
            Player::Opponent
        },
        me_went_first,
        last_trick: Vec::new(),
        last_trick_type: None,
        current_start_order: 0,
    };
    for _ in 0..DECK_SIZE {
        //default is haggis
        game.locations.push(Location::Haggis);
    }

    println!(
        "decompressing: opponent_hand_size {}, my_hand_size {}",
        opponent_hand_size, my_hand_size
    );

    //my hand
    for i in 0..my_hand_size {
        let card_id = card_order[i];
        game.locations[card_id] = Location::Hand(Player::Me);
    }

    //opponents hand
    for i in my_hand_size..net_hand_size {
        let card_id = card_order[i];
        game.locations[card_id] = Location::Hand(Player::Opponent);
    }

    //using grouping array to parse cards on the table, also replay the game at the same time
    let num_cards_on_table = DECK_SIZE - HAGGIS_SIZE - net_hand_size;

    let mut combination: Vec<CardId> = vec![CardId(card_order[net_hand_size])];
    for grouping_array_idx in 1..num_cards_on_table {
        let card_id = CardId(card_order[net_hand_size + grouping_array_idx]);
        combination.push(card_id);
        let is_last_card_of_combination =
            read_bit_from_grouping_array(&grouping_array, grouping_array_idx, 0);
        let is_last_card_of_combination_group =
            read_bit_from_grouping_array(&grouping_array, grouping_array_idx, 1);
        if is_last_card_of_combination {
            let success = game.play_cards(combination);
            if !success {
                panic!("Play cards unsuccessful")
            }
            combination = Vec::new();
        }
        if is_last_card_of_combination_group {
            game.pass();
        }
    }

    println!("decode_game: {:?}", game);

    game
}

mod test {
    use super::*;
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    #[test]
    fn test_card_order_compress_decompress() {
        let mut rng = thread_rng();
        let mut card_order_goal: Vec<usize> = (0..DECK_SIZE).collect();
        card_order_goal.shuffle(&mut rng);
        println!("{:?}", card_order_goal);
        let card_order_result = decompress_card_order(compress_card_order(&card_order_goal));
        assert_eq!(
            &card_order_goal[0..(DECK_SIZE - HAGGIS_SIZE)],
            &card_order_result[0..(DECK_SIZE - HAGGIS_SIZE)]
        );
    }

    #[test]
    fn test_10000_card_order_compress_decompress() {
        for _ in 0..10000 {
            test_card_order_compress_decompress();
        }
    }

    #[test]
    fn test_init_game_compress_decompress() {
        let game = Game::create_state(None);
        // let opponent_result = decode_game(&encode_game(&game));
        // println!("{:?}", opponent_result);
        let my_result = decode_game(&encode_game(&game));
        assert_eq!(game, my_result);
    }

    fn print_game(game: &Game) {
        println!(
            "My hand: {:?}",
            game.get_hand(Player::Me)
                .iter()
                .map(|card_id| card_id.0)
                .collect::<Vec<_>>()
        );
        println!(
            "Opponent's hand: {:?}",
            game.get_hand(Player::Opponent)
                .iter()
                .map(|card_id| card_id.0)
                .collect::<Vec<_>>()
        );
        println!(
            "Last trick: {:?}",
            game.last_trick
                .iter()
                .map(|card_id| card_id.0)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_rand_game_encode_decode() {
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
            last_trick: vec![],
            last_trick_type: None,
            current_start_order: 0,
        };

        game.play_cards(vec![CardId(11), CardId(12), CardId(13)]);

        print_game(&game);

        game.pass();

        print_game(&game);

        game.play_cards(vec![CardId(15)]);
        game.play_cards(vec![CardId(9)]);

        print_game(&game);

        let my_result = decode_game(&encode_game(&game));
        assert_eq!(game, my_result);
    }

    #[test]
    fn test_preserve_last_player_pass() {
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
            last_trick: vec![],
            last_trick_type: None,
            current_start_order: 0,
        };

        game.play_cards(vec![CardId(11), CardId(12), CardId(13)]);

        assert_eq!(game, decode_game(&encode_game(&game)));

        println!("Decoding and encoding worked without passing!");

        game.pass();

        assert_eq!(game, decode_game(&encode_game(&game)));
    }
}
