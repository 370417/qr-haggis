trait Game {
    /// cards is a set of cards
    fn play_cards(&mut self, cards: ()) -> error;

    fn is_game_over(&self) -> bool;

    fn get_hand(&self) -> cards;

    fn create_state(qr_code: Option<QRCode>) -> Self;

    fn set_state(&mut self, qr_code: Option<QRCode>);

    fn get_table(&self) -> cards;
}
