use super::{backtracking_walk, Ctx, MazeSolver};
use crate::{Cell, Grid, MazeError, Path, Rng};

/// The Backtracking solver: follow random passages, turning back only at
/// dead ends.
#[derive(Clone, Debug, Default)]
pub struct BacktrackingSolver;

impl BacktrackingSolver {
    /// A new solver.
    pub fn new() -> Self {
        BacktrackingSolver
    }
}

impl MazeSolver for BacktrackingSolver {
    fn solve(
        &mut self,
        grid: &Grid,
        start: Cell,
        end: Cell,
        rng: &mut Rng,
    ) -> Result<Vec<Path>, MazeError> {
        let ctx = Ctx::new(grid, start, end)?;
        let path = backtracking_walk(&ctx, vec![ctx.first_cell()], ctx.end, rng)?;
        Ok(vec![path])
    }
}
