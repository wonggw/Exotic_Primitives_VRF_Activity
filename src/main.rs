mod game;

fn main() {
    let mut game = game::PokerGame::new();

    // Adding 4 players
    game.add_player(1);
    game.add_player(2);
    game.add_player(3);
    game.add_player(4);

    game.play();
}