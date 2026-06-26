use crate::maze::{Maze, Pos};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ExplorationStatus {
    Running,
    Solved,
    Failed,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ExplorationEvent {
    Visit(Pos),
    Move { from: Pos, to: Pos },
    Backtrack { from: Pos, to: Pos },
    Solved,
    Failed,
}

#[derive(Clone, Debug)]
pub struct Explorer {
    current: Pos,
    stack: Vec<Pos>,
    visited: Vec<bool>,
    parent: Vec<Option<Pos>>,
    final_path: Vec<Pos>,
    status: ExplorationStatus,
    step_count: usize,
    last_event: ExplorationEvent,
}

impl Explorer {
    pub fn new(maze: &Maze) -> Self {
        let current = maze.start();
        let mut visited = vec![false; maze.len()];
        visited[maze.index(current)] = true;

        Self {
            current,
            stack: Vec::new(),
            visited,
            parent: vec![None; maze.len()],
            final_path: Vec::new(),
            status: ExplorationStatus::Running,
            step_count: 0,
            last_event: ExplorationEvent::Visit(current),
        }
    }

    pub fn current(&self) -> Pos {
        self.current
    }

    pub fn step_count(&self) -> usize {
        self.step_count
    }

    pub fn status(&self) -> ExplorationStatus {
        self.status
    }

    pub fn is_finished(&self) -> bool {
        matches!(
            self.status,
            ExplorationStatus::Solved | ExplorationStatus::Failed
        )
    }

    pub fn visited(&self, maze: &Maze, pos: Pos) -> bool {
        self.visited[maze.index(pos)]
    }

    pub fn is_final_path(&self, pos: Pos) -> bool {
        self.final_path.contains(&pos)
    }

    pub fn final_path(&self) -> &[Pos] {
        &self.final_path
    }

    pub fn last_event(&self) -> ExplorationEvent {
        self.last_event
    }

    pub fn step(&mut self, maze: &Maze) -> ExplorationEvent {
        if self.is_finished() {
            return self.last_event;
        }

        if self.current == maze.exit() {
            self.solve(maze);
            return self.last_event;
        }

        if let Some((_, next)) = maze
            .reachable_neighbors(self.current)
            .find(|(_, pos)| !self.visited[maze.index(*pos)])
        {
            let from = self.current;
            self.stack.push(from);
            self.current = next;
            self.visited[maze.index(next)] = true;
            self.parent[maze.index(next)] = Some(from);
            self.step_count += 1;
            self.last_event = ExplorationEvent::Move { from, to: next };

            if self.current == maze.exit() {
                self.solve(maze);
            }

            return self.last_event;
        }

        if let Some(previous) = self.stack.pop() {
            let from = self.current;
            self.current = previous;
            self.step_count += 1;
            self.last_event = ExplorationEvent::Backtrack { from, to: previous };
            return self.last_event;
        }

        self.status = ExplorationStatus::Failed;
        self.last_event = ExplorationEvent::Failed;
        self.last_event
    }

    fn solve(&mut self, maze: &Maze) {
        let mut path = Vec::new();
        let mut cursor = maze.exit();
        path.push(cursor);

        while cursor != maze.start() {
            let Some(parent) = self.parent[maze.index(cursor)] else {
                self.status = ExplorationStatus::Failed;
                self.last_event = ExplorationEvent::Failed;
                return;
            };
            cursor = parent;
            path.push(cursor);
        }

        path.reverse();
        self.final_path = path;
        self.status = ExplorationStatus::Solved;
        self.last_event = ExplorationEvent::Solved;
    }
}
