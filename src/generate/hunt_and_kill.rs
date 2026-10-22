use super::{Dims, HuntOrder, MazeGenerator};
use crate::{midpoint, Cell, Grid, MazeError, Rng};

/// The Hunt-and-Kill algorithm.
///
/// 1. Randomly choose a starting cell.
/// 2. Random-walk, carving passages to unvisited neighbors, until stuck.
/// 3. Hunt for a new cell (randomly or in serpentine order); if it is
///    visited, walk from it.
/// 4. Repeat until (very probably) every cell is visited.
#[derive(Clone, Debug)]
pub struct HuntAndKill {
    dims: Dims,
    hunt_order: HuntOrder,
}

impl HuntAndKill {
    /// A generator for an `h` x `w` maze.
    pub fn new(h: i64, w: i64, hunt_order: HuntOrder) -> Result<Self, MazeError> {
        Ok(HuntAndKill {
            dims: Dims::new(h, w)?,
            hunt_order,
        })
    }

    fn walk(&self, grid: &mut Grid, start: Cell, rng: &mut Rng) -> Result<(), MazeError> {
        if grid.at(start) != 0 {
            return Ok(());
        }
        let d = self.dims;
        let mut current = start;
        let mut unvisited = d.find_neighbors(current, grid, true, rng);
        while !unvisited.is_empty() {
            let neighbor = rng.py.choice(&unvisited)?;
            grid.set(neighbor, 0);
            grid.set(midpoint(neighbor, current), 0);
            current = neighbor;
            unvisited = d.find_neighbors(current, grid, true, rng);
        }
        Ok(())
    }

    fn hunt(&self, grid: &Grid, count: i64, rng: &mut Rng) -> Result<Cell, MazeError> {
        let d = self.dims;
        match self.hunt_order {
            HuntOrder::Serpentine => {
                let (mut row, mut col) = (1, 1);
                loop {
                    col += 2;
                    if col > d.ww - 2 {
                        row += 2;
                        col = 1;
                        if row > d.hh - 2 {
                            return Ok((-1, -1));
                        }
                    }
                    if grid.at((row, col)) == 0
                        && !d.find_neighbors((row, col), grid, true, rng).is_empty()
                    {
                        return Ok((row, col));
                    }
                }
            }
            HuntOrder::Random => {
                if count >= d.hh * d.ww {
                    Ok((-1, -1))
                } else {
                    d.random_cell(rng)
                }
            }
        }
    }
}

impl MazeGenerator for HuntAndKill {
    fn generate(&mut self, rng: &mut Rng) -> Result<Grid, MazeError> {
        let mut grid = self.dims.grid(1);
        let mut current = self.dims.random_cell(rng)?;
        grid.set(current, 0);

        let mut num_trials = 0;
        while current != (-1, -1) {
            self.walk(&mut grid, current, rng)?;
            current = self.hunt(&grid, num_trials, rng)?;
            num_trials += 1;
        }
        Ok(grid)
    }
}
