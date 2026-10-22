use super::{Dims, HuntOrder, MazeGenerator};
use crate::{midpoint, Cell, Grid, MazeError, Rng};
use std::collections::HashMap;

/// Wilson's algorithm (loop-erased random walks).
///
/// 1. Add a random cell to the uniform spanning tree.
/// 2. From a cell outside the tree, random-walk until reaching the tree,
///    remembering only the last exit direction from each cell.
/// 3. Add the (loop-erased) walk to the tree; repeat until it spans the maze.
#[derive(Clone, Debug)]
pub struct Wilsons {
    dims: Dims,
    hunt_order: HuntOrder,
}

impl Wilsons {
    /// A generator for an `h` x `w` maze.
    pub fn new(h: i64, w: i64, hunt_order: HuntOrder) -> Result<Self, MazeError> {
        Ok(Wilsons {
            dims: Dims::new(h, w)?,
            hunt_order,
        })
    }

    fn hunt(&self, grid: &Grid, count: i64, rng: &mut Rng) -> Result<Cell, MazeError> {
        let d = self.dims;
        match self.hunt_order {
            HuntOrder::Serpentine => {
                let mut cell = (1, -1);
                loop {
                    cell = (cell.0, cell.1 + 2);
                    if cell.1 > d.ww - 2 {
                        cell = (cell.0 + 2, 1);
                        if cell.0 > d.hh - 2 {
                            return Ok((-1, -1));
                        }
                    }
                    if grid.at(cell) != 0 {
                        return Ok(cell);
                    }
                }
            }
            HuntOrder::Random => {
                if count >= d.h * d.w {
                    Ok((-1, -1))
                } else {
                    d.random_cell(rng)
                }
            }
        }
    }

    /// One random step that stays inside the maze.
    fn random_dir(&self, (r, c): Cell, rng: &mut Rng) -> Result<Cell, MazeError> {
        let d = self.dims;
        let mut options = Vec::with_capacity(4);
        if r > 1 {
            options.push((-2, 0)); // North
        }
        if r < d.hh - 2 {
            options.push((2, 0)); // South
        }
        if c > 1 {
            options.push((0, -2)); // West
        }
        if c < d.ww - 2 {
            options.push((0, 2)); // East
        }
        rng.py.choice(&options)
    }

    /// Walk from `start` until hitting a visited cell; returns the last
    /// direction taken out of each cell.
    fn random_walk(
        &self,
        grid: &Grid,
        start: Cell,
        rng: &mut Rng,
    ) -> Result<HashMap<Cell, Cell>, MazeError> {
        let mut walk = HashMap::new();
        let mut direction = self.random_dir(start, rng)?;
        walk.insert(start, direction);
        let mut current = (start.0 + direction.0, start.1 + direction.1);
        while grid.at(current) == 1 {
            direction = self.random_dir(current, rng)?;
            walk.insert(current, direction);
            current = (current.0 + direction.0, current.1 + direction.1);
        }
        Ok(walk)
    }

    /// Follow the walk from `start`, carving as it goes; returns the number
    /// of cells visited.
    fn solve_random_walk(
        &self,
        grid: &mut Grid,
        walk: &HashMap<Cell, Cell>,
        start: Cell,
    ) -> Result<i64, MazeError> {
        let mut visits = 0;
        let mut current = start;
        while grid.at(current) != 0 {
            grid.set(current, 0);
            let dir = walk
                .get(&current)
                .ok_or_else(|| MazeError::Index(format!("no walk direction from {current:?}")))?;
            let next = (current.0 + dir.0, current.1 + dir.1);
            grid.set(midpoint(next, current), 0);
            visits += 1;
            current = next;
        }
        Ok(visits)
    }
}

impl MazeGenerator for Wilsons {
    fn generate(&mut self, rng: &mut Rng) -> Result<Grid, MazeError> {
        let mut grid = self.dims.grid(1);
        let first = self.dims.random_cell(rng)?;
        grid.set(first, 0);
        let mut num_visited = 1;
        let mut cell = self.hunt(&grid, num_visited, rng)?;

        while cell.0 != -1 && cell.1 != -1 {
            let walk = self.random_walk(&grid, cell, rng)?;
            num_visited += self.solve_random_walk(&mut grid, &walk, cell)?;
            cell = self.hunt(&grid, num_visited, rng)?;
        }
        Ok(grid)
    }
}
