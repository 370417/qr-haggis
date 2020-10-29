use crate::game::Game;
use num_bigint::BigUint;

fn compress_card_order(card_order_goal: &[usize]) -> BigUint {
    let mut compressed = BigUint::new(vec![0]);

    let mut curr_card_order: Vec<usize> = (0..42).collect(); // can be removed
    let mut search_helper: Vec<usize> = (0..42).collect();

    let mut distances = Vec::new();

    for i in 0..36 {
        let j = search_helper[i];
        curr_card_order.swap(i, j); // can be removed
        search_helper[i] = j;

        distances.push(j - i);
    }

    while let Some(distance) = distances.pop() {
        let card_possibilities = 42 - distances.len();
        compressed = BigUint::new(vec![distance as u32]) + (card_possibilities * compressed);
    }

    compressed
}

fn decompress_card_order(mut compressed: BigUint) -> Vec<usize> {
    let mut card_possibilities: u32 = 42;

    let mut curr_card_order: Vec<usize> = (0..42).collect();

    for i in 0..36 {
        let distance: BigUint = &compressed % card_possibilities;
        let distance = distance.to_u32_digits()[0] as usize;
        compressed /= card_possibilities;
        card_possibilities -= 1;

        curr_card_order.swap(i, i + distance);
    }

    curr_card_order
}
