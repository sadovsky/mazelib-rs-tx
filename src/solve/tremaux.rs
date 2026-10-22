use super::{remove_item, Ctx, MazeSolver};
use crate::{midpoint, within_one, Cell, Grid, MazeError, Path, Rng};
use std::collections::HashMap;

/// Trémaux's algorithm: mark each cell as you pass, prefer unmarked
/// passages, and never take a passage marked twice when there is a choice.
/// Finds one (not necessarily shortest) solution.
#[derive(Clone, Debug, Default)]
pub struct Tremaux {
    visited_cells: HashMap<Cell, u32>,
}

impl Tremaux {
    /// A new solver.
    pub fn new() -> Self {
        Tremaux::default()
    }

    fn visit(&mut self, cell: Cell) {
        *self.visited_cells.entry(cell).or_insert(0) += 1;
    }

    /// The visit count, capped at 2.
    fn visit_count(&self, cell: Cell) -> usize {
        self.visited_cells
            .get(&cell)
            .map_or(0, |&n| n.min(2) as usize)
    }

    fn what_next(&self, ns: &[Cell], solution: &[Cell], rng: &mut Rng) -> Result<Cell, MazeError> {
        if ns.len() <= 1 {
            return ns
                .first()
                .copied()
                .ok_or_else(|| MazeError::Index("list index out of range".into()));
        }

        // neighbors grouped by visit count
        let mut by_count: [Vec<Cell>; 3] = Default::default();
        for &n in ns {
            by_count[self.visit_count(n)].push(n);
        }

        let previous = (solution.len() > 2).then(|| solution[solution.len() - 3]);
        let bucket = if !by_count[0].is_empty() {
            return rng.py.choice(&by_count[0]);
        } else if !by_count[1].is_empty() {
            &mut by_count[1]
        } else {
            &mut by_count[2]
        };
        // try not to backtrack, if you can
        if let Some(prev) = previous {
            if bucket.len() > 1 && bucket.contains(&prev) {
                remove_item(bucket, prev);
            }
        }
        rng.py.choice(bucket)
    }
}

impl MazeSolver for Tremaux {
    fn solve(
        &mut self,
        grid: &Grid,
        start: Cell,
        end: Cell,
        rng: &mut Rng,
    ) -> Result<Vec<Path>, MazeError> {
        let ctx = Ctx::new(grid, start, end)?;
        self.visited_cells.clear();

        let mut solution = vec![ctx.start];
        self.visit(ctx.start);
        if ctx.on_edge(ctx.start) {
            let pushed = ctx.push_edge(ctx.start);
            solution.push(pushed);
            self.visit(pushed);
        }

        loop {
            let last = *solution.last().expect("non-empty");
            if within_one(last, ctx.end) {
                break;
            }
            let ns = ctx.unblocked(last, rng)?;
            let next = self.what_next(&ns, &solution, rng)?;
            solution.push(midpoint(last, next));
            solution.push(next);
            self.visit(next);
        }
        Ok(vec![solution])
    }
}
