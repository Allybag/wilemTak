use alpha_tak::{Network, Player};
use rand::{prelude::SliceRandom, thread_rng};
use tak::*;

const PIT_GAMES: u32 = 50;
const BATCH_SIZE: u32 = 16;
const ROLLOUTS: u32 = 50;

const RANDOM_PLIES: u32 = 2;

const NOISE_ALPHA: f32 = 0.4;
const NOISE_RATIO: f32 = 0.5;
const NOISE_PLIES: u16 = 30;

pub fn pit<const N: usize, NET: Network<N>>(new: &NET, old: &NET) -> PitResult {
    let mut result = PitResult::default();

    let mut rng = thread_rng();
    for i in 0..PIT_GAMES {
        println!("pit game {i}/{PIT_GAMES}");
        let mut opening = Vec::new();
        for color in [Color::White, Color::Black] {
            let mut game = Game::with_komi(2);

            let mut new_player = Player::new(new, BATCH_SIZE, false, false, &game);
            let mut old_player = Player::new(old, BATCH_SIZE, false, false, &game);

            // Do random opening.
            if opening.is_empty() {
                for _ in 0..RANDOM_PLIES {
                    let my_move = *game.possible_moves().choose(&mut rng).unwrap();
                    opening.push(my_move);
                    new_player.play_move(my_move, &game, false);
                    old_player.play_move(my_move, &game, false);
                    game.play(my_move).unwrap();
                }
                println!(
                    "opening: {:?}",
                    opening.iter().map(Move::to_string).collect::<Vec<_>>()
                );
            } else {
                for &my_move in opening.iter() {
                    new_player.play_move(my_move, &game, false);
                    old_player.play_move(my_move, &game, false);
                    game.play(my_move).unwrap();
                }
            }

            while game.result() == GameResult::Ongoing {
                let to_move = if game.to_move == color {
                    &mut new_player
                } else {
                    &mut old_player
                };
                if game.ply < NOISE_PLIES {
                    to_move.add_noise(NOISE_ALPHA, NOISE_RATIO, &game);
                }
                for _ in 0..ROLLOUTS {
                    to_move.rollout(&game);
                }
                let my_move = to_move.pick_move(true);
                new_player.play_move(my_move, &game, true);
                old_player.play_move(my_move, &game, true);
                game.play(my_move).unwrap();
            }
            println!("{:?} in {} plies as {color}", game.result(), game.ply);

            result.update(game.result(), color);
        }
    }

    result
}

#[derive(Debug, Default)]
pub struct PitResult {
    wins: u32,
    losses: u32,
    draws: u32,
}

impl PitResult {
    pub fn win_rate(&self) -> f64 {
        // another option:
        // (self.wins as f64 + self.draws as f64 / 2.) /
        // (self.wins + self.draws + self.losses) as f64
        self.wins as f64 / (self.wins + self.losses) as f64
    }

    fn update(&mut self, result: GameResult, color: Color) {
        match result {
            GameResult::Winner { color: winner, .. } => {
                if winner == color {
                    self.wins += 1
                } else {
                    self.losses += 1
                }
            }
            GameResult::Draw { .. } => self.draws += 1,
            GameResult::Ongoing => {}
        }
    }
}
