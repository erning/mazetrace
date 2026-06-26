use mazetrace::config::GeneratorAlgorithm;
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
