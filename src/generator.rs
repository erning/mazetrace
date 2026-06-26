use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use crate::maze::{Direction, Maze, Pos};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum GenerationStatus {
    Running,
    Done,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum GenerationEvent {
    Visit(Pos),
    Carve {
        from: Pos,
        to: Pos,
        direction: Direction,
    },
    Backtrack {
        from: Pos,
        to: Pos,
    },
    Done,
}

#[derive(Clone, Debug)]
pub struct MazeGenerator {
    current: Pos,
    stack: Vec<Pos>,
    visited: Vec<bool>,
    visited_count: usize,
    step_count: usize,
    status: GenerationStatus,
    last_event: GenerationEvent,
    rng: StdRng,
}

impl MazeGenerator {
    pub fn new(maze: &Maze, seed: u64) -> Self {
        let current = maze.start();
        let mut visited = vec![false; maze.len()];
        visited[maze.index(current)] = true;

        Self {
            current,
            stack: Vec::new(),
            visited,
            visited_count: 1,
            step_count: 0,
            status: GenerationStatus::Running,
            last_event: GenerationEvent::Visit(current),
            rng: StdRng::seed_from_u64(seed),
        }
    }

    pub fn current(&self) -> Pos {
        self.current
    }

    pub fn step_count(&self) -> usize {
        self.step_count
    }

    pub fn status(&self) -> GenerationStatus {
        self.status
    }

    pub fn is_done(&self) -> bool {
        self.status == GenerationStatus::Done
    }

    pub fn last_event(&self) -> GenerationEvent {
        self.last_event
    }

    pub fn visited(&self, maze: &Maze, pos: Pos) -> bool {
        self.visited[maze.index(pos)]
    }

    pub fn step(&mut self, maze: &mut Maze) -> GenerationEvent {
        if self.status == GenerationStatus::Done {
            return GenerationEvent::Done;
        }

        let unvisited_neighbors: Vec<(Direction, Pos)> = maze
            .neighbors(self.current)
            .filter(|(_, pos)| !self.visited[maze.index(*pos)])
            .collect();

        if let Some((direction, next)) = unvisited_neighbors.choose(&mut self.rng).copied() {
            let from = self.current;
            maze.carve_between(from, next);
            self.stack.push(from);
            self.current = next;
            self.visited[maze.index(next)] = true;
            self.visited_count += 1;
            self.step_count += 1;
            self.last_event = GenerationEvent::Carve {
                from,
                to: next,
                direction,
            };
            return self.last_event;
        }

        if let Some(previous) = self.stack.pop() {
            let from = self.current;
            self.current = previous;
            self.step_count += 1;
            self.last_event = GenerationEvent::Backtrack { from, to: previous };
            return self.last_event;
        }

        debug_assert_eq!(self.visited_count, maze.len());
        maze.open_entrance_exit();
        self.status = GenerationStatus::Done;
        self.last_event = GenerationEvent::Done;
        self.last_event
    }
}
