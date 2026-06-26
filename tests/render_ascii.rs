use mazetrace::explorer::Explorer;
use mazetrace::generator::MazeGenerator;
use mazetrace::maze::Maze;
use mazetrace::render::{render_maze, RenderPhase};

#[test]
fn ascii_render_omits_unicode_wall_and_path_characters() {
    let mut maze = Maze::new(5, 5);
    let mut generator = MazeGenerator::new(&maze, 11);
    while !generator.is_done() {
        generator.step(&mut maze);
    }

    let mut explorer = Explorer::new(&maze);
    while !explorer.is_finished() {
        explorer.step(&maze);
    }

    let lines = render_maze(&maze, &generator, &explorer, RenderPhase::Solved, true);
    let output = lines.join("\n");

    for character in [
        '─', '│', '┼', '┌', '┐', '└', '┘', '├', '┤', '┬', '┴', '═', '║', '╔', '╗', '╚', '╝',
    ] {
        assert!(!output.contains(character));
    }
}
