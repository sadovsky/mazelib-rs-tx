use super::shortest_path::{bfs_solve, Mode};
use super::{Ctx, MazeSolver};
use crate::{ensure, Cell, Grid, MazeError, Path, Rng};

/// The Shortest Paths solver: breadth-first search that keeps every path
/// that reaches the end, sorted shortest first.
#[derive(Clone, Debug, Default)]
pub struct ShortestPaths;

impl ShortestPaths {
    /// A new solver.
    pub fn new() -> Self {
        ShortestPaths
    }
}

impl MazeSolver for ShortestPaths {
    fn solve(
        &mut self,
        grid: &Grid,
        start: Cell,
        end: Cell,
        rng: &mut Rng,
    ) -> Result<Vec<Path>, MazeError> {
        let ctx = Ctx::new(grid, start, end)?;
        let end_edge = ctx.on_edge(ctx.end);
        let solutions = bfs_solve(&ctx, end_edge, Mode::All, rng)?;
        ensure(
            solutions.first().is_some_and(|s| !s.is_empty()),
            "No valid solutions found.",
        )?;
        Ok(solutions)
    }
}
