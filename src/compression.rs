use crate::game::constant::*;
use crate::game::Game;
use num_bigint::BigUint;

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
