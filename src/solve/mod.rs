//! Maze-solving algorithms (`mazelib.solve`).

use crate::{ensure, unblocked_neighbors, within_one, Cell, Grid, MazeError, Path, Rng};

mod backtracking_solver;
mod chain;
mod collision;
mod random_mouse;
mod shortest_path;
mod shortest_paths;
mod tremaux;

pub use backtracking_solver::BacktrackingSolver;
pub use chain::{Chain, Turn};
pub use collision::Collision;
pub use random_mouse::RandomMouse;
pub use shortest_path::ShortestPath;
pub use shortest_paths::ShortestPaths;
pub use tremaux::Tremaux;

/// A maze-solving algorithm (`MazeSolveAlgo`).
pub trait MazeSolver {
    /// Find solutions from `start` to `end`. The grid is not modified.
    /// Raw solutions may include backtracking; see [`prune_solutions`].
    fn solve(
        &mut self,
        grid: &Grid,
        start: Cell,
        end: Cell,
        rng: &mut Rng,
    ) -> Result<Vec<Path>, MazeError>;
}

/// The state every solver works on (`_solve_preprocessor`): a private copy
/// of the grid plus validated entrances.
pub(crate) struct Ctx {
    pub grid: Grid,
    pub start: Cell,
    pub end: Cell,
}

impl Ctx {
    pub fn new(grid: &Grid, start: Cell, end: Cell) -> Result<Self, MazeError> {
        let (hh, ww) = (grid.height() as i64, grid.width() as i64);
        for cell in [start, end] {
            ensure(cell.0 >= 0 && cell.0 < hh, "Entrance is outside the grid.")?;
            ensure(cell.1 >= 0 && cell.1 < ww, "Entrance is outside the grid.")?;
        }
        Ok(Ctx {
            grid: grid.clone(),
            start,
            end,
        })
    }

    pub fn unblocked(&self, cell: Cell, rng: &mut Rng) -> Result<Vec<Cell>, MazeError> {
        unblocked_neighbors(&self.grid, cell, rng)
    }

    pub fn on_edge(&self, cell: Cell) -> bool {
        self.grid.on_edge(cell)
    }

    /// `_push_edge`: the cell just inside an entrance on the border.
    pub fn push_edge(&self, (r, c): Cell) -> Cell {
        if r == 0 {
            (1, c)
        } else if r == self.grid.height() as i64 - 1 {
            (r - 1, c)
        } else if c == 0 {
            (r, 1)
        } else {
            (r, c - 1)
        }
    }

    /// The first cell to stand on: `start`, pushed in from the border.
    pub fn first_cell(&self) -> Cell {
        if self.on_edge(self.start) {
            self.push_edge(self.start)
        } else {
            self.start
        }
    }
}

/// Remove the first occurrence of `item` from `v`, if present.
pub(crate) fn remove_item(v: &mut Vec<Cell>, item: Cell) {
    if let Some(i) = v.iter().position(|&x| x == item) {
        v.remove(i);
    }
}

/// The random walk shared by [`BacktrackingSolver`] and [`Chain`]'s robots:
/// step to a random open neighbor, avoiding the cell just left when there
/// is a choice, until within one move of `goal`.
pub(crate) fn backtracking_walk(
    ctx: &Ctx,
    mut path: Vec<Cell>,
    goal: Cell,
    rng: &mut Rng,
) -> Result<Vec<Cell>, MazeError> {
    loop {
        let last = *path.last().expect("walks start non-empty");
        if within_one(last, goal) {
            return Ok(path);
        }
        let mut ns = ctx.unblocked(last, rng)?;
        if ns.len() > 1 && path.len() > 2 {
            remove_item(&mut ns, path[path.len() - 3]);
        }
        let next = rng.py.choice(&ns)?;
        path.push(crate::midpoint(last, next));
        path.push(next);
    }
}

/// `_prune_solution`: cut out loops (backtracking) and drop the entrances.
pub fn prune_solution(mut solution: Path, start: Cell, end: Cell) -> Path {
    let mut found = true;
    let mut attempt = 0;
    let max_attempt = solution.len();

    while found && solution.len() > 2 && attempt < max_attempt {
        found = false;
        attempt += 1;
        let mut cut = None;
        for i in 0..solution.len() - 1 {
            let first = solution[i];
            if let Some(j) = solution[i + 1..].iter().position(|&x| x == first) {
                cut = Some((i, j + i + 1));
                break;
            }
        }
        if let Some((first_i, last_i)) = cut {
            found = true;
            solution.drain(first_i..last_i);
        }
    }

    // solutions do not include the entrances
    if solution.len() > 1 {
        if solution[0] == start {
            solution.remove(0);
        }
        if solution.last() == Some(&end) {
            solution.pop();
        }
    }
    solution
}

/// `prune_solutions`: [`prune_solution`] on every solution.
pub fn prune_solutions(solutions: Vec<Path>, start: Cell, end: Cell) -> Vec<Path> {
    solutions
        .into_iter()
        .map(|s| prune_solution(s, start, end))
        .collect()
}
