use crate::explorer::Explorer;
use crate::generator::{GenerationStatus, MazeGenerator};
use crate::maze::{Direction, Maze, Pos};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RenderPhase {
    Generating,
    Exploring,
    Solved,
    Failed,
}

pub fn render_size(width: usize, height: usize) -> (usize, usize) {
    (width * 4 + 1, height * 2 + 1)
}

pub fn render_maze(
    maze: &Maze,
    generator: &MazeGenerator,
    explorer: &Explorer,
    phase: RenderPhase,
    ascii: bool,
) -> Vec<String> {
    let (render_width, render_height) = render_size(maze.width(), maze.height());
    let mut canvas = vec![vec![' '; render_width]; render_height];

    draw_walls(maze, &mut canvas, ascii);
    draw_marks(maze, generator, explorer, phase, &mut canvas, ascii);

    canvas
        .into_iter()
        .map(|line| line.into_iter().collect())
        .collect()
}

fn draw_walls(maze: &Maze, canvas: &mut [Vec<char>], ascii: bool) {
    let horizontal = if ascii { '-' } else { '─' };
    let vertical = if ascii { '|' } else { '│' };

    for row in 0..maze.height() {
        for col in 0..maze.width() {
            let pos = Pos::new(row, col);
            let y = row * 2;
            let x = col * 4;

            if maze.has_wall(pos, Direction::North) {
                for dx in 1..=3 {
                    canvas[y][x + dx] = horizontal;
                }
            }
            if maze.has_wall(pos, Direction::South) {
                for dx in 1..=3 {
                    canvas[y + 2][x + dx] = horizontal;
                }
            }
            if maze.has_wall(pos, Direction::West) {
                canvas[y + 1][x] = vertical;
            }
            if maze.has_wall(pos, Direction::East) {
                canvas[y + 1][x + 4] = vertical;
            }
        }
    }

    for row in 0..=maze.height() {
        for col in 0..=maze.width() {
            let y = row * 2;
            let x = col * 4;
            canvas[y][x] = junction_char(
                connection_up(maze, row, col),
                connection_right(maze, row, col),
                connection_down(maze, row, col),
                connection_left(maze, row, col),
                ascii,
            );
        }
    }
}

fn draw_marks(
    maze: &Maze,
    generator: &MazeGenerator,
    explorer: &Explorer,
    phase: RenderPhase,
    canvas: &mut [Vec<char>],
    ascii: bool,
) {
    if generator.status() == GenerationStatus::Done {
        put_cell(canvas, maze.start(), 'S');
        put_cell(canvas, maze.exit(), 'E');
    }

    match phase {
        RenderPhase::Generating => {
            put_cell(canvas, generator.current(), if ascii { '@' } else { '○' });
        }
        RenderPhase::Exploring | RenderPhase::Solved | RenderPhase::Failed => {
            for row in 0..maze.height() {
                for col in 0..maze.width() {
                    let pos = Pos::new(row, col);
                    if explorer.visited(maze, pos) {
                        put_cell(canvas, pos, if ascii { '.' } else { '·' });
                    }
                }
            }

            for pos in explorer.final_path() {
                put_cell(canvas, *pos, if ascii { '*' } else { '◆' });
            }

            put_cell(canvas, maze.start(), 'S');
            put_cell(canvas, maze.exit(), 'E');

            if matches!(phase, RenderPhase::Exploring | RenderPhase::Failed) {
                put_cell(canvas, explorer.current(), if ascii { '@' } else { '●' });
            }
        }
    }
}

fn put_cell(canvas: &mut [Vec<char>], pos: Pos, value: char) {
    canvas[pos.row * 2 + 1][pos.col * 4 + 2] = value;
}

fn connection_up(maze: &Maze, row: usize, col: usize) -> bool {
    if row == 0 {
        return false;
    }

    if col == 0 {
        maze.has_wall(Pos::new(row - 1, 0), Direction::West)
    } else if col == maze.width() {
        maze.has_wall(Pos::new(row - 1, maze.width() - 1), Direction::East)
    } else {
        maze.has_wall(Pos::new(row - 1, col - 1), Direction::East)
            || maze.has_wall(Pos::new(row - 1, col), Direction::West)
    }
}

fn connection_down(maze: &Maze, row: usize, col: usize) -> bool {
    if row == maze.height() {
        return false;
    }

    if col == 0 {
        maze.has_wall(Pos::new(row, 0), Direction::West)
    } else if col == maze.width() {
        maze.has_wall(Pos::new(row, maze.width() - 1), Direction::East)
    } else {
        maze.has_wall(Pos::new(row, col - 1), Direction::East)
            || maze.has_wall(Pos::new(row, col), Direction::West)
    }
}

fn connection_left(maze: &Maze, row: usize, col: usize) -> bool {
    if col == 0 {
        return false;
    }

    if row == 0 {
        maze.has_wall(Pos::new(0, col - 1), Direction::North)
    } else if row == maze.height() {
        maze.has_wall(Pos::new(maze.height() - 1, col - 1), Direction::South)
    } else {
        maze.has_wall(Pos::new(row - 1, col - 1), Direction::South)
            || maze.has_wall(Pos::new(row, col - 1), Direction::North)
    }
}

fn connection_right(maze: &Maze, row: usize, col: usize) -> bool {
    if col == maze.width() {
        return false;
    }

    if row == 0 {
        maze.has_wall(Pos::new(0, col), Direction::North)
    } else if row == maze.height() {
        maze.has_wall(Pos::new(maze.height() - 1, col), Direction::South)
    } else {
        maze.has_wall(Pos::new(row - 1, col), Direction::South)
            || maze.has_wall(Pos::new(row, col), Direction::North)
    }
}

fn junction_char(up: bool, right: bool, down: bool, left: bool, ascii: bool) -> char {
    if ascii {
        return if up || right || down || left {
            '+'
        } else {
            ' '
        };
    }

    match (up, right, down, left) {
        (false, false, false, false) => ' ',
        (true, false, true, false) => '│',
        (false, true, false, true) => '─',
        (false, true, true, false) => '┌',
        (false, false, true, true) => '┐',
        (true, true, false, false) => '└',
        (true, false, false, true) => '┘',
        (true, true, true, false) => '├',
        (true, false, true, true) => '┤',
        (false, true, true, true) => '┬',
        (true, true, false, true) => '┴',
        (true, true, true, true) => '┼',
        (true, false, false, false) | (false, false, true, false) => '│',
        (false, true, false, false) | (false, false, false, true) => '─',
    }
}
