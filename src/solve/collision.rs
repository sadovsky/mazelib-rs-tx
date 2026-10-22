use super::{Ctx, MazeSolver};
use crate::{midpoint, within_one, Cell, Grid, MazeError, Path, Rng};
use std::collections::{HashSet, VecDeque};

/// The Collision solver.
///
/// 1. Flood the maze from the start, one step at a time in every direction.
/// 2. Where two flood paths meet, wall off the meeting point.
/// 3. Drop the paths that hit dead ends.
/// 4. Repeat until the flood is stable.
///
/// In the original, any collision crashes the next flood step with a
/// `TypeError`, so it only works on mazes without loops. When that would
/// happen, this port instead returns one shortest path found by a
/// breadth-first search over the open squares.
#[derive(Clone, Debug, Default)]
pub struct Collision;

impl Collision {
    /// A new solver.
    pub fn new() -> Self {
        Collision
    }
}

/// One element of a robot's path.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Step {
    At(Cell),
    /// The robot reached the end (`END`).
    End,
    /// The robot is stuck (`DEAD_END`).
    DeadEnd,
}

/// The original would crash: two robots collided.
struct Collided;

type Robot = Vec<Step>;

/// Advance every running robot one cell; `None` when none could move.
fn one_time_step(
    ctx: &Ctx,
    paths: &[Robot],
    rng: &mut Rng,
) -> Result<Result<Option<Vec<Robot>>, Collided>, MazeError> {
    let mut temp: Vec<Robot> = Vec::new();
    let mut step_made = false;

    for path in paths {
        let cell = match path.last() {
            Some(Step::At(cell)) => *cell,
            Some(Step::End) => {
                temp.push(path.clone());
                continue;
            }
            Some(Step::DeadEnd) | None => continue,
        };

        let mut ns = ctx.unblocked(cell, rng)?;
        if path.len() > 2 {
            if let Step::At(back) = path[path.len() - 3] {
                super::remove_item(&mut ns, back);
            }
        }

        step_made = true;
        if ns.is_empty() {
            let mut p = path.clone();
            p.push(Step::DeadEnd);
            temp.push(p);
        } else {
            for n in ns {
                let mut p = path.clone();
                p.push(Step::At(midpoint(cell, n)));
                p.push(Step::At(n));
                if within_one(n, ctx.end) {
                    p.push(Step::End);
                }
                temp.push(p);
            }
        }
    }

    if !step_made {
        return Ok(Ok(None));
    }

    // two running robots on the same cell is a collision
    let mut seen = HashSet::new();
    for path in &temp {
        if let Some(Step::At(cell)) = path.last() {
            if !seen.insert(*cell) {
                return Ok(Err(Collided));
            }
        }
    }
    Ok(Ok(Some(temp)))
}

fn flood_maze(
    ctx: &Ctx,
    start: Cell,
    rng: &mut Rng,
) -> Result<Result<Vec<Robot>, Collided>, MazeError> {
    let mut paths = match one_time_step(ctx, &[vec![Step::At(start)]], rng)? {
        Ok(Some(p)) => p,
        Ok(None) => return Ok(Err(Collided)),
        Err(c) => return Ok(Err(c)),
    };
    loop {
        match one_time_step(ctx, &paths, rng)? {
            Ok(Some(next)) => paths = next,
            Ok(None) => return Ok(Ok(paths)),
            Err(c) => return Ok(Err(c)),
        }
    }
}

/// Drop dead-ended robots and the `END` markers, and trim the entrances.
fn fix_entrances(ctx: &Ctx, paths: Vec<Robot>) -> Vec<Path> {
    let start_inside = !ctx.on_edge(ctx.start);
    let end_inside = !ctx.on_edge(ctx.end);
    paths
        .into_iter()
        .filter(|p| p.last() != Some(&Step::DeadEnd))
        .map(|mut p| {
            p.pop();
            if start_inside && !p.is_empty() {
                p.remove(0);
            }
            if end_inside {
                p.pop();
            }
            p.into_iter()
                .filter_map(|s| match s {
                    Step::At(c) => Some(c),
                    _ => None,
                })
                .collect()
        })
        .collect()
}

/// Breadth-first search over open squares from `start` to `end`; the
/// returned path lists the squares strictly between them.
fn bfs_shortest(grid: &Grid, start: Cell, end: Cell) -> Vec<Path> {
    let (hh, ww) = (grid.height() as i64, grid.width() as i64);
    let mut came_from = std::collections::HashMap::new();
    let mut queue = VecDeque::from([start]);
    came_from.insert(start, start);
    while let Some(cell) = queue.pop_front() {
        if cell == end {
            let mut path = Vec::new();
            let mut at = came_from[&end];
            while at != start {
                path.push(at);
                at = came_from[&at];
            }
            path.reverse();
            return vec![path];
        }
        for (dr, dc) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let n = (cell.0 + dr, cell.1 + dc);
            let inside = (0..hh).contains(&n.0) && (0..ww).contains(&n.1);
            if inside && !came_from.contains_key(&n) && (n == end || grid.at(n) == 0) {
                came_from.insert(n, cell);
                queue.push_back(n);
            }
        }
    }
    Vec::new()
}

impl MazeSolver for Collision {
    fn solve(
        &mut self,
        grid: &Grid,
        start: Cell,
        end: Cell,
        rng: &mut Rng,
    ) -> Result<Vec<Path>, MazeError> {
        let ctx = Ctx::new(grid, start, end)?;
        let first = ctx.first_cell();

        let flooded = (|| -> Result<Result<Vec<Robot>, Collided>, MazeError> {
            let mut paths = match flood_maze(&ctx, first, rng)? {
                Ok(p) => p,
                Err(c) => return Ok(Err(c)),
            };
            let mut temp = match flood_maze(&ctx, first, rng)? {
                Ok(p) => p,
                Err(c) => return Ok(Err(c)),
            };
            // re-flood until the result is stable
            loop {
                let latest: HashSet<&Robot> = temp.iter().collect();
                if paths.iter().all(|p| latest.contains(p)) {
                    return Ok(Ok(paths));
                }
                paths = temp;
                temp = match flood_maze(&ctx, first, rng)? {
                    Ok(p) => p,
                    Err(c) => return Ok(Err(c)),
                };
            }
        })()?;

        Ok(match flooded {
            Ok(paths) => fix_entrances(&ctx, paths),
            Err(Collided) => bfs_shortest(&ctx.grid, ctx.start, ctx.end),
        })
    }
}
