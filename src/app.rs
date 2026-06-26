use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rand::random;

use crate::config::{Config, GeneratorAlgorithm, SolverAlgorithm};
use crate::explorer::{ExplorationStatus, Explorer};
use crate::generator::MazeGenerator;
use crate::maze::Maze;
use crate::render::{render_size, RenderPhase};

const STATUS_HEIGHT: u16 = 3;
const MIN_WIDTH: usize = 5;
const MIN_HEIGHT: usize = 5;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Phase {
    Generating,
    Ready,
    Exploring,
    Solved,
    Failed,
}

#[derive(Debug)]
pub struct App {
    config: Config,
    maze: Maze,
    generator: MazeGenerator,
    explorer: Explorer,
    phase: Phase,
    paused: bool,
    speed: u64,
    seed: u64,
    maze_generation: u64,
    terminal_width: u16,
    terminal_height: u16,
    fits: bool,
    message: String,
    last_step: Instant,
}

impl App {
    pub fn new(config: Config, terminal_width: u16, terminal_height: u16) -> Self {
        let speed = config.normalized_speed();
        let seed = config.seed.unwrap_or_else(random);
        let mut app = Self::with_placeholder(config, terminal_width, terminal_height, speed, seed);
        app.new_maze();
        app
    }

    fn with_placeholder(
        config: Config,
        terminal_width: u16,
        terminal_height: u16,
        speed: u64,
        seed: u64,
    ) -> Self {
        let maze = Maze::new(MIN_WIDTH, MIN_HEIGHT);
        let generator = MazeGenerator::with_algorithm(&maze, config.generator, seed);
        let explorer = Explorer::with_algorithm(&maze, config.solver_algorithm());

        Self {
            config,
            maze,
            generator,
            explorer,
            phase: Phase::Generating,
            paused: false,
            speed,
            seed,
            maze_generation: 0,
            terminal_width,
            terminal_height,
            fits: true,
            message: String::new(),
            last_step: Instant::now(),
        }
    }

    pub fn maze(&self) -> &Maze {
        &self.maze
    }

    pub fn generator(&self) -> &MazeGenerator {
        &self.generator
    }

    pub fn explorer(&self) -> &Explorer {
        &self.explorer
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn paused(&self) -> bool {
        self.paused
    }

    pub fn speed(&self) -> u64 {
        self.speed
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn generator_algorithm(&self) -> GeneratorAlgorithm {
        self.config.generator
    }

    pub fn solver_algorithm(&self) -> SolverAlgorithm {
        self.config.solver_algorithm()
    }

    pub fn ascii(&self) -> bool {
        self.config.ascii
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn fits(&self) -> bool {
        self.fits
    }

    pub fn render_phase(&self) -> RenderPhase {
        match self.phase {
            Phase::Generating => RenderPhase::Generating,
            Phase::Ready => RenderPhase::Ready,
            Phase::Exploring => RenderPhase::Exploring,
            Phase::Solved => RenderPhase::Solved,
            Phase::Failed => RenderPhase::Failed,
        }
    }

    pub fn required_render_size(&self) -> (usize, usize) {
        render_size(self.maze.width(), self.maze.height())
    }

    pub fn tick(&mut self) {
        if self.paused || !self.fits {
            return;
        }

        let interval = Duration::from_secs_f64(1.0 / self.speed as f64);
        if self.last_step.elapsed() >= interval {
            self.step_once();
            self.last_step = Instant::now();
        }
    }

    pub fn step_once(&mut self) {
        if !self.fits {
            return;
        }

        match self.phase {
            Phase::Generating => {
                self.generator.step(&mut self.maze);
                if self.generator.is_done() {
                    self.explorer =
                        Explorer::with_algorithm(&self.maze, self.config.solver_algorithm());
                    if self.config.auto_start {
                        self.begin_exploration();
                    } else {
                        self.phase = Phase::Ready;
                        self.paused = true;
                        self.message =
                            "Generation complete. Press Space to start exploring.".to_string();
                    }
                }
            }
            Phase::Ready => {
                self.begin_exploration();
                self.step_exploration();
            }
            Phase::Exploring => self.step_exploration(),
            Phase::Solved | Phase::Failed => {}
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'))
        {
            return true;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => return true,
            KeyCode::Char(' ') => {
                if self.phase == Phase::Ready {
                    self.begin_exploration();
                } else {
                    self.paused = !self.paused;
                    self.message = if self.paused {
                        "Paused. Press Space to continue or S to step.".to_string()
                    } else {
                        "Running.".to_string()
                    };
                }
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.step_once();
                self.paused = true;
                self.message = "Stepped once. Press Space to resume.".to_string();
            }
            KeyCode::Char('n') | KeyCode::Char('N') => self.new_maze(),
            KeyCode::Char('r') | KeyCode::Char('R') => self.restart_exploration(),
            KeyCode::Char('+') | KeyCode::Char('=') => self.adjust_speed(10),
            KeyCode::Char('-') | KeyCode::Char('_') => self.adjust_speed(-10),
            KeyCode::Char('1') => {
                self.set_solver(SolverAlgorithm::Dfs);
            }
            KeyCode::Char('2') => {
                self.set_solver(SolverAlgorithm::Bfs);
            }
            KeyCode::Char('3') => {
                self.set_solver(SolverAlgorithm::Astar);
            }
            KeyCode::Char('4') => {
                self.set_solver(SolverAlgorithm::Dijkstra);
            }
            KeyCode::Char('5') => {
                self.set_solver(SolverAlgorithm::DeadEnd);
            }
            KeyCode::Char('6') => {
                self.set_solver(SolverAlgorithm::WallFollower);
            }
            _ => {}
        }

        false
    }

    pub fn handle_resize(&mut self, width: u16, height: u16) {
        self.terminal_width = width;
        self.terminal_height = height;
        self.fits = self.current_maze_fits();

        if self.fits {
            self.message = "Window resized. Press N to generate for the new size.".to_string();
        } else {
            self.paused = true;
            self.message =
                "Window is too small. Enlarge it or press N for an auto-sized maze.".to_string();
        }
    }

    fn new_maze(&mut self) {
        let (width, height) = self.next_maze_dimensions();
        self.seed = self.next_seed();
        self.maze = Maze::new(width, height);
        self.generator =
            MazeGenerator::with_algorithm(&self.maze, self.config.generator, self.seed);
        self.explorer = Explorer::with_algorithm(&self.maze, self.config.solver_algorithm());
        self.phase = Phase::Generating;
        self.paused = false;
        self.fits = self.current_maze_fits();
        self.last_step = Instant::now();
        self.message = if self.fits {
            format!(
                "Generating {} maze with seed {}.",
                self.config.generator.label(),
                self.seed
            )
        } else {
            "Maze does not fit. Enlarge the terminal or use smaller dimensions.".to_string()
        };
    }

    fn restart_exploration(&mut self) {
        if matches!(self.phase, Phase::Generating) {
            self.message = "Finish generation before replaying exploration.".to_string();
            return;
        }

        self.explorer = Explorer::with_algorithm(&self.maze, self.config.solver_algorithm());
        if self.config.auto_start {
            self.begin_exploration();
        } else {
            self.phase = Phase::Ready;
            self.paused = true;
            self.message = "Exploration reset. Press Space to start.".to_string();
        }
    }

    fn adjust_speed(&mut self, delta: i64) {
        let next = (self.speed as i64 + delta).clamp(1, 240);
        self.speed = next as u64;
        self.message = format!("Speed: {} steps/sec.", self.speed);
    }

    fn begin_exploration(&mut self) {
        self.phase = Phase::Exploring;
        self.paused = false;
        self.message = format!(
            "Exploring with {}. Press Space to pause.",
            self.config.solver_algorithm().label()
        );
    }

    fn step_exploration(&mut self) {
        self.explorer.step(&self.maze);
        match self.explorer.status() {
            ExplorationStatus::Solved => {
                self.phase = Phase::Solved;
                self.message = "Solved. Press N for a new maze or R to replay.".to_string();
            }
            ExplorationStatus::Failed => {
                self.phase = Phase::Failed;
                self.message = "No path found. Press N for a new maze.".to_string();
            }
            ExplorationStatus::Running => {}
        }
    }

    fn set_solver(&mut self, solver: SolverAlgorithm) {
        self.config.solver = solver;
        self.config.algorithm = None;

        if matches!(self.phase, Phase::Generating) {
            self.message = format!("Solver set to {}.", solver.label());
            return;
        }

        self.explorer = Explorer::with_algorithm(&self.maze, solver);
        self.phase = Phase::Ready;
        self.paused = true;
        self.message = format!("Solver set to {}. Press Space to start.", solver.label());
    }

    fn next_seed(&mut self) -> u64 {
        self.maze_generation += 1;
        self.config
            .seed
            .map(|seed| seed.wrapping_add(self.maze_generation - 1))
            .unwrap_or_else(random)
    }

    fn next_maze_dimensions(&self) -> (usize, usize) {
        let (auto_width, auto_height) = auto_dimensions(self.terminal_width, self.terminal_height);
        (
            self.config.width.unwrap_or(auto_width),
            self.config.height.unwrap_or(auto_height),
        )
    }

    fn current_maze_fits(&self) -> bool {
        let (required_width, required_height) = self.required_render_size();
        let available_width = usize::from(self.terminal_width.saturating_sub(2));
        let available_height = usize::from(self.terminal_height.saturating_sub(2 + STATUS_HEIGHT));

        required_width <= available_width && required_height <= available_height
    }
}

pub fn auto_dimensions(terminal_width: u16, terminal_height: u16) -> (usize, usize) {
    let available_width = usize::from(terminal_width.saturating_sub(2));
    let available_height = usize::from(terminal_height.saturating_sub(2 + STATUS_HEIGHT));

    let width = available_width.saturating_sub(1) / 4;
    let height = available_height.saturating_sub(1) / 2;

    (width.max(MIN_WIDTH), height.max(MIN_HEIGHT))
}
