use crate::explorer::Explorer;
use crate::generator::{GenerationStatus, MazeGenerator};
use crate::maze::{Direction, Maze, Pos};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RenderPhase {
    Generating,
    Ready,
    Exploring,
    Solved,
    Failed,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RenderKind {
    Empty,
    Wall,
    Start,
    Exit,
    GeneratorCurrent,
    ExplorerCurrent,
    Explored,
    FinalPath,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RenderCell {
    pub ch: char,
    pub kind: RenderKind,
}

impl RenderCell {
    fn new(ch: char, kind: RenderKind) -> Self {
        Self { ch, kind }
    }
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
    render_maze_cells(maze, generator, explorer, phase, ascii)
        .into_iter()
        .map(|line| line.into_iter().map(|cell| cell.ch).collect())
        .collect()
}

pub fn render_maze_cells(
    maze: &Maze,
    generator: &MazeGenerator,
    explorer: &Explorer,
    phase: RenderPhase,
    ascii: bool,
) -> Vec<Vec<RenderCell>> {
    let (render_width, render_height) = render_size(maze.width(), maze.height());
    let empty = RenderCell::new(' ', RenderKind::Empty);
    let mut canvas = vec![vec![empty; render_width]; render_height];

    draw_walls(maze, &mut canvas, ascii);
    draw_marks(maze, generator, explorer, phase, &mut canvas, ascii);

    canvas
}

fn draw_walls(maze: &Maze, canvas: &mut [Vec<RenderCell>], ascii: bool) {
    let horizontal = if ascii { '-' } else { '─' };
    let vertical = if ascii { '|' } else { '│' };

    for row in 0..maze.height() {
        for col in 0..maze.width() {
            let pos = Pos::new(row, col);
            let y = row * 2;
            let x = col * 4;

            if maze.has_wall(pos, Direction::North) {
                for dx in 1..=3 {
                    canvas[y][x + dx] = RenderCell::new(horizontal, RenderKind::Wall);
                }
            }
            if maze.has_wall(pos, Direction::South) {
                for dx in 1..=3 {
                    canvas[y + 2][x + dx] = RenderCell::new(horizontal, RenderKind::Wall);
                }
            }
            if maze.has_wall(pos, Direction::West) {
                canvas[y + 1][x] = RenderCell::new(vertical, RenderKind::Wall);
            }
            if maze.has_wall(pos, Direction::East) {
                canvas[y + 1][x + 4] = RenderCell::new(vertical, RenderKind::Wall);
            }
        }
    }

    for row in 0..=maze.height() {
        for col in 0..=maze.width() {
            let y = row * 2;
            let x = col * 4;
            canvas[y][x] = RenderCell::new(
                junction_char(
                    connection_up(maze, row, col),
                    connection_right(maze, row, col),
                    connection_down(maze, row, col),
                    connection_left(maze, row, col),
                    ascii,
                ),
                RenderKind::Wall,
            );
        }
    }
}

fn draw_marks(
    maze: &Maze,
    generator: &MazeGenerator,
    explorer: &Explorer,
    phase: RenderPhase,
    canvas: &mut [Vec<RenderCell>],
    ascii: bool,
) {
    if generator.status() == GenerationStatus::Done {
        put_cell(canvas, maze.start(), 'S', RenderKind::Start);
        put_cell(canvas, maze.exit(), 'E', RenderKind::Exit);
    }

    match phase {
        RenderPhase::Generating => {
            put_cell(
                canvas,
                generator.current(),
                if ascii { '@' } else { '○' },
                RenderKind::GeneratorCurrent,
            );
        }
        RenderPhase::Ready => {}
        RenderPhase::Exploring | RenderPhase::Solved | RenderPhase::Failed => {
            for row in 0..maze.height() {
                for col in 0..maze.width() {
                    let pos = Pos::new(row, col);
                    if explorer.visited(maze, pos) {
                        put_cell(
                            canvas,
                            pos,
                            if ascii { '.' } else { '·' },
                            RenderKind::Explored,
                        );
                    }
                }
            }

            draw_final_path(canvas, explorer.final_path(), ascii);

            put_cell(canvas, maze.start(), 'S', RenderKind::Start);
            put_cell(canvas, maze.exit(), 'E', RenderKind::Exit);

            if matches!(phase, RenderPhase::Exploring | RenderPhase::Failed) {
                put_cell(
                    canvas,
                    explorer.current(),
                    if ascii { '@' } else { '●' },
                    RenderKind::ExplorerCurrent,
                );
            }
        }
    }
}

fn put_cell(canvas: &mut [Vec<RenderCell>], pos: Pos, value: char, kind: RenderKind) {
    canvas[pos.row * 2 + 1][pos.col * 4 + 2] = RenderCell::new(value, kind);
}

fn draw_final_path(canvas: &mut [Vec<RenderCell>], path: &[Pos], ascii: bool) {
    if path.is_empty() {
        return;
    }

    for pair in path.windows(2) {
        draw_path_segment(canvas, pair[0], pair[1], ascii);
    }

    for (index, pos) in path.iter().enumerate() {
        let previous = index
            .checked_sub(1)
            .and_then(|previous_index| path.get(previous_index))
            .and_then(|previous| path_direction(*pos, *previous));
        let next = path
            .get(index + 1)
            .and_then(|next| path_direction(*pos, *next));

        put_cell(
            canvas,
            *pos,
            path_connector_char(previous, next, ascii),
            RenderKind::FinalPath,
        );
    }
}

fn draw_path_segment(canvas: &mut [Vec<RenderCell>], from: Pos, to: Pos, ascii: bool) {
    let (from_x, from_y) = cell_center(from);
    let (to_x, to_y) = cell_center(to);

    if from_y == to_y {
        let horizontal = if ascii { '-' } else { '═' };
        let start = from_x.min(to_x) + 1;
        let end = from_x.max(to_x);

        for cell in canvas[from_y].iter_mut().take(end).skip(start) {
            *cell = RenderCell::new(horizontal, RenderKind::FinalPath);
        }
    } else if from_x == to_x {
        let vertical = if ascii { '|' } else { '║' };
        let start = from_y.min(to_y) + 1;
        let end = from_y.max(to_y);

        for row in canvas.iter_mut().take(end).skip(start) {
            row[from_x] = RenderCell::new(vertical, RenderKind::FinalPath);
        }
    }
}

fn cell_center(pos: Pos) -> (usize, usize) {
    (pos.col * 4 + 2, pos.row * 2 + 1)
}

fn path_direction(from: Pos, to: Pos) -> Option<Direction> {
    if from.row == to.row && from.col + 1 == to.col {
        Some(Direction::East)
    } else if from.row == to.row && to.col + 1 == from.col {
        Some(Direction::West)
    } else if from.col == to.col && from.row + 1 == to.row {
        Some(Direction::South)
    } else if from.col == to.col && to.row + 1 == from.row {
        Some(Direction::North)
    } else {
        None
    }
}

fn path_connector_char(previous: Option<Direction>, next: Option<Direction>, ascii: bool) -> char {
    let up = matches!(previous, Some(Direction::North)) || matches!(next, Some(Direction::North));
    let right = matches!(previous, Some(Direction::East)) || matches!(next, Some(Direction::East));
    let down = matches!(previous, Some(Direction::South)) || matches!(next, Some(Direction::South));
    let left = matches!(previous, Some(Direction::West)) || matches!(next, Some(Direction::West));

    if ascii {
        return match (up, right, down, left) {
            (true, false, true, false) => '|',
            (false, true, false, true) => '-',
            _ if up || right || down || left => '+',
            _ => '*',
        };
    }

    match (up, right, down, left) {
        (false, false, false, false) => '═',
        (true, false, true, false) => '║',
        (false, true, false, true) => '═',
        (false, true, true, false) => '╔',
        (false, false, true, true) => '╗',
        (true, true, false, false) => '╚',
        (true, false, false, true) => '╝',
        (true, true, true, false) => '╠',
        (true, false, true, true) => '╣',
        (false, true, true, true) => '╦',
        (true, true, false, true) => '╩',
        (true, true, true, true) => '╬',
        (true, false, false, false) | (false, false, true, false) => '║',
        (false, true, false, false) | (false, false, false, true) => '═',
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_size_matches_cell_spacing() {
        assert_eq!(render_size(1, 1), (5, 3));
        assert_eq!(render_size(3, 2), (13, 5));
    }

    #[test]
    fn junction_char_uses_ascii_fallback() {
        assert_eq!(junction_char(true, true, false, false, true), '+');
        assert_eq!(junction_char(false, false, false, false, true), ' ');
    }

    #[test]
    fn path_connector_char_connects_corners_and_ascii() {
        assert_eq!(
            path_connector_char(Some(Direction::North), Some(Direction::East), false),
            '╚'
        );
        assert_eq!(
            path_connector_char(Some(Direction::West), Some(Direction::South), false),
            '╗'
        );
        assert_eq!(
            path_connector_char(Some(Direction::North), Some(Direction::East), true),
            '+'
        );
    }
}
