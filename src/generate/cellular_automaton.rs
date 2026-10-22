use super::{Dims, MazeGenerator};
use crate::{Grid, MazeError, Rng};

/// The Cellular Automaton algorithm: walls grow from random seeds, mostly
/// along the border, like a Game of Life variant.
///
/// `complexity` and `density` at or below 1.0 are scaled by the maze size
/// the first time [`generate`](MazeGenerator::generate) runs (the original
/// stores the scaled values, so later runs reuse them).
#[derive(Clone, Debug)]
pub struct CellularAutomaton {
    dims: Dims,
    complexity: f64,
    density: f64,
}

impl CellularAutomaton {
    /// A generator for an `h` x `w` maze (defaults: `1.0`, `1.0`).
    pub fn new(h: i64, w: i64, complexity: f64, density: f64) -> Result<Self, MazeError> {
        Ok(CellularAutomaton {
            dims: Dims::new(h, w)?,
            complexity,
            density,
        })
    }
}

impl MazeGenerator for CellularAutomaton {
    fn generate(&mut self, rng: &mut Rng) -> Result<Grid, MazeError> {
        let d = self.dims;
        let mut grid = d.grid(0);
        for c in 0..d.ww {
            grid.set((0, c), 1);
            grid.set((d.hh - 1, c), 1);
        }
        for r in 0..d.hh {
            grid.set((r, 0), 1);
            grid.set((r, d.ww - 1), 1);
        }

        // adjust complexity and density relative to maze size
        if self.complexity <= 1.0 {
            self.complexity *= (d.h + d.w) as f64;
        }
        if self.density <= 1.0 {
            self.density *= (d.h * d.w) as f64;
        }

        for i in 0..(2.0 * self.density) as i64 {
            // choose a starting location
            let (mut y, mut x);
            if (i as f64) < self.density {
                // make sure many walls touch the outside of the maze
                if rng.py.choice(&[0, 1])? != 0 {
                    y = rng.py.choice(&[0, d.hh - 1])?;
                    x = rng.py.randrange(0, d.ww, 2)?;
                } else {
                    x = rng.py.choice(&[0, d.ww - 1])?;
                    y = rng.py.randrange(0, d.hh, 2)?;
                }
            } else {
                // fill in any voids left in the maze
                y = rng.py.randrange(0, d.hh, 2)?;
                x = rng.py.randrange(0, d.ww, 2)?;
            }

            // build a wall through the maze
            grid.set((y, x), 1);
            for _ in 0..self.complexity as i64 {
                let walls = d.find_neighbors((y, x), &grid, true, rng);
                if !walls.is_empty() && walls.len() < 4 {
                    let open = d.find_neighbors((y, x), &grid, false, rng);
                    if open.is_empty() {
                        continue;
                    }
                    let (r, c) = rng.py.choice(&open)?;
                    if grid.at((r, c)) == 0 {
                        grid.set((r, c), 1);
                        grid.set((r + (y - r).div_euclid(2), c + (x - c).div_euclid(2)), 1);
                        x = c;
                        y = r;
                    }
                }
            }
        }
        Ok(grid)
    }
}
