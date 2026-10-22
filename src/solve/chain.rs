use super::{backtracking_walk, Ctx, MazeSolver};
use crate::{midpoint, within_one, Cell, Grid, MazeError, Path, Rng};

/// Which way [`Chain`]'s robots prefer to turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Turn {
    /// Turn right (the default).
    #[default]
    Right,
    /// Turn left.
    Left,
}

impl Turn {
    /// `"left"` (exact match) or else [`Turn::Right`].
    pub fn parse(s: &str) -> Self {
        if s == "left" {
            Turn::Left
        } else {
            Turn::Right
        }
    }
}

/// The Chain solver.
///
/// 1. Draw a straight-ish guiding line from start to end, ignoring walls.
/// 2. Follow the line; when a wall blocks it, send backtracking robots out
///    from the blocked point until one reaches the next point of the line.
/// 3. Repeat until reaching the end.
#[derive(Clone, Debug, Default)]
pub struct Chain {
    turn: Turn,
}

impl Chain {
    /// A new solver. The original records the turn direction but its robots
    /// do not use it.
    pub fn new(turn: Turn) -> Self {
        Chain { turn }
    }

    /// The configured turn direction.
    pub fn turn(&self) -> Turn {
        self.turn
    }
}

/// A (mostly) straight line from the start to the end.
fn draw_guiding_line(ctx: &Ctx) -> Vec<Cell> {
    let (r2, c2) = ctx.end;
    let mut current = ctx.first_cell();
    let mut path = vec![current];
    while !within_one(current, ctx.end) {
        let (r1, c1) = current;
        let mut rdiff = 0;
        let mut cdiff = 0;
        if (r1 - r2).abs() > 1 {
            rdiff = if r1 < r2 { 2 } else { -2 };
        }
        if (c1 - c2).abs() > 1 {
            cdiff = if c1 < c2 { 2 } else { -2 };
        }
        current = (r1 + rdiff, c1 + cdiff);
        path.push(current);
    }
    path
}

/// Step straight to the next point of the guiding line if the way is open.
fn try_direct_move(
    ctx: &Ctx,
    solution: &mut Vec<Cell>,
    line: &[Cell],
    i: usize,
) -> Result<bool, MazeError> {
    let (r, c) = line[i];
    let next = line[i + 1];
    let rdiff = next.0 - r;
    let cdiff = next.1 - c;
    let (rh, ch) = (rdiff.div_euclid(2), cdiff.div_euclid(2));
    let g = &ctx.grid;

    if rdiff == 0 || cdiff == 0 {
        if g.py_get((r + rh, c + ch))? == 0 {
            solution.push((r + rh, c + ch));
            solution.push((r + rdiff, c + cdiff));
            return Ok(true);
        }
        return Ok(false);
    }
    if g.py_get((r + rh, c))? == 0 && g.py_get((r + rdiff, c + ch))? == 0 {
        solution.extend([
            (r + rh, c),
            (r + rdiff, c),
            (r + rdiff, c + ch),
            (r + rdiff, c + cdiff),
        ]);
        Ok(true)
    } else if g.py_get((r, c + ch))? == 0 && g.py_get((r + rh, c + cdiff))? == 0 {
        solution.extend([
            (r, c + ch),
            (r, c + cdiff),
            (r + rh, c + cdiff),
            (r + rdiff, c + cdiff),
        ]);
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Send a robot down every open passage from `line[i]`; append the shortest
/// robot path to the solution and return where it rejoined the line.
fn send_out_robots(
    ctx: &Ctx,
    solution: &mut Vec<Cell>,
    line: &[Cell],
    i: usize,
    rng: &mut Rng,
) -> Result<usize, MazeError> {
    let ns = ctx.unblocked(line[i], rng)?;
    let robot_paths: Vec<Vec<Cell>> = ns
        .iter()
        .map(|&n| vec![line[i], midpoint(line[i], n), n])
        .collect();
    let mut walked = Vec::with_capacity(robot_paths.len());
    for path in robot_paths {
        walked.push(backtracking_walk(ctx, path, line[i + 1], rng)?);
    }

    // the first of the shortest
    let shortest = walked
        .iter()
        .min_by_key(|p| p.len())
        .ok_or_else(|| MazeError::Value("min() iterable argument is empty".into()))?;
    solution.extend_from_slice(&shortest[1..]);

    let last = *solution.last().expect("non-empty");
    line.iter()
        .position(|&x| x == last)
        .ok_or_else(|| MazeError::Value(format!("{last:?} is not in list")))
}

impl MazeSolver for Chain {
    fn solve(
        &mut self,
        grid: &Grid,
        start: Cell,
        end: Cell,
        rng: &mut Rng,
    ) -> Result<Vec<Path>, MazeError> {
        let ctx = Ctx::new(grid, start, end)?;
        let line = draw_guiding_line(&ctx);
        let mut current = 0;
        let mut solution = vec![line[0]];
        while current + 1 < line.len() {
            if try_direct_move(&ctx, &mut solution, &line, current)? {
                current += 1;
            } else {
                current = send_out_robots(&ctx, &mut solution, &line, current, rng)?;
            }
        }
        Ok(vec![solution])
    }
}
