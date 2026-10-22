use super::{Dims, MazeGenerator};
use crate::{midpoint, Grid, MazeError, Rng};

/// The shape of a [`TrivialMaze`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MazeType {
    /// A path spiralling in from the top-left corner (the default).
    #[default]
    Spiral,
    /// A path snaking back and forth.
    Serpentine,
}

impl MazeType {
    /// `"serpentine"` (case-insensitive, trimmed) or else [`MazeType::Spiral`].
    pub fn parse(s: &str) -> Self {
        if s.trim().to_lowercase() == "serpentine" {
            MazeType::Serpentine
        } else {
            MazeType::Spiral
        }
    }
}

/// Simple unicursal mazes (no dead ends): a serpentine or a spiral.
#[derive(Clone, Debug)]
pub struct TrivialMaze {
    dims: Dims,
    maze_type: MazeType,
}

impl TrivialMaze {
    /// A generator for an `h` x `w` maze.
    pub fn new(h: i64, w: i64, maze_type: MazeType) -> Result<Self, MazeError> {
        Ok(TrivialMaze {
            dims: Dims::new(h, w)?,
            maze_type,
        })
    }

    fn serpentine(&self, mut grid: Grid, rng: &mut Rng) -> Result<Grid, MazeError> {
        let vertical_skew = rng.py.randint(0, 1)?;
        let (height, width) = (self.dims.hh, self.dims.ww);
        if vertical_skew != 0 {
            for row in 1..height - 1 {
                for col in (1..width - 1).step_by(2) {
                    grid.set((row, col), 0);
                }
            }
            // minor passages
            for col in (2..width - 1).step_by(4) {
                grid.set((1, col), 0);
            }
            for col in (4..width - 1).step_by(4) {
                grid.set((height - 2, col), 0);
            }
        } else {
            for row in (1..height - 1).step_by(2) {
                for col in 1..width - 1 {
                    grid.set((row, col), 0);
                }
            }
            for row in (2..height - 1).step_by(4) {
                grid.set((row, 1), 0);
            }
            for row in (4..height - 1).step_by(4) {
                grid.set((row, width - 2), 0);
            }
        }
        Ok(grid)
    }

    fn spiral(&self, mut grid: Grid, rng: &mut Rng) -> Result<Grid, MazeError> {
        let clockwise = rng.py.randint(0, 1)?;
        let directions = if clockwise == 1 {
            [(0, -2), (2, 0), (0, 2), (-2, 0)]
        } else {
            [(-2, 0), (0, 2), (2, 0), (0, -2)]
        };

        let mut current = (1, 1);
        grid.set(current, 0);
        let mut next_dir = 0;
        loop {
            let (dr, dc) = directions[next_dir];
            let next_cell = (current.0 + dr, current.1 + dc);
            let ns = self.dims.find_neighbors(current, &grid, true, rng);
            if ns.contains(&next_cell) {
                grid.set(midpoint(current, next_cell), 0);
                grid.set(next_cell, 0);
                current = next_cell;
            } else if ns.is_empty() {
                break;
            } else {
                next_dir = (next_dir + 1) % 4;
            }
        }
        Ok(grid)
    }
}

impl MazeGenerator for TrivialMaze {
    fn generate(&mut self, rng: &mut Rng) -> Result<Grid, MazeError> {
        let grid = self.dims.grid(1);
        match self.maze_type {
            MazeType::Serpentine => self.serpentine(grid, rng),
            MazeType::Spiral => self.spiral(grid, rng),
        }
    }
}
