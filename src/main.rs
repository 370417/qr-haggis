mod card;
mod game;
// mod quirc_bindings;

use game::Game;

fn main() {
    let game = Game::create_state(None);

    let serialized = ron::to_string(&game);

    println!("{}", serialized);
}
