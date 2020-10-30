use std::collections::HashMap;

use crate::game::constant::*;
use crate::game::{Game, Location, Player};
use num_bigint::BigUint;

const GROUPING_ARRAY_BYTE_LEN: usize = (2 * (DECK_SIZE - HAGGIS_SIZE) + 7) / 8;
const CARD_ORDER_BYTE_LEN: usize = 20;

fn compress_card_order(card_order_goal: &[usize]) -> BigUint {
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

    curr_card_order
}

// bit == 0 means the first bit (head of the combination),
// bit == 1 means the second bit (head of the group of combinations)
// card_idx is relative to the cards on table, not including the cards in hand
fn set_1_for_grouping_array(grouping_array: &mut u128, card_idx: usize, bit: usize) {
    let bit_idx = 2 * card_idx + bit;
    *grouping_array = *grouping_array | 1 << bit_idx;
}

// Standard card order when sending a qr code:
// - my hand
// - opponent's hand
// - group of combinations
// - next group of combinations, after a player passed
// - ...
fn compress_game(game: &Game) -> Vec<u8> {
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
    // The first bit is 1 iff the card is the first card of its combination
    // The second bit is 1 iff the card is the first card of its combination group
    // (ie start of game, or the other player just passed)
    let mut grouping_array = 0_u128;
    let mut card_order = [0; DECK_SIZE - HAGGIS_SIZE];

    for (i, card_id) in my_hand.iter().enumerate() {
        card_order[i] = *card_id;
    }

    for (i, card_id) in opponent_hand.iter().enumerate() {
        card_order[my_hand_size + i] = *card_id;
    }

    //u128, u64, or u32 or u8
    // u128 & 1 << 8 == u8[14] & 1 << 0
    // u128 & 1 << 16 == u8[13] & 1 << 0
    // u128 => u8[16]
    let mut prev_captured_by = None;
    let mut i: usize = 0;
    for combination_idx in 0.. {
        match cards_on_table.get(&combination_idx) {
            Some(combination) => {
                let curr_captured_by = game.locations[combination[0]].captured_by();
                if curr_captured_by != prev_captured_by {
                    set_1_for_grouping_array(&mut grouping_array, i, 1);
                    prev_captured_by = curr_captured_by;
                }

                set_1_for_grouping_array(&mut grouping_array, i, 0);
                for card_id in combination {
                    card_order[i + my_hand_size + opponent_hand_size] = *card_id;
                    i += 1;
                }
            }
            None => break,
        }
    }

    let size_of_u128 = std::mem::size_of::<u128>();
    let grouping_array_bytes =
        &grouping_array.to_be_bytes()[size_of_u128 - GROUPING_ARRAY_BYTE_LEN..size_of_u128];

    let compressed_card_order = compress_card_order(&card_order);
    let mut card_order_bytes = compressed_card_order.to_bytes_be();
    while card_order_bytes.len() < CARD_ORDER_BYTE_LEN {
        card_order_bytes.insert(0, 0);
    }

    let mut compressed_game = Vec::new();

    // output vec:
    // compressed_card_order as bytes (20 bytes),
    // hand sizes (2 bytes),
    // grouping array (33 elements, 9 bytes)
    // Player::Me went first bool (1 byte)
    compressed_game.append(&mut card_order_bytes);
    compressed_game.push(my_hand_size as u8);
    compressed_game.push(opponent_hand_size as u8);
    compressed_game.append(&mut grouping_array_bytes.to_vec());
    compressed_game.push(game.me_went_first as u8);

    compressed_game
}

fn decompress_game(compressed_game: &[u8]) -> Game {
    let card_order_bytes = &compressed_game[0..CARD_ORDER_BYTE_LEN];
    let opponent_hand_size = compressed_game[CARD_ORDER_BYTE_LEN] as usize;
    let my_hand_size = compressed_game[CARD_ORDER_BYTE_LEN + 1] as usize;
    let grouping_array_bytes = &compressed_game
        [CARD_ORDER_BYTE_LEN + 2..CARD_ORDER_BYTE_LEN + 2 + GROUPING_ARRAY_BYTE_LEN];
    let me_went_first = *compressed_game.last().unwrap() == 0;

    let compressed_card_order = BigUint::from_bytes_be(card_order_bytes);
    let card_order = decompress_card_order(compressed_card_order);

    let mut fixed_grouping_array_bytes = [0; 16];
    let num_zeroes = 16 - grouping_array_bytes.len();
    for (i, grouping_byte) in grouping_array_bytes.iter().enumerate() {
        fixed_grouping_array_bytes[i + num_zeroes] = *grouping_byte;
    }
    let grouping_array = u128::from_be_bytes(fixed_grouping_array_bytes);

    let mut game = Game {
        locations: Vec::new(),
        current_player: Player::Me,
        me_went_first,
        last_trick: Vec::new(),
        last_trick_type: None,
        current_start_order: 0,
    };
    for _ in 0..DECK_SIZE {
        game.locations.push(Location::Haggis);
    }

    for i in 0..opponent_hand_size {
        let card_id = card_order[i];
        game.locations[card_id] = Location::Hand(Player::Opponent);
    }

    for i in opponent_hand_size..(my_hand_size + opponent_hand_size) {
        let card_id = card_order[i];
        game.locations[card_id] = Location::Hand(Player::Me);
    }

    for combination_idx in 0.. {}

    todo!();
}

mod test {
    use super::*;
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    #[test]
    fn test_compress_decompress() {
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
}
