trait Game {
    /// cards is a set of cards
    fn play_cards(&mut self, cards: ()) -> error;

    fn is_game_over(&self) -> bool;

    fn get_hand(&self) -> cards;

    fn create_state(qr_code: Option<QRCode>) -> Self;

    fn set_state(&mut self, qr_code: Option<QRCode>);

    fn get_table(&self) -> cards;
}

// A thought I had about Game::set_state:
// The only time we will ever call game.set_state(None) is from inside
// game.create_state(None).
// So how about we move that code into create_state? (or into an init_state function)
// Then we can get rid of the Option and match from set_state.
// And I think we should avoid exposing an internal-only operation to a public API.

// The game struct doesn't actually need to store the current player, right?
// You talked about how it will always be Player::Me.
// But I think we do need to store the order number of the first and last combination
// played this trick. Storing the last order number is just convenient, but the first
// one really matters because we need to distinguish between these two situations:
//      Player A plays [5], Player B passes, Player A plays [6]
//      Player A plays [5], Player B plays [6]
// Actually, what do we do if it is the first turn of the game, or the opponent just passed?
// How should we store that information?
