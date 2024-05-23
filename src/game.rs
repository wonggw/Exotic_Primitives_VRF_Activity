use std::io::stdin;
use sp_core::*;
use rand::Rng;
use schnorrkel::{
    Keypair, signing_context,
};
use sha2::{Sha256, Digest};
use crate::try_draw;
#[derive(Debug)]
pub struct Player {
    pub keypair: Keypair,
    cards: Vec<(u16, [u8; 97])>,
    pub balance: i32,
}

impl Player {
    pub fn new(keypair: Keypair, balance: i32) -> Self {
        Self {
            keypair,
            cards: vec![],
            balance,
        }
    }
    pub fn hand_card(&mut self, cards: Vec<(u16, [u8; 97])>) {
        self.cards = cards;
    }
    pub fn get_balance(&self) -> i32 {
        self.balance
    }
}

pub mod game_util {
    use super::*;

    pub fn sign_message(players: &Vec<Player>, message: &[u8]) -> Vec<u8> {
        let ctx = signing_context(message);
        players.iter().fold(Vec::new(), |mut byte, player| {
            let mut signature_bytes = player.keypair.sign(ctx.bytes(message)).to_bytes().to_vec();
            byte.append(&mut signature_bytes);
            byte
        })
    }

    pub fn hash_signatures(signatures: Vec<u8>) -> [u8; 16] {
        sp_core::blake2_128(&signatures)
    }

    pub fn create_players(count: i32) -> Vec<Player> {
        let mut csprng = rand_core::OsRng;
        (0..count)
            .map(|_| Player::new(Keypair::generate_with(&mut csprng), 1000))
            .collect()
    }
}

pub mod game_handler {
    use super::*;

    fn wait() {
        println!("Press enter to continue...");
        stdin().read_line(&mut String::new()).expect("error reading line");
    }

    fn bid(players: &mut [Player], bank: &mut i32) {
        players.iter_mut().for_each(|player| {
            let bid = rand::thread_rng().gen_range(0..301);
            player.balance -= bid;
            println!(
                "Player with key {:?} made a bid of {}",
                player.keypair.public.to_bytes(), bid
            );
            *bank += bid;
        });
    }

    pub fn run() {
        println!("Game starts!");
        let mut input: String = String::new();
        println!("Enter the number of players");

        stdin().read_line(&mut input).expect("Reading string error");
        input = input.replace('\n', "");
        let n: i32 = input.parse().unwrap();

        println!("There {} player(s) with $1000 each", n);

        let mut players = game_util::create_players(n);

        // Let each player sign something
        let message: &[u8] = b"I join the table";
        let signatures = game_util::sign_message(&players, message);
        let vrf_seed = game_util::hash_signatures(signatures);

        // Give each player 2 cards
        players.iter_mut().for_each(|player| {
            let cards: Vec<(u16, [u8; 97])> = (0..2)
                .filter_map(|i| try_draw(&player.keypair, &vrf_seed, i))
                .collect();
            player.hand_card(cards);
        });

        let mut bank = 0;
        println!("Players are given 2 cards each");
        wait();
        bid(&mut players, &mut bank);

        // More game logic here such as revealing cards...
    }

}
