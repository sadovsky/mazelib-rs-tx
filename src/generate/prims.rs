use super::{Dims, MazeGenerator};
use crate::pyset::PySet;
use crate::{midpoint, Grid, MazeError, Rng};

/// Prim's algorithm.
///
/// 1. Start from a random cell.
/// 2. Pick a random frontier cell, connect it to a visited neighbor, and add
///    its unvisited neighbors to the frontier.
/// 3. Repeat until every cell is visited.
#[derive(Clone, Debug)]
pub struct Prims {
    dims: Dims,
}

impl Prims {
    /// A generator for an `h` x `w` maze.
    pub fn new(h: i64, w: i64) -> Result<Self, MazeError> {
        Ok(Prims {
            dims: Dims::new(h, w)?,
        })
    }
}

impl MazeGenerator for Prims {
    fn generate(&mut self, rng: &mut Rng) -> Result<Grid, MazeError> {
        let d = self.dims;
        let mut grid = d.grid(1);
        let start = d.random_cell(rng)?;
        grid.set(start, 0);

        let mut neighbors = d.find_neighbors(start, &grid, true, rng);
        let mut visited = 1;
        while visited < d.h * d.w {
            let nn = rng.py.randrange_to(neighbors.len() as i64)? as usize;
            let current = neighbors.remove(nn);
            visited += 1;
            grid.set(current, 0);

            // connect it to a random visited neighbor
            let open = d.find_neighbors(current, &grid, false, rng);
            let nearest = *open
                .first()
                .ok_or_else(|| MazeError::Index("list index out of range".into()))?;
            grid.set(midpoint(current, nearest), 0);

            // the frontier is rebuilt through a Python set, which reorders it
            let unvisited = d.find_neighbors(current, &grid, true, rng);
            neighbors.extend(unvisited);
            neighbors = PySet::from_items(neighbors).to_vec();
        }
        Ok(grid)
    }
}
