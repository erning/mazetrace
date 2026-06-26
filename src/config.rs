use clap::{Parser, ValueEnum};

#[derive(Clone, Debug, Parser)]
#[command(author, version, about)]
pub struct Config {
    /// Maze width in logical cells.
    #[arg(long)]
    pub width: Option<usize>,

    /// Maze height in logical cells.
    #[arg(long)]
    pub height: Option<usize>,

    /// Animation speed in steps per second.
    #[arg(long, default_value_t = 60)]
    pub speed: u64,

    /// Exploration algorithm to use.
    #[arg(long, value_enum, default_value_t = Algorithm::Dfs)]
    pub algorithm: Algorithm,

    /// Render with ASCII characters instead of Unicode line art.
    #[arg(long)]
    pub ascii: bool,

    /// Random seed used for maze generation.
    #[arg(long)]
    pub seed: Option<u64>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum Algorithm {
    Dfs,
}

impl Config {
    pub fn normalized_speed(&self) -> u64 {
        self.speed.clamp(1, 240)
    }
}
