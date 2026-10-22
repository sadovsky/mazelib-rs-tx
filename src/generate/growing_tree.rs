use super::{Dims, MazeGenerator};
use crate::{midpoint, Grid, MazeError, Rng};

/// The Growing Tree algorithm.
///
/// Keep a list of active cells; repeatedly pick one (the newest with
/// probability `backtrack_chance`, otherwise a random one) and carve to an
/// unvisited neighbor, or retire it if it has none.
#[derive(Clone, Debug)]
pub struct GrowingTree {
    dims: Dims,
    backtrack_chance: f64,
}

impl GrowingTree {
    /// A generator for an `h` x `w` maze (default `backtrack_chance`: `1.0`).
    pub fn new(h: i64, w: i64, backtrack_chance: f64) -> Result<Self, MazeError> {
        Ok(GrowingTree {
            dims: Dims::new(h, w)?,
            backtrack_chance,
        })
    }
}

impl MazeGenerator for GrowingTree {
    fn generate(&mut self, rng: &mut Rng) -> Result<Grid, MazeError> {
        let d = self.dims;
        let mut grid = d.grid(1);
        let start = d.random_cell(rng)?;
        grid.set(start, 0);
        let mut active = vec![start];

        while let Some(&last) = active.last() {
            let current = if rng.py.random() < self.backtrack_chance {
                last
            } else {
                rng.py.choice(&active)?
            };

            let next_neighbors = d.find_neighbors(current, &grid, true, rng);
            if next_neighbors.is_empty() {
                active.retain(|&a| a != current);
                continue;
            }

            let nn = rng.py.choice(&next_neighbors)?;
            active.push(nn);
            grid.set(nn, 0);
            grid.set(midpoint(current, nn), 0);
        }
        Ok(grid)
    }
}
