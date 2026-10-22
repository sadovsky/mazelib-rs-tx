use super::{Ctx, MazeSolver};
use crate::{midpoint, within_one, Cell, Grid, MazeError, Path, Rng};

/// The Random Mouse solver: wander at random until finding the end. Expect
/// very long solutions with lots of doubling back.
#[derive(Clone, Debug, Default)]
pub struct RandomMouse;

impl RandomMouse {
    /// A new solver.
    pub fn new() -> Self {
        RandomMouse
    }
}

impl MazeSolver for RandomMouse {
    fn solve(
        &mut self,
        grid: &Grid,
        start: Cell,
        end: Cell,
        rng: &mut Rng,
    ) -> Result<Vec<Path>, MazeError> {
        let ctx = Ctx::new(grid, start, end)?;
        let mut solution = vec![ctx.first_cell()];
        loop {
            let last = *solution.last().expect("non-empty");
            if within_one(last, ctx.end) {
                break;
            }
            let ns = ctx.unblocked(last, rng)?;
            let next = rng.py.choice(&ns)?;
            solution.push(midpoint(last, next));
            solution.push(next);
        }
        Ok(vec![solution])
    }
}
