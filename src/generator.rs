use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};

use crate::config::GeneratorAlgorithm;
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
    algorithm: GeneratorAlgorithm,
    state: GeneratorState,
}

#[derive(Clone, Debug)]
enum GeneratorState {
    Dfs(DfsGenerator),
    Prim(PrimGenerator),
    Kruskal(KruskalGenerator),
}

impl MazeGenerator {
    pub fn new(maze: &Maze, seed: u64) -> Self {
        Self::with_algorithm(maze, GeneratorAlgorithm::Dfs, seed)
    }

    pub fn with_algorithm(maze: &Maze, algorithm: GeneratorAlgorithm, seed: u64) -> Self {
        let state = match algorithm {
            GeneratorAlgorithm::Dfs => GeneratorState::Dfs(DfsGenerator::new(maze, seed)),
            GeneratorAlgorithm::Prim => GeneratorState::Prim(PrimGenerator::new(maze, seed)),
            GeneratorAlgorithm::Kruskal => {
                GeneratorState::Kruskal(KruskalGenerator::new(maze, seed))
            }
        };

        Self { algorithm, state }
    }

    pub fn algorithm(&self) -> GeneratorAlgorithm {
        self.algorithm
    }

    pub fn current(&self) -> Pos {
        match &self.state {
            GeneratorState::Dfs(state) => state.current,
            GeneratorState::Prim(state) => state.current,
            GeneratorState::Kruskal(state) => state.current,
        }
    }

    pub fn step_count(&self) -> usize {
        match &self.state {
            GeneratorState::Dfs(state) => state.step_count,
            GeneratorState::Prim(state) => state.step_count,
            GeneratorState::Kruskal(state) => state.step_count,
        }
    }

    pub fn status(&self) -> GenerationStatus {
        match &self.state {
            GeneratorState::Dfs(state) => state.status,
            GeneratorState::Prim(state) => state.status,
            GeneratorState::Kruskal(state) => state.status,
        }
    }

    pub fn is_done(&self) -> bool {
        self.status() == GenerationStatus::Done
    }

    pub fn last_event(&self) -> GenerationEvent {
        match &self.state {
            GeneratorState::Dfs(state) => state.last_event,
            GeneratorState::Prim(state) => state.last_event,
            GeneratorState::Kruskal(state) => state.last_event,
        }
    }

    pub fn visited(&self, maze: &Maze, pos: Pos) -> bool {
        match &self.state {
            GeneratorState::Dfs(state) => state.visited[maze.index(pos)],
            GeneratorState::Prim(state) => state.visited[maze.index(pos)],
            GeneratorState::Kruskal(state) => state.touched[maze.index(pos)],
        }
    }

    pub fn step(&mut self, maze: &mut Maze) -> GenerationEvent {
        match &mut self.state {
            GeneratorState::Dfs(state) => state.step(maze),
            GeneratorState::Prim(state) => state.step(maze),
            GeneratorState::Kruskal(state) => state.step(maze),
        }
    }
}

#[derive(Clone, Debug)]
struct DfsGenerator {
    current: Pos,
    stack: Vec<Pos>,
    visited: Vec<bool>,
    visited_count: usize,
    step_count: usize,
    status: GenerationStatus,
    last_event: GenerationEvent,
    rng: StdRng,
}

impl DfsGenerator {
    fn new(maze: &Maze, seed: u64) -> Self {
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

    fn step(&mut self, maze: &mut Maze) -> GenerationEvent {
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
        finish_generation(maze, &mut self.status, &mut self.last_event);
        self.last_event
    }
}

#[derive(Clone, Debug)]
struct PrimGenerator {
    current: Pos,
    frontier: Vec<Edge>,
    visited: Vec<bool>,
    visited_count: usize,
    step_count: usize,
    status: GenerationStatus,
    last_event: GenerationEvent,
    rng: StdRng,
}

impl PrimGenerator {
    fn new(maze: &Maze, seed: u64) -> Self {
        let current = maze.start();
        let mut visited = vec![false; maze.len()];
        visited[maze.index(current)] = true;
        let mut state = Self {
            current,
            frontier: Vec::new(),
            visited,
            visited_count: 1,
            step_count: 0,
            status: GenerationStatus::Running,
            last_event: GenerationEvent::Visit(current),
            rng: StdRng::seed_from_u64(seed),
        };
        state.add_frontier(maze, current);
        state
    }

    fn step(&mut self, maze: &mut Maze) -> GenerationEvent {
        if self.status == GenerationStatus::Done {
            return GenerationEvent::Done;
        }

        while !self.frontier.is_empty() {
            let edge_index = self.rng.gen_range(0..self.frontier.len());
            let edge = self.frontier.swap_remove(edge_index);

            if self.visited[maze.index(edge.to)] {
                continue;
            }

            maze.carve_between(edge.from, edge.to);
            self.current = edge.to;
            self.visited[maze.index(edge.to)] = true;
            self.visited_count += 1;
            self.step_count += 1;
            self.add_frontier(maze, edge.to);
            self.last_event = GenerationEvent::Carve {
                from: edge.from,
                to: edge.to,
                direction: edge.direction,
            };
            return self.last_event;
        }

        debug_assert_eq!(self.visited_count, maze.len());
        finish_generation(maze, &mut self.status, &mut self.last_event);
        self.last_event
    }

    fn add_frontier(&mut self, maze: &Maze, from: Pos) {
        self.frontier
            .extend(maze.neighbors(from).filter_map(|(direction, to)| {
                (!self.visited[maze.index(to)]).then_some(Edge {
                    from,
                    to,
                    direction,
                })
            }));
    }
}

#[derive(Clone, Debug)]
struct KruskalGenerator {
    current: Pos,
    edges: Vec<Edge>,
    disjoint_set: DisjointSet,
    touched: Vec<bool>,
    step_count: usize,
    status: GenerationStatus,
    last_event: GenerationEvent,
}

impl KruskalGenerator {
    fn new(maze: &Maze, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut edges = Vec::new();

        for row in 0..maze.height() {
            for col in 0..maze.width() {
                let from = Pos::new(row, col);
                for direction in [Direction::East, Direction::South] {
                    if let Some(to) = maze.neighbor(from, direction) {
                        edges.push(Edge {
                            from,
                            to,
                            direction,
                        });
                    }
                }
            }
        }

        edges.shuffle(&mut rng);

        Self {
            current: maze.start(),
            edges,
            disjoint_set: DisjointSet::new(maze.len()),
            touched: vec![false; maze.len()],
            step_count: 0,
            status: GenerationStatus::Running,
            last_event: GenerationEvent::Visit(maze.start()),
        }
    }

    fn step(&mut self, maze: &mut Maze) -> GenerationEvent {
        if self.status == GenerationStatus::Done {
            return GenerationEvent::Done;
        }

        while let Some(edge) = self.edges.pop() {
            let from_index = maze.index(edge.from);
            let to_index = maze.index(edge.to);

            if !self.disjoint_set.union(from_index, to_index) {
                continue;
            }

            maze.carve_between(edge.from, edge.to);
            self.current = edge.to;
            self.touched[from_index] = true;
            self.touched[to_index] = true;
            self.step_count += 1;
            self.last_event = GenerationEvent::Carve {
                from: edge.from,
                to: edge.to,
                direction: edge.direction,
            };
            return self.last_event;
        }

        finish_generation(maze, &mut self.status, &mut self.last_event);
        self.last_event
    }
}

#[derive(Copy, Clone, Debug)]
struct Edge {
    from: Pos,
    to: Pos,
    direction: Direction,
}

#[derive(Clone, Debug)]
struct DisjointSet {
    parent: Vec<usize>,
    rank: Vec<u8>,
}

impl DisjointSet {
    fn new(size: usize) -> Self {
        Self {
            parent: (0..size).collect(),
            rank: vec![0; size],
        }
    }

    fn find(&mut self, item: usize) -> usize {
        if self.parent[item] != item {
            self.parent[item] = self.find(self.parent[item]);
        }

        self.parent[item]
    }

    fn union(&mut self, left: usize, right: usize) -> bool {
        let left_root = self.find(left);
        let right_root = self.find(right);

        if left_root == right_root {
            return false;
        }

        if self.rank[left_root] < self.rank[right_root] {
            self.parent[left_root] = right_root;
        } else if self.rank[left_root] > self.rank[right_root] {
            self.parent[right_root] = left_root;
        } else {
            self.parent[right_root] = left_root;
            self.rank[left_root] += 1;
        }

        true
    }
}

fn finish_generation(
    maze: &mut Maze,
    status: &mut GenerationStatus,
    last_event: &mut GenerationEvent,
) {
    maze.open_entrance_exit();
    *status = GenerationStatus::Done;
    *last_event = GenerationEvent::Done;
}
