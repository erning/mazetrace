use mazetrace::app::auto_dimensions;
use mazetrace::explorer::{ExplorationStatus, Explorer};
use mazetrace::generator::{GenerationStatus, MazeGenerator};
use mazetrace::maze::{Direction, Maze, Pos};
use mazetrace::render::{render_maze, render_size, RenderPhase};

#[test]
fn carve_between_opens_matching_walls() {
    let mut maze = Maze::new(2, 1);
    let left = Pos::new(0, 0);
    let right = Pos::new(0, 1);

    maze.carve_between(left, right);

    assert!(!maze.has_wall(left, Direction::East));
    assert!(!maze.has_wall(right, Direction::West));
}

#[test]
fn generator_completes_and_opens_entrance_and_exit() {
    let mut maze = Maze::new(6, 5);
    let mut generator = MazeGenerator::new(&maze, 42);

    for _ in 0..(maze.len() * 3) {
        if generator.status() == GenerationStatus::Done {
            break;
        }
        generator.step(&mut maze);
    }

    assert_eq!(generator.status(), GenerationStatus::Done);
    assert!(!maze.has_wall(maze.start(), Direction::West));
    assert!(!maze.has_wall(maze.exit(), Direction::East));
}

#[test]
fn explorer_solves_generated_maze() {
    let mut maze = Maze::new(8, 6);
    let mut generator = MazeGenerator::new(&maze, 7);
    while !generator.is_done() {
        generator.step(&mut maze);
    }

    let mut explorer = Explorer::new(&maze);
    for _ in 0..(maze.len() * 4) {
        if explorer.is_finished() {
            break;
        }
        explorer.step(&maze);
    }

    assert_eq!(explorer.status(), ExplorationStatus::Solved);
    assert_eq!(explorer.final_path().first().copied(), Some(maze.start()));
    assert_eq!(explorer.final_path().last().copied(), Some(maze.exit()));
}

#[test]
fn render_dimensions_follow_design_ratio() {
    assert_eq!(render_size(21, 11), (85, 23));
}

#[test]
fn render_outputs_expected_line_count() {
    let mut maze = Maze::new(5, 5);
    let mut generator = MazeGenerator::new(&maze, 9);
    while !generator.is_done() {
        generator.step(&mut maze);
    }
    let explorer = Explorer::new(&maze);

    let lines = render_maze(&maze, &generator, &explorer, RenderPhase::Exploring, false);

    assert_eq!(lines.len(), 11);
    assert!(lines.iter().all(|line| line.chars().count() == 21));
}

#[test]
fn solved_render_uses_double_line_path() {
    let mut maze = Maze::new(5, 5);
    let mut generator = MazeGenerator::new(&maze, 11);
    while !generator.is_done() {
        generator.step(&mut maze);
    }

    let mut explorer = Explorer::new(&maze);
    while !explorer.is_finished() {
        explorer.step(&maze);
    }

    let lines = render_maze(&maze, &generator, &explorer, RenderPhase::Solved, false);
    let output = lines.join("\n");

    assert!(output
        .chars()
        .any(|ch| matches!(ch, '═' | '║' | '╔' | '╗' | '╚' | '╝')));
    assert!(!output.contains('◆'));
}

#[test]
fn auto_dimensions_keep_minimum_when_terminal_is_tiny() {
    assert_eq!(auto_dimensions(10, 6), (5, 5));
}
