#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Pos {
    pub row: usize,
    pub col: usize,
}

impl Pos {
    pub const fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    pub const ALL: [Direction; 4] = [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ];

    pub const fn opposite(self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::East => Direction::West,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
        }
    }

    const fn index(self) -> usize {
        match self {
            Direction::North => 0,
            Direction::East => 1,
            Direction::South => 2,
            Direction::West => 3,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Cell {
    walls: [bool; 4],
}

impl Cell {
    const fn full() -> Self {
        Self { walls: [true; 4] }
    }

    pub fn has_wall(&self, direction: Direction) -> bool {
        self.walls[direction.index()]
    }

    fn set_wall(&mut self, direction: Direction, present: bool) {
        self.walls[direction.index()] = present;
    }
}

#[derive(Clone, Debug)]
pub struct Maze {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

impl Maze {
    pub fn new(width: usize, height: usize) -> Self {
        assert!(width > 0, "maze width must be positive");
        assert!(height > 0, "maze height must be positive");

        Self {
            width,
            height,
            cells: vec![Cell::full(); width * height],
        }
    }

    pub const fn width(&self) -> usize {
        self.width
    }

    pub const fn height(&self) -> usize {
        self.height
    }

    pub const fn start(&self) -> Pos {
        Pos::new(0, 0)
    }

    pub fn exit(&self) -> Pos {
        Pos::new(self.height - 1, self.width - 1)
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    pub fn index(&self, pos: Pos) -> usize {
        pos.row * self.width + pos.col
    }

    pub fn contains(&self, pos: Pos) -> bool {
        pos.row < self.height && pos.col < self.width
    }

    pub fn cell(&self, pos: Pos) -> &Cell {
        &self.cells[self.index(pos)]
    }

    pub fn has_wall(&self, pos: Pos, direction: Direction) -> bool {
        self.cell(pos).has_wall(direction)
    }

    pub fn neighbor(&self, pos: Pos, direction: Direction) -> Option<Pos> {
        match direction {
            Direction::North if pos.row > 0 => Some(Pos::new(pos.row - 1, pos.col)),
            Direction::East if pos.col + 1 < self.width => Some(Pos::new(pos.row, pos.col + 1)),
            Direction::South if pos.row + 1 < self.height => Some(Pos::new(pos.row + 1, pos.col)),
            Direction::West if pos.col > 0 => Some(Pos::new(pos.row, pos.col - 1)),
            _ => None,
        }
    }

    pub fn neighbors(&self, pos: Pos) -> impl Iterator<Item = (Direction, Pos)> + '_ {
        Direction::ALL.into_iter().filter_map(move |direction| {
            self.neighbor(pos, direction).map(|next| (direction, next))
        })
    }

    pub fn reachable_neighbors(&self, pos: Pos) -> impl Iterator<Item = (Direction, Pos)> + '_ {
        self.neighbors(pos)
            .filter(move |(direction, _)| !self.has_wall(pos, *direction))
    }

    pub fn carve_between(&mut self, from: Pos, to: Pos) -> Option<Direction> {
        self.set_wall_between(from, to, false)
    }

    pub fn add_wall_between(&mut self, from: Pos, to: Pos) -> Option<Direction> {
        self.set_wall_between(from, to, true)
    }

    pub fn set_wall_between(&mut self, from: Pos, to: Pos, present: bool) -> Option<Direction> {
        let direction = self.direction_between(from, to)?;
        let opposite = direction.opposite();
        let from_idx = self.index(from);
        let to_idx = self.index(to);

        self.cells[from_idx].set_wall(direction, present);
        self.cells[to_idx].set_wall(opposite, present);
        Some(direction)
    }

    pub fn open_all_internal_walls(&mut self) {
        for row in 0..self.height {
            for col in 0..self.width {
                let pos = Pos::new(row, col);

                if let Some(east) = self.neighbor(pos, Direction::East) {
                    self.carve_between(pos, east);
                }
                if let Some(south) = self.neighbor(pos, Direction::South) {
                    self.carve_between(pos, south);
                }
            }
        }
    }

    pub fn open_entrance_exit(&mut self) {
        let start = self.start();
        let exit = self.exit();
        let start_idx = self.index(start);
        let exit_idx = self.index(exit);

        self.cells[start_idx].set_wall(Direction::West, false);
        self.cells[exit_idx].set_wall(Direction::East, false);
    }

    fn direction_between(&self, from: Pos, to: Pos) -> Option<Direction> {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direction_between_detects_adjacent_cells() {
        let maze = Maze::new(3, 3);
        let center = Pos::new(1, 1);

        assert_eq!(
            maze.direction_between(center, Pos::new(0, 1)),
            Some(Direction::North)
        );
        assert_eq!(
            maze.direction_between(center, Pos::new(1, 2)),
            Some(Direction::East)
        );
        assert_eq!(
            maze.direction_between(center, Pos::new(2, 1)),
            Some(Direction::South)
        );
        assert_eq!(
            maze.direction_between(center, Pos::new(1, 0)),
            Some(Direction::West)
        );
        assert_eq!(maze.direction_between(center, Pos::new(2, 2)), None);
    }

    #[test]
    fn wall_updates_are_symmetric() {
        let mut maze = Maze::new(2, 1);
        let left = Pos::new(0, 0);
        let right = Pos::new(0, 1);

        maze.carve_between(left, right);
        assert!(!maze.has_wall(left, Direction::East));
        assert!(!maze.has_wall(right, Direction::West));

        maze.add_wall_between(left, right);
        assert!(maze.has_wall(left, Direction::East));
        assert!(maze.has_wall(right, Direction::West));
    }
}
