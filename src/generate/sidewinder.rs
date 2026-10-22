use super::{Dims, MazeGenerator};
use crate::{Cell, Grid, MazeError, Rng};

/// The Sidewinder algorithm.
///
/// Work row by row: extend a run of cells East at random, and whenever the
/// run stops, carve North from a random cell of the run. `skew` below 0.5
/// favors East-West passages, above 0.5 North-South ones.
#[derive(Clone, Debug)]
pub struct Sidewinder {
    dims: Dims,
    skew: f64,
}

impl Sidewinder {
    /// A generator for an `h` x `w` maze (default `skew`: `0.5`).
    pub fn new(h: i64, w: i64, skew: f64) -> Result<Self, MazeError> {
        Ok(Sidewinder {
            dims: Dims::new(h, w)?,
            skew,
        })
    }
}

impl MazeGenerator for Sidewinder {
    fn generate(&mut self, rng: &mut Rng) -> Result<Grid, MazeError> {
        let d = self.dims;
        let mut grid = d.grid(1);

        // the first row is always open: you can't carve North from it
        for col in 1..d.ww - 1 {
            grid.set((1, col), 0);
        }

        for row in (3..d.hh).step_by(2) {
            let mut run: Vec<Cell> = Vec::new();
            for col in (1..d.ww).step_by(2) {
                grid.set((row, col), 0);
                run.push((row, col));

                let carve_east = rng.py.random() > self.skew;
                if carve_east && col < d.ww - 2 {
                    grid.set((row, col + 1), 0);
                } else {
                    let north = rng.py.choice(&run)?;
                    grid.set((north.0 - 1, north.1), 0);
                    run.clear();
                }
            }
        }
        Ok(grid)
    }
}
