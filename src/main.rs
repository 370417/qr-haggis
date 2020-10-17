use rand::prelude::*;

fn main() {
    let mut game = Game::new();
    let card = game.hand1.remove(0);
    game.play(card);
    println!("{:#?}", game);
}

#[derive(Debug)]
struct Card {
    // 2 through 10 + Jack, Queen, King
    rank: u32,
    // 0, 1, 2, 3
    suit: u32,
}

enum TrickType {
    Normal {
        set_size: usize,
        sequence_size: usize,
    },
    Bomb(Vec<Card>),
}

#[derive(Debug)]
struct Game {
    haggis: Vec<Card>,
    hand1: Vec<Card>,
    hand2: Vec<Card>,
    turn: Player,
    table: Vec<Vec<Card>>,
    captured1: Vec<Card>,
    captured2: Vec<Card>,
}

#[derive(Debug)]
enum Player {
    Player1,
    Player2,
}

/// Create a deck without face cards
fn create_deck() -> Vec<Card> {
    let mut deck = Vec::new();
    for suit in 0..4 {
        for rank in 2..=13 {
            deck.push(Card {
                rank,
                suit,
            });
        }
    }
    deck
}

impl Game {
    fn new() -> Game {
        let mut deck = create_deck();
        deck.shuffle(&mut rand::thread_rng());
        let mut hands = deck.split_off(8);
        let hand2 = hands.split_off(14);
        Game {
            haggis: deck, // put the first eight cards here
            hand1: hands, // 14 cards here
            hand2: hand2, // 14 cards here
            turn: Player::Player1,
            table: Vec::new(),
            captured1: Vec::new(),
            captured2: Vec::new(),
        }
    }

    fn play(&mut self, card: Card) {
        self.table.push(vec![card]);
    }
}

/// Precondition: the cards in trick are in order
fn trick_type(trick: &Vec<Card>) -> Option<TrickType> {
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
