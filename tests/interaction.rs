use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use mazetrace::app::{App, Phase};
use mazetrace::config::{Config, GeneratorAlgorithm, SolverAlgorithm};

#[test]
fn quit_keys_exit_application() {
    let mut app = App::new(test_config(), 80, 30);

    assert!(app.handle_key(key(KeyCode::Char('q'))));
    assert!(app.handle_key(key(KeyCode::Char('Q'))));
    assert!(app.handle_key(key(KeyCode::Esc)));
    assert!(app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)));
}

#[test]
fn space_starts_ready_maze_and_toggles_pause_afterward() {
    let mut app = App::new(test_config(), 80, 30);
    finish_generation(&mut app);

    assert_eq!(app.phase(), Phase::Ready);
    assert!(app.paused());

    assert!(!app.handle_key(key(KeyCode::Char(' '))));
    assert_eq!(app.phase(), Phase::Exploring);
    assert!(!app.paused());

    app.handle_key(key(KeyCode::Char(' ')));
    assert!(app.paused());

    app.handle_key(key(KeyCode::Char(' ')));
    assert!(!app.paused());
}

#[test]
fn speed_keys_clamp_to_supported_range() {
    let mut fast = App::new(test_config_with_speed(235), 80, 30);

    fast.handle_key(key(KeyCode::Char('+')));
    assert_eq!(fast.speed(), 240);
    fast.handle_key(key(KeyCode::Char('=')));
    assert_eq!(fast.speed(), 240);

    let mut slow = App::new(test_config_with_speed(5), 80, 30);

    slow.handle_key(key(KeyCode::Char('-')));
    assert_eq!(slow.speed(), 1);
    slow.handle_key(key(KeyCode::Char('_')));
    assert_eq!(slow.speed(), 1);
}

#[test]
fn number_keys_switch_solver_and_reset_exploration() {
    let mut app = App::new(test_config(), 80, 30);
    finish_generation(&mut app);

    for (key_char, solver) in [
        ('1', SolverAlgorithm::Dfs),
        ('2', SolverAlgorithm::Bfs),
        ('3', SolverAlgorithm::Astar),
        ('4', SolverAlgorithm::Dijkstra),
        ('5', SolverAlgorithm::DeadEnd),
        ('6', SolverAlgorithm::WallFollower),
    ] {
        app.handle_key(key(KeyCode::Char(key_char)));

        assert_eq!(app.solver_algorithm(), solver);
        assert_eq!(app.phase(), Phase::Ready);
        assert!(app.paused());
        assert!(app.message().contains("Press Space to start"));
    }
}

#[test]
fn replay_is_rejected_while_generation_is_running() {
    let mut app = App::new(test_config(), 80, 30);

    assert_eq!(app.phase(), Phase::Generating);

    app.handle_key(key(KeyCode::Char('R')));

    assert_eq!(app.phase(), Phase::Generating);
    assert!(app.message().contains("Finish generation"));
}

#[test]
fn new_maze_key_starts_fresh_generation() {
    let mut app = App::new(test_config(), 80, 30);

    assert_eq!(app.seed(), 1);

    app.handle_key(key(KeyCode::Char('N')));

    assert_eq!(app.phase(), Phase::Generating);
    assert_eq!(app.seed(), 2);
}

#[test]
fn resize_pauses_when_maze_no_longer_fits_without_changing_maze_size() {
    let mut app = App::new(test_config(), 80, 30);
    finish_generation(&mut app);
    let original_size = (app.maze().width(), app.maze().height());

    app.handle_resize(10, 6);

    assert!(!app.fits());
    assert!(app.paused());
    assert!(app.message().contains("too small"));
    assert_eq!((app.maze().width(), app.maze().height()), original_size);

    app.handle_resize(120, 60);

    assert!(app.fits());
    assert!(app.message().contains("Press N"));
    assert_eq!((app.maze().width(), app.maze().height()), original_size);
}

fn finish_generation(app: &mut App) {
    let max_steps = app.maze().len() * 2_000;

    for _ in 0..max_steps {
        if app.phase() != Phase::Generating {
            break;
        }
        app.step_once();
    }

    assert_ne!(app.phase(), Phase::Generating);
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn test_config() -> Config {
    test_config_with_speed(60)
}

fn test_config_with_speed(speed: u64) -> Config {
    Config {
        width: Some(5),
        height: Some(5),
        speed,
        generator: GeneratorAlgorithm::Dfs,
        solver: SolverAlgorithm::Dfs,
        algorithm: None,
        auto_start: false,
        ascii: false,
        seed: Some(1),
    }
}
