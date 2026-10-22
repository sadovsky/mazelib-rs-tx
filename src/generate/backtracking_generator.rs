use super::{Dims, MazeGenerator};
use crate::{midpoint, Grid, MazeError, Rng};

/// The Backtracking (recursive backtracker) algorithm.
///
/// 1. Randomly choose a starting cell.
/// 2. Open a passage to a random unvisited neighbor; it becomes current.
/// 3. If every neighbor is visited, back up to the previous cell.
/// 4. Stop when the walk has backed up to the start.
#[derive(Clone, Debug)]
pub struct BacktrackingGenerator {
    dims: Dims,
}

impl BacktrackingGenerator {
    /// A generator for an `h` x `w` maze.
    pub fn new(h: i64, w: i64) -> Result<Self, MazeError> {
        Ok(BacktrackingGenerator {
            dims: Dims::new(h, w)?,
        })
    }
}

impl MazeGenerator for BacktrackingGenerator {
    fn generate(&mut self, rng: &mut Rng) -> Result<Grid, MazeError> {
        let d = self.dims;
        let mut grid = d.grid(1);
        let start = d.random_cell(rng)?;
        let mut track = vec![start];
        grid.set(start, 0);

        while let Some(&current) = track.last() {
            let neighbors = d.find_neighbors(current, &grid, true, rng);
            match neighbors.first() {
                None => {
                    track.pop();
                }
                Some(&next) => {
                    grid.set(next, 0);
                    grid.set(midpoint(next, current), 0);
                    track.push(next);
                }
            }
        }
        Ok(grid)
    }
}
