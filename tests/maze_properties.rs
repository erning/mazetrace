use mazetrace::config::{GeneratorAlgorithm, SolverAlgorithm};
use mazetrace::explorer::{ExplorationStatus, Explorer};
use mazetrace::generator::{GenerationStatus, MazeGenerator};
use mazetrace::maze::{Direction, Maze, Pos};

#[test]
fn generators_produce_connected_acyclic_mazes_with_consistent_walls() {
    for algorithm in [
        GeneratorAlgorithm::Dfs,
        GeneratorAlgorithm::Prim,
        GeneratorAlgorithm::Kruskal,
        GeneratorAlgorithm::AldousBroder,
        GeneratorAlgorithm::Wilson,
        GeneratorAlgorithm::RecursiveDivision,
    ] {
        let maze = generated_maze(algorithm);

        assert_walls_are_symmetric(&maze);
        assert_internal_passages_form_tree(&maze);
    }
}

fn generated_maze(algorithm: GeneratorAlgorithm) -> Maze {
    let mut maze = Maze::new(6, 5);
    let mut generator = MazeGenerator::with_algorithm(&maze, algorithm, 42);

    for _ in 0..(maze.len() * 2_000) {
        if generator.status() == GenerationStatus::Done {
            break;
        }
        generator.step(&mut maze);
    }

    assert_eq!(generator.status(), GenerationStatus::Done);
    maze
}

fn braided_maze(algorithm: GeneratorAlgorithm, ratio: f64) -> Maze {
    let mut maze = Maze::new(8, 6);
    let mut generator = MazeGenerator::with_braid(&maze, algorithm, 42, ratio);

    for _ in 0..(maze.len() * 2_000) {
        if generator.status() == GenerationStatus::Done {
            break;
        }
        generator.step(&mut maze);
    }

    assert_eq!(generator.status(), GenerationStatus::Done);
    maze
}

fn assert_walls_are_symmetric(maze: &Maze) {
    for row in 0..maze.height() {
        for col in 0..maze.width() {
            let pos = Pos::new(row, col);

            for direction in [Direction::East, Direction::South] {
                if let Some(next) = maze.neighbor(pos, direction) {
                    assert_eq!(
                        maze.has_wall(pos, direction),
                        maze.has_wall(next, direction.opposite())
                    );
                }
            }
        }
    }
}

fn assert_internal_passages_form_tree(maze: &Maze) {
    let mut disjoint_set = TestDisjointSet::new(maze.len());
    let mut passage_count = 0;

    for row in 0..maze.height() {
        for col in 0..maze.width() {
            let pos = Pos::new(row, col);

            for direction in [Direction::East, Direction::South] {
                if let Some(next) = maze.neighbor(pos, direction) {
                    if !maze.has_wall(pos, direction) {
                        passage_count += 1;
                        assert!(disjoint_set.union(maze.index(pos), maze.index(next)));
                    }
                }
            }
        }
    }

    assert_eq!(passage_count, maze.len() - 1);

    let root = disjoint_set.find(0);
    for index in 1..maze.len() {
        assert_eq!(disjoint_set.find(index), root);
    }
}

#[test]
fn braided_mazes_stay_connected_with_cycles() {
    for algorithm in [
        GeneratorAlgorithm::Dfs,
        GeneratorAlgorithm::Prim,
        GeneratorAlgorithm::Kruskal,
    ] {
        let maze = braided_maze(algorithm, 0.6);

        assert_passages_form_one_connected_component(&maze);

        // Braiding only removes walls, never adds them, so a perfect maze's
        // single component stays connected while cycles push the passage count
        // above the spanning-tree edge count.
        let passages = count_passages(&maze);
        assert!(
            passages > maze.len() - 1,
            "{} braided maze should have cycles",
            algorithm.label()
        );
    }
}

#[test]
fn robust_solvers_solve_braided_maze() {
    // Dead-end filling is excluded: it assumes a perfect maze and may report
    // no path once braiding adds loops. The remaining four solvers are general
    // graph algorithms and must always find a route.
    for algorithm in [
        GeneratorAlgorithm::Dfs,
        GeneratorAlgorithm::Prim,
        GeneratorAlgorithm::Kruskal,
    ] {
        let maze = braided_maze(algorithm, 0.8);

        for solver in [
            SolverAlgorithm::Dfs,
            SolverAlgorithm::Bfs,
            SolverAlgorithm::Astar,
            SolverAlgorithm::Dijkstra,
        ] {
            let mut explorer = Explorer::with_algorithm(&maze, solver);
            for _ in 0..(maze.len() * 20) {
                if explorer.is_finished() {
                    break;
                }
                explorer.step(&maze);
            }

            assert_eq!(
                explorer.status(),
                ExplorationStatus::Solved,
                "{:?} failed on braided {:?}",
                solver,
                algorithm
            );
            assert_eq!(explorer.final_path().first().copied(), Some(maze.start()));
            assert_eq!(explorer.final_path().last().copied(), Some(maze.exit()));
        }
    }
}

fn assert_passages_form_one_connected_component(maze: &Maze) {
    let mut disjoint_set = TestDisjointSet::new(maze.len());

    for row in 0..maze.height() {
        for col in 0..maze.width() {
            let pos = Pos::new(row, col);
            for direction in [Direction::East, Direction::South] {
                if let Some(next) = maze.neighbor(pos, direction) {
                    if !maze.has_wall(pos, direction) {
                        disjoint_set.union(maze.index(pos), maze.index(next));
                    }
                }
            }
        }
    }

    let root = disjoint_set.find(0);
    for index in 1..maze.len() {
        assert_eq!(
            disjoint_set.find(index),
            root,
            "maze is not fully connected"
        );
    }
}

fn count_passages(maze: &Maze) -> usize {
    let mut passages = 0;
    for row in 0..maze.height() {
        for col in 0..maze.width() {
            let pos = Pos::new(row, col);
            for direction in [Direction::East, Direction::South] {
                if maze.neighbor(pos, direction).is_some() && !maze.has_wall(pos, direction) {
                    passages += 1;
                }
            }
        }
    }
    passages
}

struct TestDisjointSet {
    parent: Vec<usize>,
}

impl TestDisjointSet {
    fn new(size: usize) -> Self {
        Self {
            parent: (0..size).collect(),
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

        self.parent[right_root] = left_root;
        true
    }
}
