mod game;

fn main() {
    
}

enum TrickType {
    Normal {
        set_size: usize,
        sequence_size: usize,
    },
    Bomb(Vec<Card>),
}

/// Precondition: the cards in trick are in order
fn trick_type(trick: &Vec<Card>) -> Option<TrickType> {
    /*
    0. Sort the cards by rank and suit.
    1. Check for bombs:
        If 
    */
    if trick.is_empty() {
        return None;
    }

    let first_card = &trick[0];

    for i in 1..trick.len() {
        // check for different rank
        let card = &trick[i];
        if card.rank != first_card.rank {
            let sequence_size = trick.len() / i;
            return sequence_type(trick, sequence_size);
        }
    }

    // All the cards have the same rank
    Some(TrickType::Normal {
        set_size: trick.len(),
        sequence_size: 1,
    })
}

/// Return the type of the trick, assuming trick is a sequence.
/// Suits and ranks must match
fn sequence_type(trick: &Vec<Card>, sequence_size: usize) -> Option<TrickType> {
    let set_size = trick.len() / sequence_size;
    let smallest_rank = trick[0].rank;
    for i in 0..sequence_size {
        for j in 0..set_size {
            let current_card = &trick[j + i * set_size];
            let suit_card = &trick[j];
            if current_card.suit != suit_card.suit {
                // unless wildcard
                return None;
            }
            // if current_card.rank != 
        }
    }
    todo!()
}

