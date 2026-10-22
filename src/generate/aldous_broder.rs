use super::{Dims, MazeGenerator};
use crate::{midpoint, Grid, MazeError, Rng};

/// The Aldous-Broder algorithm.
///
/// 1. Choose a random cell.
/// 2. Choose a random neighbor of the current cell and visit it. If the
///    neighbor has not yet been visited, add the traveled edge to the
///    spanning tree.
/// 3. Repeat step 2 until all cells have been visited.
#[derive(Clone, Debug)]
pub struct AldousBroder {
    dims: Dims,
}

impl AldousBroder {
    /// A generator for an `h` x `w` maze.
    pub fn new(h: i64, w: i64) -> Result<Self, MazeError> {
        Ok(AldousBroder {
            dims: Dims::new(h, w)?,
        })
    }
}

impl MazeGenerator for AldousBroder {
    fn generate(&mut self, rng: &mut Rng) -> Result<Grid, MazeError> {
        let d = self.dims;
        let mut grid = d.grid(1);
        let mut current = d.random_cell(rng)?;
        grid.set(current, 0);
        let mut num_visited = 1;

        while num_visited < d.h * d.w {
            let neighbors = d.find_neighbors(current, &grid, true, rng);
            if neighbors.is_empty() {
                // every neighbor is visited: move to a random one
                let open = d.find_neighbors(current, &grid, false, rng);
                current = rng.py.choice(&open)?;
                continue;
            }
            // the first unvisited neighbor becomes current
            if let Some(&next) = neighbors.iter().find(|&&n| grid.at(n) > 0) {
                grid.set(midpoint(next, current), 0);
                grid.set(next, 0);
                num_visited += 1;
                current = next;
            }
        }
        Ok(grid)
    }
}
