use merlin::Transcript;
use rand::rngs::OsRng;
use rand::Rng;
use schnorrkel::vrf::{VRFInOut, VRFProof};
use schnorrkel::Keypair;
use std::collections::HashMap;

// Assuming 52 Card Deck
const DECK_SIZE: usize = 52;

struct Player {
    keypair: Keypair,
    hand: Option<i32>,
    proof: Option<VRFProof>,
    transcript: Transcript,
}

impl Player {
    fn new() -> Player {
        let mut csprng = OsRng;
        let mut seed = [0u8];
        let mut rng = rand::thread_rng();
        rng.fill(&mut seed[..]);
        let mut transcript = Transcript::new(b"new player");
        transcript.append_message(b"seed", &seed);

        Player {
            keypair: Keypair::generate_with(&mut csprng),
            hand: None,
            proof: None,
            transcript,
        }
    }

    fn random_function_card(&self, vrf_io: &VRFInOut) -> i32 {
        let card: i32 = vrf_io.as_output_bytes().iter().map(|&b| b as i32).sum();
        card % DECK_SIZE as i32
    }

    fn draw(&mut self) {
        let transcript = self.transcript.clone();
        let (vrf_io, vrf_proof, _) = self.keypair.vrf_sign(transcript);
        self.proof = Some(vrf_proof);
        self.hand = Some(self.random_function_card(&vrf_io));
    }

    fn reveal_card(&self) -> Option<(i32, VRFProof)> {
        Some((self.hand.unwrap(), self.proof.as_ref().unwrap().clone()))
    }

    fn verify_card(&self, vrf_proof: VRFProof) -> i32 {
        let transcript = self.transcript.clone();
        let (vrf_io, _, _) = self.keypair.vrf_sign(transcript);
        let transcript = self.transcript.clone();
        let vrf_io = self
            .keypair
            .public
            .vrf_verify(transcript, &vrf_io.to_preout(), &vrf_proof)
            .unwrap()
            .0;
        self.random_function_card(&vrf_io)
    }
}

pub struct PokerGame {
    players: HashMap<u8, Player>,
}

impl PokerGame {
    pub fn new() -> PokerGame {
        PokerGame {
            players: HashMap::new(),
        }
    }

    pub fn add_player(&mut self, id: u8) {
        self.players.insert(id, Player::new());
    }

    pub fn draw(&mut self, id: u8) {
        if let Some(player) = self.players.get_mut(&id) {
            player.draw();
        }
    }

    pub fn reveal_card_game(&self, player_id: u8) -> Option<(i32, VRFProof)> {
        let player = &self.players[&player_id];
        if let Some((card, proof)) = player.reveal_card() {
            println!(
                "Player {} reveals card {:?} with proof {:?}",
                player_id, card, proof
            );
            return Some((card, proof));
        }
        println!("Invalid proof.");
        None
    }

    pub fn verify_card_game(&self, player_id: u8, vrf_proof: VRFProof) {
        let player = &self.players[&player_id];
        let card = player.verify_card(vrf_proof);
        println!("Player {} verify card is {:?} ", player_id, card);
    }

    pub fn play(&mut self) {
        let players_keys = self.players.keys().cloned().collect::<Vec<u8>>();
        for id in players_keys {
            self.draw(id);
        }

        let mut highest_card = 0;
        let mut player_id = 0;
        let players_keys = self.players.keys().cloned().collect::<Vec<u8>>();
        for id in players_keys {
            let (card, vrf_proof) = self.reveal_card_game(id).unwrap();
            self.verify_card_game(id, vrf_proof);

            if card > highest_card {
                highest_card = card;
                player_id = id;
            }
        }
        println!(
            "Player {} is the winner with card {} ",
            player_id, highest_card
        );
    }
}

