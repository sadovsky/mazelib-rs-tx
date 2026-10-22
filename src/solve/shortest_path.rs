use super::{Ctx, MazeSolver};
use crate::pyset::PySet;
use crate::{ensure, midpoint, within_one, Cell, Grid, MazeError, Path, Rng};

/// A search path; `None` marks a finished (stopped) path.
pub(crate) type Search = Vec<Option<Cell>>;

/// The Shortest Path solver: breadth-first search that returns as soon as
/// one path reaches the end.
///
/// Like the original, it decides whether the *end* is on the border by
/// looking at the *start*, so it sometimes returns no solution on mazes
/// that have one.
#[derive(Clone, Debug, Default)]
pub struct ShortestPath;

impl ShortestPath {
    /// A new solver.
    pub fn new() -> Self {
        ShortestPath
    }
}

/// How the breadth-first solvers treat a path that reaches the end.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    /// `ShortestPath`: return the first path to arrive.
    First,
    /// `ShortestPaths`: stop each path that arrives and keep searching.
    All,
}

/// The search loop shared by [`ShortestPath`] and `ShortestPaths`.
pub(crate) fn bfs_solve(
    ctx: &Ctx,
    end_edge: bool,
    mode: Mode,
    rng: &mut Rng,
) -> Result<Vec<Path>, MazeError> {
    let start_edge = ctx.on_edge(ctx.start);
    let start = ctx.first_cell();

    let start_posis = ctx.unblocked(start, rng)?;
    ensure(!start_posis.is_empty(), "Input maze is invalid.")?;

    let mut solutions: Vec<Search> = start_posis
        .iter()
        .map(|&sp| {
            if start_edge {
                vec![Some(start), Some(midpoint(start, sp)), Some(sp)]
            } else {
                vec![Some(midpoint(start, sp)), Some(sp)]
            }
        })
        .collect();

    let mut num_unfinished = solutions.len();
    while num_unfinished > 0 {
        for s in 0..solutions.len() {
            let sol = &solutions[s];
            let last = *sol.last().expect("non-empty");
            let n = sol.len();
            if last.is_some() && sol[..n - 1].contains(&last) {
                // stop solutions that have looped
                solutions[s].push(None);
                continue;
            }
            let Some(cell) = last else {
                continue;
            };
            if within_one(cell, ctx.end) {
                // this solution reached the end
                match mode {
                    Mode::First => {
                        let mut sol = solutions[s].clone();
                        if !ctx.on_edge(ctx.end) {
                            sol.pop();
                        }
                        return clean_up(ctx, end_edge, vec![sol]);
                    }
                    Mode::All => {
                        solutions[s].push(None);
                        continue;
                    }
                }
            }
            if n > 1 {
                // gone past the end?
                if let Some(prev) = sol[n - 2] {
                    if midpoint(cell, prev) == ctx.end {
                        match mode {
                            Mode::First => {
                                let sol = solutions[s][..n - 1].to_vec();
                                return clean_up(ctx, end_edge, vec![sol]);
                            }
                            Mode::All => {
                                solutions[s].push(None);
                                continue;
                            }
                        }
                    }
                }
            }

            let ns = ctx.unblocked(cell, rng)?;
            let ns: Vec<Cell> = match mode {
                Mode::First => ns
                    .into_iter()
                    .filter(|x| !sol.contains(&Some(*x)))
                    .collect(),
                Mode::All => {
                    let tail = &sol[n.saturating_sub(2)..];
                    ns.into_iter()
                        .filter(|x| !tail.contains(&Some(*x)))
                        .collect()
                }
            };

            match ns.len() {
                0 => solutions[s].push(None),
                _ => {
                    for &next in &ns[1..] {
                        let mut branch = solutions[s].clone();
                        branch.push(Some(midpoint(next, cell)));
                        branch.push(Some(next));
                        solutions.push(branch);
                    }
                    solutions[s].push(Some(midpoint(ns[0], cell)));
                    solutions[s].push(Some(ns[0]));
                }
            }
        }
        num_unfinished = solutions.iter().filter(|s| s.last() != Some(&None)).count();
    }

    clean_up(ctx, end_edge, solutions)
}

/// `_clean_up`: drop incomplete solutions, trim the end, remove duplicates
/// (through a Python set, which fixes their order) and sort by length.
fn clean_up(ctx: &Ctx, end_edge: bool, solutions: Vec<Search>) -> Result<Vec<Path>, MazeError> {
    let mut new_solutions: Vec<Path> = Vec::new();
    for sol in solutions {
        let n = sol.len();
        let mut new_sol: Option<Search> = None;
        let second_last = if n > 2 { sol[n - 2] } else { None };
        if let Some(sl) = second_last {
            if end_edge {
                let last = ctx.push_edge(ctx.end);
                if within_one(sl, ctx.end) {
                    new_sol = Some(sol[..n - 1].to_vec());
                } else if within_one(sl, last) {
                    let mut v = sol[..n - 1].to_vec();
                    v.push(Some(last));
                    new_sol = Some(v);
                }
            } else if within_one(sl, ctx.end) {
                new_sol = Some(sol[..n - 1].to_vec());
            }
        }

        if let Some(mut ns) = new_sol.filter(|v| !v.is_empty()) {
            if ns.last() == Some(&Some(ctx.end)) {
                ns.pop();
            }
            let cells: Option<Path> = ns.into_iter().collect();
            let cells =
                cells.ok_or_else(|| MazeError::Type("unexpected None in solution".into()))?;
            new_solutions.push(cells);
        }
    }

    let mut unique = PySet::from_items(new_solutions).to_vec();
    unique.sort_by_key(|s| s.len());
    Ok(unique)
}

impl MazeSolver for ShortestPath {
    fn solve(
        &mut self,
        grid: &Grid,
        start: Cell,
        end: Cell,
        rng: &mut Rng,
    ) -> Result<Vec<Path>, MazeError> {
        let ctx = Ctx::new(grid, start, end)?;
        // (sic) the original tests the start here, not the end
        let end_edge = ctx.on_edge(ctx.start);
        bfs_solve(&ctx, end_edge, Mode::First, rng)
    }
}
