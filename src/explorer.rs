use std::collections::VecDeque;

use crate::config::SolverAlgorithm;
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
    algorithm: SolverAlgorithm,
    current: Pos,
    visited: Vec<bool>,
    final_path: Vec<Pos>,
    steps: Vec<SolverStep>,
    next_step: usize,
    status: ExplorationStatus,
    step_count: usize,
    last_event: ExplorationEvent,
}

impl Explorer {
    pub fn new(maze: &Maze) -> Self {
        Self::with_algorithm(maze, SolverAlgorithm::Dfs)
    }

    pub fn with_algorithm(maze: &Maze, algorithm: SolverAlgorithm) -> Self {
        let current = maze.start();
        let mut visited = vec![false; maze.len()];
        visited[maze.index(current)] = true;

        Self {
            algorithm,
            current,
            visited,
            final_path: Vec::new(),
            steps: plan_steps(maze, algorithm),
            next_step: 0,
            status: ExplorationStatus::Running,
            step_count: 0,
            last_event: ExplorationEvent::Visit(current),
        }
    }

    pub fn algorithm(&self) -> SolverAlgorithm {
        self.algorithm
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

        let Some(step) = self.steps.get(self.next_step).cloned() else {
            self.status = ExplorationStatus::Failed;
            self.last_event = ExplorationEvent::Failed;
            return self.last_event;
        };
        self.next_step += 1;

        match step {
            SolverStep::Visit(pos) => {
                self.current = pos;
                self.visited[maze.index(pos)] = true;
                self.step_count += 1;
                self.last_event = ExplorationEvent::Visit(pos);
            }
            SolverStep::Move { from, to } => {
                self.current = to;
                self.visited[maze.index(to)] = true;
                self.step_count += 1;
                self.last_event = ExplorationEvent::Move { from, to };
            }
            SolverStep::Backtrack { from, to } => {
                self.current = to;
                self.step_count += 1;
                self.last_event = ExplorationEvent::Backtrack { from, to };
            }
            SolverStep::Solved(path) => {
                self.final_path = path;
                self.status = ExplorationStatus::Solved;
                self.last_event = ExplorationEvent::Solved;
            }
            SolverStep::Failed => {
                self.status = ExplorationStatus::Failed;
                self.last_event = ExplorationEvent::Failed;
            }
        }

        self.last_event
    }
}

#[derive(Clone, Debug)]
enum SolverStep {
    Visit(Pos),
    Move { from: Pos, to: Pos },
    Backtrack { from: Pos, to: Pos },
    Solved(Vec<Pos>),
    Failed,
}

fn plan_steps(maze: &Maze, algorithm: SolverAlgorithm) -> Vec<SolverStep> {
    match algorithm {
        SolverAlgorithm::Dfs => plan_dfs(maze),
        SolverAlgorithm::Bfs => plan_bfs(maze),
        SolverAlgorithm::Astar => plan_astar(maze),
        SolverAlgorithm::DeadEnd => plan_dead_end(maze),
    }
}

fn plan_dfs(maze: &Maze) -> Vec<SolverStep> {
    let mut steps = Vec::new();
    let mut current = maze.start();
    let mut stack = Vec::new();
    let mut visited = vec![false; maze.len()];
    let mut parent = vec![None; maze.len()];
    visited[maze.index(current)] = true;

    loop {
        if current == maze.exit() {
            push_solution(maze, &parent, &mut steps);
            return steps;
        }

        if let Some((_, next)) = maze
            .reachable_neighbors(current)
            .find(|(_, pos)| !visited[maze.index(*pos)])
        {
            stack.push(current);
            parent[maze.index(next)] = Some(current);
            visited[maze.index(next)] = true;
            steps.push(SolverStep::Move {
                from: current,
                to: next,
            });
            current = next;
            continue;
        }

        let Some(previous) = stack.pop() else {
            steps.push(SolverStep::Failed);
            return steps;
        };

        steps.push(SolverStep::Backtrack {
            from: current,
            to: previous,
        });
        current = previous;
    }
}

fn plan_bfs(maze: &Maze) -> Vec<SolverStep> {
    let mut steps = Vec::new();
    let mut queue = VecDeque::from([maze.start()]);
    let mut visited = vec![false; maze.len()];
    let mut parent = vec![None; maze.len()];
    visited[maze.index(maze.start())] = true;

    while let Some(from) = queue.pop_front() {
        if from == maze.exit() {
            push_solution(maze, &parent, &mut steps);
            return steps;
        }

        for (_, next) in maze.reachable_neighbors(from) {
            if visited[maze.index(next)] {
                continue;
            }

            visited[maze.index(next)] = true;
            parent[maze.index(next)] = Some(from);
            queue.push_back(next);
            steps.push(SolverStep::Move { from, to: next });

            if next == maze.exit() {
                push_solution(maze, &parent, &mut steps);
                return steps;
            }
        }
    }

    steps.push(SolverStep::Failed);
    steps
}

fn plan_astar(maze: &Maze) -> Vec<SolverStep> {
    let mut steps = Vec::new();
    let mut open = vec![maze.start()];
    let mut closed = vec![false; maze.len()];
    let mut best_cost = vec![usize::MAX; maze.len()];
    let mut parent = vec![None; maze.len()];
    best_cost[maze.index(maze.start())] = 0;

    while !open.is_empty() {
        let best_index = (0..open.len())
            .min_by_key(|index| {
                let pos = open[*index];
                let cost = best_cost[maze.index(pos)];
                (
                    cost + manhattan(pos, maze.exit()),
                    manhattan(pos, maze.exit()),
                )
            })
            .expect("open set is not empty");
        let current = open.swap_remove(best_index);
        let current_index = maze.index(current);

        if closed[current_index] {
            continue;
        }

        closed[current_index] = true;
        if current != maze.start() {
            steps.push(SolverStep::Visit(current));
        }

        if current == maze.exit() {
            push_solution(maze, &parent, &mut steps);
            return steps;
        }

        for (_, next) in maze.reachable_neighbors(current) {
            let next_index = maze.index(next);
            let tentative = best_cost[current_index] + 1;

            if tentative < best_cost[next_index] {
                best_cost[next_index] = tentative;
                parent[next_index] = Some(current);
                open.push(next);
            }
        }
    }

    steps.push(SolverStep::Failed);
    steps
}

fn plan_dead_end(maze: &Maze) -> Vec<SolverStep> {
    let mut steps = Vec::new();
    let mut active = vec![true; maze.len()];
    let mut queue = VecDeque::new();

    for row in 0..maze.height() {
        for col in 0..maze.width() {
            let pos = Pos::new(row, col);
            if is_terminal(maze, pos) {
                continue;
            }

            if active_degree(maze, &active, pos) <= 1 {
                queue.push_back(pos);
            }
        }
    }

    while let Some(pos) = queue.pop_front() {
        let index = maze.index(pos);
        if !active[index] || is_terminal(maze, pos) || active_degree(maze, &active, pos) > 1 {
            continue;
        }

        active[index] = false;
        steps.push(SolverStep::Visit(pos));

        for (_, neighbor) in maze.reachable_neighbors(pos) {
            if !is_terminal(maze, neighbor)
                && active[maze.index(neighbor)]
                && active_degree(maze, &active, neighbor) <= 1
            {
                queue.push_back(neighbor);
            }
        }
    }

    if let Some(path) = remaining_path(maze, &active) {
        steps.push(SolverStep::Solved(path));
    } else {
        steps.push(SolverStep::Failed);
    }

    steps
}

fn push_solution(maze: &Maze, parent: &[Option<Pos>], steps: &mut Vec<SolverStep>) {
    if let Some(path) = reconstruct_path(maze, parent) {
        steps.push(SolverStep::Solved(path));
    } else {
        steps.push(SolverStep::Failed);
    }
}

fn reconstruct_path(maze: &Maze, parent: &[Option<Pos>]) -> Option<Vec<Pos>> {
    let mut path = Vec::new();
    let mut cursor = maze.exit();
    path.push(cursor);

    while cursor != maze.start() {
        cursor = parent[maze.index(cursor)]?;
        path.push(cursor);
    }

    path.reverse();
    Some(path)
}

fn remaining_path(maze: &Maze, active: &[bool]) -> Option<Vec<Pos>> {
    let mut path = vec![maze.start()];
    let mut seen = vec![false; maze.len()];
    let mut previous = None;
    let mut current = maze.start();

    while current != maze.exit() {
        seen[maze.index(current)] = true;
        let next = maze
            .reachable_neighbors(current)
            .find_map(|(_, neighbor)| {
                (active[maze.index(neighbor)]
                    && Some(neighbor) != previous
                    && !seen[maze.index(neighbor)])
                .then_some(neighbor)
            })?;

        previous = Some(current);
        current = next;
        path.push(current);
    }

    Some(path)
}

fn active_degree(maze: &Maze, active: &[bool], pos: Pos) -> usize {
    maze.reachable_neighbors(pos)
        .filter(|(_, neighbor)| active[maze.index(*neighbor)])
        .count()
}

fn is_terminal(maze: &Maze, pos: Pos) -> bool {
    pos == maze.start() || pos == maze.exit()
}

fn manhattan(left: Pos, right: Pos) -> usize {
    left.row.abs_diff(right.row) + left.col.abs_diff(right.col)
}
