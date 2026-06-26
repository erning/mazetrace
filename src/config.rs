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

    /// Maze generation algorithm to use.
    #[arg(long, value_enum, default_value_t = GeneratorAlgorithm::Dfs)]
    pub generator: GeneratorAlgorithm,

    /// Maze solving algorithm to use.
    #[arg(long, value_enum, default_value_t = SolverAlgorithm::Dfs)]
    pub solver: SolverAlgorithm,

    /// Deprecated alias for --solver.
    #[arg(long, value_enum)]
    pub algorithm: Option<SolverAlgorithm>,

    /// Start solving immediately after maze generation completes.
    #[arg(long)]
    pub auto_start: bool,

    /// Render with ASCII characters instead of Unicode line art.
    #[arg(long)]
    pub ascii: bool,

    /// Random seed used for maze generation.
    #[arg(long)]
    pub seed: Option<u64>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum GeneratorAlgorithm {
    Dfs,
    Prim,
    Kruskal,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum SolverAlgorithm {
    Dfs,
    Bfs,
    Astar,
    DeadEnd,
}

impl GeneratorAlgorithm {
    pub const fn label(self) -> &'static str {
        match self {
            GeneratorAlgorithm::Dfs => "DFS",
            GeneratorAlgorithm::Prim => "Prim",
            GeneratorAlgorithm::Kruskal => "Kruskal",
        }
    }
}

impl SolverAlgorithm {
    pub const fn label(self) -> &'static str {
        match self {
            SolverAlgorithm::Dfs => "DFS",
            SolverAlgorithm::Bfs => "BFS",
            SolverAlgorithm::Astar => "A*",
            SolverAlgorithm::DeadEnd => "Dead-End",
        }
    }
}

impl Config {
    pub fn normalized_speed(&self) -> u64 {
        self.speed.clamp(1, 240)
    }

    pub fn solver_algorithm(&self) -> SolverAlgorithm {
        self.algorithm.unwrap_or(self.solver)
    }
}
