extern crate schnorrkel;
use merlin::Transcript;
use schnorrkel::{Keypair, PublicKey, vrf::{VRFInOut, VRFPreOut, VRFProof}};

use crate::game::game_handler;

const NUM_DRAWS : u8 = 5;
const NUM_CARDS : u16 = 52;
mod game;

fn main() {
    println!("Start game");
    game_handler::run();
}

fn draw_transcript(seed: &[u8; 16], draw_num: u8) -> Option<Transcript> {
    if draw_num > NUM_DRAWS { 
        return None; 
    }
    let mut t = Transcript::new(b"Card Draw Transcript");
    t.append_message(b"seed", seed);
    t.append_u64(b"draw", draw_num as u64);
    Some(t)
}

fn find_card(io: &VRFInOut) -> Option<u16> {
    let b: [u8; 8] = io.make_bytes(b"card");
    Some((u64::from_le_bytes(b) % (NUM_CARDS as u64)) as u16)
}

fn try_draw( keypair: &Keypair, seed: &[u8; 16], draw_num: u8) -> Option<(u16, [u8; 97])> {
    let t = draw_transcript(seed, draw_num)?;
    let (io, proof, _) = keypair.vrf_sign(t);
    let card = find_card(&io)?;
    let mut vrf_signature = [0u8; 97];
    vrf_signature[..32].copy_from_slice(&io.to_preout().to_bytes()[..]);
    vrf_signature[32..96].copy_from_slice(&proof.to_bytes()[..]);
    vrf_signature[96] = draw_num;
    Some((card, vrf_signature))
}

/// Draws all our cards for the give seed
fn draw_cards(keypair: &Keypair, seed: &[u8; 16]) -> Vec<(u16, [u8; 97])> {
    (0..NUM_DRAWS).filter_map(|i| try_draw(keypair, seed, i)).collect()
}

fn verify_card_play(public: &PublicKey, vrf_signature: &[u8; 97], seed: &[u8; 16]) -> Option<u16> {
    let t = draw_transcript(seed, vrf_signature[96]) ?;
    let out = VRFPreOut::from_bytes(&vrf_signature[..32]).ok() ?;
    let proof = VRFProof::from_bytes(&vrf_signature[32..96]).ok() ?;
    let (io, _) = public.vrf_verify(t, &out, &proof).ok() ?;
    find_card(&io)
}