mod board;
mod game;
mod moves;
mod reserves;

#[cfg(test)]
mod tests;

pub use game::{game_repr, input_channels};
pub use moves::{output_size, possible_moves};
