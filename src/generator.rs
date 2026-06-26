use rand::rngs::StdRng;
use rand::seq::{IndexedRandom, SliceRandom};
use rand::{RngExt, SeedableRng};

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
    Wall {
        from: Pos,
        to: Pos,
        direction: Direction,
    },
    Done,
}

#[derive(Debug)]
pub struct MazeGenerator {
    algorithm: GeneratorAlgorithm,
    state: GeneratorState,
}

#[derive(Debug)]
enum GeneratorState {
    Dfs(DfsGenerator),
    Prim(PrimGenerator),
    Kruskal(KruskalGenerator),
    AldousBroder(AldousBroderGenerator),
    Wilson(WilsonGenerator),
    RecursiveDivision(RecursiveDivisionGenerator),
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
            GeneratorAlgorithm::AldousBroder => {
                GeneratorState::AldousBroder(AldousBroderGenerator::new(maze, seed))
            }
            GeneratorAlgorithm::Wilson => GeneratorState::Wilson(WilsonGenerator::new(maze, seed)),
            GeneratorAlgorithm::RecursiveDivision => {
                GeneratorState::RecursiveDivision(RecursiveDivisionGenerator::new(maze, seed))
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
            GeneratorState::AldousBroder(state) => state.current,
            GeneratorState::Wilson(state) => state.current,
            GeneratorState::RecursiveDivision(state) => state.current,
        }
    }

    pub fn step_count(&self) -> usize {
        match &self.state {
            GeneratorState::Dfs(state) => state.step_count,
            GeneratorState::Prim(state) => state.step_count,
            GeneratorState::Kruskal(state) => state.step_count,
            GeneratorState::AldousBroder(state) => state.step_count,
            GeneratorState::Wilson(state) => state.step_count,
            GeneratorState::RecursiveDivision(state) => state.step_count,
        }
    }

    pub fn status(&self) -> GenerationStatus {
        match &self.state {
            GeneratorState::Dfs(state) => state.status,
            GeneratorState::Prim(state) => state.status,
            GeneratorState::Kruskal(state) => state.status,
            GeneratorState::AldousBroder(state) => state.status,
            GeneratorState::Wilson(state) => state.status,
            GeneratorState::RecursiveDivision(state) => state.status,
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
            GeneratorState::AldousBroder(state) => state.last_event,
            GeneratorState::Wilson(state) => state.last_event,
            GeneratorState::RecursiveDivision(state) => state.last_event,
        }
    }

    pub fn visited(&self, maze: &Maze, pos: Pos) -> bool {
        match &self.state {
            GeneratorState::Dfs(state) => state.visited[maze.index(pos)],
            GeneratorState::Prim(state) => state.visited[maze.index(pos)],
            GeneratorState::Kruskal(state) => state.touched[maze.index(pos)],
            GeneratorState::AldousBroder(state) => state.visited[maze.index(pos)],
            GeneratorState::Wilson(state) => state.in_tree[maze.index(pos)],
            GeneratorState::RecursiveDivision(state) => state.touched[maze.index(pos)],
        }
    }

    pub fn step(&mut self, maze: &mut Maze) -> GenerationEvent {
        match &mut self.state {
            GeneratorState::Dfs(state) => state.step(maze),
            GeneratorState::Prim(state) => state.step(maze),
            GeneratorState::Kruskal(state) => state.step(maze),
            GeneratorState::AldousBroder(state) => state.step(maze),
            GeneratorState::Wilson(state) => state.step(maze),
            GeneratorState::RecursiveDivision(state) => state.step(maze),
        }
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
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
            let edge_index = self.rng.random_range(0..self.frontier.len());
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

#[derive(Debug)]
struct AldousBroderGenerator {
    current: Pos,
    visited: Vec<bool>,
    visited_count: usize,
    step_count: usize,
    status: GenerationStatus,
    last_event: GenerationEvent,
    rng: StdRng,
}

impl AldousBroderGenerator {
    fn new(maze: &Maze, seed: u64) -> Self {
        let current = maze.start();
        let mut visited = vec![false; maze.len()];
        visited[maze.index(current)] = true;

        Self {
            current,
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

        if self.visited_count == maze.len() {
            finish_generation(maze, &mut self.status, &mut self.last_event);
            return self.last_event;
        }

        let neighbors: Vec<(Direction, Pos)> = maze.neighbors(self.current).collect();
        let Some((direction, next)) = neighbors.choose(&mut self.rng).copied() else {
            finish_generation(maze, &mut self.status, &mut self.last_event);
            return self.last_event;
        };

        let from = self.current;
        self.current = next;
        self.step_count += 1;

        if self.visited[maze.index(next)] {
            self.last_event = GenerationEvent::Visit(next);
            return self.last_event;
        }

        maze.carve_between(from, next);
        self.visited[maze.index(next)] = true;
        self.visited_count += 1;
        self.last_event = GenerationEvent::Carve {
            from,
            to: next,
            direction,
        };

        if self.visited_count == maze.len() {
            maze.open_entrance_exit();
            self.status = GenerationStatus::Done;
        }

        self.last_event
    }
}

#[derive(Debug)]
struct WilsonGenerator {
    current: Pos,
    in_tree: Vec<bool>,
    tree_count: usize,
    walk: Vec<Pos>,
    walk_indices: Vec<Option<usize>>,
    pending_path: Vec<Pos>,
    pending_index: usize,
    step_count: usize,
    status: GenerationStatus,
    last_event: GenerationEvent,
    rng: StdRng,
}

impl WilsonGenerator {
    fn new(maze: &Maze, seed: u64) -> Self {
        let current = maze.start();
        let mut in_tree = vec![false; maze.len()];
        in_tree[maze.index(current)] = true;

        Self {
            current,
            in_tree,
            tree_count: 1,
            walk: Vec::new(),
            walk_indices: vec![None; maze.len()],
            pending_path: Vec::new(),
            pending_index: 0,
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

        if self.tree_count == maze.len() {
            finish_generation(maze, &mut self.status, &mut self.last_event);
            return self.last_event;
        }

        if self.has_pending_carve() {
            return self.carve_pending_path(maze);
        }

        if self.walk.is_empty() {
            return self.start_walk(maze);
        }

        self.advance_walk(maze)
    }

    fn has_pending_carve(&self) -> bool {
        self.pending_index + 1 < self.pending_path.len()
    }

    fn carve_pending_path(&mut self, maze: &mut Maze) -> GenerationEvent {
        let from = self.pending_path[self.pending_index];
        let to = self.pending_path[self.pending_index + 1];
        let direction = maze
            .carve_between(from, to)
            .expect("Wilson pending path contains adjacent cells");

        self.mark_in_tree(maze, from);
        self.mark_in_tree(maze, to);
        self.current = to;
        self.pending_index += 1;
        self.step_count += 1;
        self.last_event = GenerationEvent::Carve {
            from,
            to,
            direction,
        };

        if !self.has_pending_carve() {
            self.pending_path.clear();
            self.pending_index = 0;

            if self.tree_count == maze.len() {
                maze.open_entrance_exit();
                self.status = GenerationStatus::Done;
            }
        }

        self.last_event
    }

    fn start_walk(&mut self, maze: &Maze) -> GenerationEvent {
        let Some(start) = self.random_unvisited(maze) else {
            self.status = GenerationStatus::Done;
            self.last_event = GenerationEvent::Done;
            return self.last_event;
        };

        self.current = start;
        self.walk.push(start);
        self.walk_indices[maze.index(start)] = Some(0);
        self.step_count += 1;
        self.last_event = GenerationEvent::Visit(start);
        self.last_event
    }

    fn advance_walk(&mut self, maze: &Maze) -> GenerationEvent {
        let from = *self
            .walk
            .last()
            .expect("Wilson walk is initialized before advancing");
        let neighbors: Vec<Pos> = maze.neighbors(from).map(|(_, pos)| pos).collect();
        let Some(next) = neighbors.choose(&mut self.rng).copied() else {
            self.last_event = GenerationEvent::Done;
            return self.last_event;
        };

        self.current = next;
        self.step_count += 1;

        if self.in_tree[maze.index(next)] {
            self.pending_path = self.walk.clone();
            self.pending_path.push(next);
            self.clear_walk_indices(maze);
            self.walk.clear();
            self.last_event = GenerationEvent::Visit(next);
            return self.last_event;
        }

        if let Some(loop_start) = self.walk_indices[maze.index(next)] {
            self.erase_walk_loop(maze, loop_start);
        } else {
            self.walk_indices[maze.index(next)] = Some(self.walk.len());
            self.walk.push(next);
        }

        self.last_event = GenerationEvent::Visit(next);
        self.last_event
    }

    fn random_unvisited(&mut self, maze: &Maze) -> Option<Pos> {
        let unvisited: Vec<usize> = self
            .in_tree
            .iter()
            .enumerate()
            .filter_map(|(index, in_tree)| (!in_tree).then_some(index))
            .collect();
        unvisited
            .choose(&mut self.rng)
            .map(|index| Pos::new(index / maze.width(), index % maze.width()))
    }

    fn erase_walk_loop(&mut self, maze: &Maze, loop_start: usize) {
        for pos in self.walk.drain(loop_start + 1..) {
            self.walk_indices[maze.index(pos)] = None;
        }
    }

    fn clear_walk_indices(&mut self, maze: &Maze) {
        for pos in &self.walk {
            self.walk_indices[maze.index(*pos)] = None;
        }
    }

    fn mark_in_tree(&mut self, maze: &Maze, pos: Pos) {
        let index = maze.index(pos);
        if !self.in_tree[index] {
            self.in_tree[index] = true;
            self.tree_count += 1;
        }
    }
}

#[derive(Debug)]
struct RecursiveDivisionGenerator {
    current: Pos,
    regions: Vec<Region>,
    pending_walls: Vec<DivisionWall>,
    touched: Vec<bool>,
    initialized: bool,
    step_count: usize,
    status: GenerationStatus,
    last_event: GenerationEvent,
    rng: StdRng,
}

impl RecursiveDivisionGenerator {
    fn new(maze: &Maze, seed: u64) -> Self {
        Self {
            current: maze.start(),
            regions: vec![Region {
                row: 0,
                col: 0,
                width: maze.width(),
                height: maze.height(),
            }],
            pending_walls: Vec::new(),
            touched: vec![false; maze.len()],
            initialized: false,
            step_count: 0,
            status: GenerationStatus::Running,
            last_event: GenerationEvent::Visit(maze.start()),
            rng: StdRng::seed_from_u64(seed),
        }
    }

    fn step(&mut self, maze: &mut Maze) -> GenerationEvent {
        if self.status == GenerationStatus::Done {
            return GenerationEvent::Done;
        }

        if !self.initialized {
            maze.open_all_internal_walls();
            self.touched.fill(true);
            self.initialized = true;
            self.step_count += 1;
            self.last_event = GenerationEvent::Visit(self.current);
            return self.last_event;
        }

        loop {
            if let Some(wall) = self.pending_walls.pop() {
                if let Some(direction) = maze.add_wall_between(wall.from, wall.to) {
                    self.current = wall.to;
                    self.step_count += 1;
                    self.last_event = GenerationEvent::Wall {
                        from: wall.from,
                        to: wall.to,
                        direction,
                    };
                    return self.last_event;
                }
            } else if let Some(region) = self.regions.pop() {
                self.divide_region(region);
            } else {
                finish_generation(maze, &mut self.status, &mut self.last_event);
                return self.last_event;
            }
        }
    }

    fn divide_region(&mut self, region: Region) {
        if region.width < 2 && region.height < 2 {
            return;
        }

        let orientation = if region.width < 2 {
            DivisionOrientation::Horizontal
        } else if region.height < 2 {
            DivisionOrientation::Vertical
        } else if region.width < region.height {
            DivisionOrientation::Horizontal
        } else if region.height < region.width {
            DivisionOrientation::Vertical
        } else if self.rng.random_bool(0.5) {
            DivisionOrientation::Horizontal
        } else {
            DivisionOrientation::Vertical
        };

        match orientation {
            DivisionOrientation::Vertical => self.divide_vertically(region),
            DivisionOrientation::Horizontal => self.divide_horizontally(region),
        }
    }

    fn divide_vertically(&mut self, region: Region) {
        if region.width < 2 {
            return;
        }

        let wall_col = self
            .rng
            .random_range(region.col + 1..region.col + region.width);
        let passage_row = self
            .rng
            .random_range(region.row..region.row + region.height);

        for row in region.row..region.row + region.height {
            if row == passage_row {
                continue;
            }

            self.pending_walls.push(DivisionWall {
                from: Pos::new(row, wall_col - 1),
                to: Pos::new(row, wall_col),
            });
        }

        let left_width = wall_col - region.col;
        let right_width = region.col + region.width - wall_col;
        self.push_region(Region {
            row: region.row,
            col: region.col,
            width: left_width,
            height: region.height,
        });
        self.push_region(Region {
            row: region.row,
            col: wall_col,
            width: right_width,
            height: region.height,
        });
    }

    fn divide_horizontally(&mut self, region: Region) {
        if region.height < 2 {
            return;
        }

        let wall_row = self
            .rng
            .random_range(region.row + 1..region.row + region.height);
        let passage_col = self.rng.random_range(region.col..region.col + region.width);

        for col in region.col..region.col + region.width {
            if col == passage_col {
                continue;
            }

            self.pending_walls.push(DivisionWall {
                from: Pos::new(wall_row - 1, col),
                to: Pos::new(wall_row, col),
            });
        }

        let top_height = wall_row - region.row;
        let bottom_height = region.row + region.height - wall_row;
        self.push_region(Region {
            row: region.row,
            col: region.col,
            width: region.width,
            height: top_height,
        });
        self.push_region(Region {
            row: wall_row,
            col: region.col,
            width: region.width,
            height: bottom_height,
        });
    }

    fn push_region(&mut self, region: Region) {
        if region.width > 1 || region.height > 1 {
            self.regions.push(region);
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct Region {
    row: usize,
    col: usize,
    width: usize,
    height: usize,
}

#[derive(Copy, Clone, Debug)]
struct DivisionWall {
    from: Pos,
    to: Pos,
}

#[derive(Copy, Clone, Debug)]
enum DivisionOrientation {
    Vertical,
    Horizontal,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disjoint_set_unions_components_once() {
        let mut disjoint_set = DisjointSet::new(4);

        assert!(disjoint_set.union(0, 1));
        assert!(disjoint_set.union(2, 3));
        assert!(disjoint_set.union(1, 2));
        assert!(!disjoint_set.union(0, 3));

        let root = disjoint_set.find(0);
        assert_eq!(disjoint_set.find(1), root);
        assert_eq!(disjoint_set.find(2), root);
        assert_eq!(disjoint_set.find(3), root);
    }
}
