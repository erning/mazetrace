use clap::CommandFactory;
use mazetrace::app::{auto_dimensions, App, Phase};
use mazetrace::config::{Config, GeneratorAlgorithm, SolverAlgorithm};
use mazetrace::explorer::{ExplorationStatus, Explorer};
use mazetrace::generator::{GenerationStatus, MazeGenerator};
use mazetrace::maze::{Direction, Maze, Pos};
use mazetrace::render::{render_maze, render_maze_cells, render_size, RenderKind, RenderPhase};

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
    for algorithm in [
        GeneratorAlgorithm::Dfs,
        GeneratorAlgorithm::Prim,
        GeneratorAlgorithm::Kruskal,
        GeneratorAlgorithm::AldousBroder,
        GeneratorAlgorithm::Wilson,
        GeneratorAlgorithm::RecursiveDivision,
    ] {
        let mut maze = Maze::new(6, 5);
        let mut generator = MazeGenerator::with_algorithm(&maze, algorithm, 42);

        for _ in 0..(maze.len() * 2_000) {
            if generator.status() == GenerationStatus::Done {
                break;
            }
            generator.step(&mut maze);
        }

        assert_eq!(generator.status(), GenerationStatus::Done);
        assert!(!maze.has_wall(maze.start(), Direction::West));
        assert!(!maze.has_wall(maze.exit(), Direction::East));
    }
}

#[test]
fn explorer_solves_generated_maze() {
    for algorithm in [
        SolverAlgorithm::Dfs,
        SolverAlgorithm::Bfs,
        SolverAlgorithm::Astar,
        SolverAlgorithm::Dijkstra,
        SolverAlgorithm::DeadEnd,
        SolverAlgorithm::WallFollower,
    ] {
        let mut maze = Maze::new(8, 6);
        let mut generator = MazeGenerator::new(&maze, 7);
        while !generator.is_done() {
            generator.step(&mut maze);
        }

        let mut explorer = Explorer::with_algorithm(&maze, algorithm);
        for _ in 0..(maze.len() * 10) {
            if explorer.is_finished() {
                break;
            }
            explorer.step(&maze);
        }

        assert_eq!(explorer.status(), ExplorationStatus::Solved);
        assert_eq!(explorer.final_path().first().copied(), Some(maze.start()));
        assert_eq!(explorer.final_path().last().copied(), Some(maze.exit()));
    }
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
fn solved_render_marks_final_path_cells() {
    let mut maze = Maze::new(5, 5);
    let mut generator = MazeGenerator::new(&maze, 13);
    while !generator.is_done() {
        generator.step(&mut maze);
    }

    let mut explorer = Explorer::new(&maze);
    while !explorer.is_finished() {
        explorer.step(&maze);
    }

    let cells = render_maze_cells(&maze, &generator, &explorer, RenderPhase::Solved, false);

    assert!(cells
        .iter()
        .flatten()
        .any(|cell| cell.kind == RenderKind::FinalPath));
}

#[test]
fn auto_dimensions_keep_minimum_when_terminal_is_tiny() {
    assert_eq!(auto_dimensions(10, 6), (5, 5));
}

#[test]
fn deprecated_algorithm_alias_overrides_solver() {
    let config = test_config(
        GeneratorAlgorithm::Dfs,
        SolverAlgorithm::Dfs,
        Some(SolverAlgorithm::Bfs),
        false,
    );

    assert_eq!(config.solver_algorithm(), SolverAlgorithm::Bfs);
    assert!(config.uses_deprecated_algorithm_alias());
}

#[test]
fn deprecated_algorithm_alias_is_hidden_from_help() {
    let help = Config::command().render_help().to_string();

    assert!(!help.contains("--algorithm"));
}

#[test]
fn app_waits_ready_after_generation_without_auto_start() {
    let mut app = App::new(
        test_config(GeneratorAlgorithm::Prim, SolverAlgorithm::Bfs, None, false),
        80,
        30,
    );

    for _ in 0..500 {
        if app.phase() != Phase::Generating {
            break;
        }
        app.step_once();
    }

    assert_eq!(app.phase(), Phase::Ready);
    assert!(app.paused());
}

#[test]
fn step_once_from_ready_starts_and_advances_exploration() {
    let mut app = App::new(
        test_config(GeneratorAlgorithm::Prim, SolverAlgorithm::Bfs, None, false),
        80,
        30,
    );

    for _ in 0..500 {
        if app.phase() != Phase::Generating {
            break;
        }
        app.step_once();
    }

    assert_eq!(app.phase(), Phase::Ready);
    assert_eq!(app.explorer().step_count(), 0);

    app.step_once();

    assert_eq!(app.phase(), Phase::Exploring);
    assert_eq!(app.explorer().step_count(), 1);
}

#[test]
fn app_auto_start_moves_from_generation_to_exploring() {
    let mut app = App::new(
        test_config(
            GeneratorAlgorithm::Kruskal,
            SolverAlgorithm::Astar,
            None,
            true,
        ),
        80,
        30,
    );

    for _ in 0..500 {
        if app.phase() != Phase::Generating {
            break;
        }
        app.step_once();
    }

    assert_eq!(app.phase(), Phase::Exploring);
    assert!(!app.paused());
}

fn test_config(
    generator: GeneratorAlgorithm,
    solver: SolverAlgorithm,
    algorithm: Option<SolverAlgorithm>,
    auto_start: bool,
) -> Config {
    Config {
        width: Some(5),
        height: Some(5),
        speed: 60,
        generator,
        solver,
        algorithm,
        auto_start,
        ascii: false,
        seed: Some(1),
        braid: 0.0,
    }
}
