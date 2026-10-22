//! A Rust port of [mazelib](https://github.com/john-science/mazelib), the
//! Python maze generation and solving library.
//!
//! The port is a transpilation: for a given seed it generates the same
//! mazes, picks the same entrances and finds the same solutions as the
//! original, because it reproduces Python's `random` and NumPy's legacy
//! `RandomState` bit for bit (see [`rng`]) and makes the same random calls
//! in the same order.
//!
//! ```
//! use mazelib::{generate::Prims, Maze};
//!
//! let mut maze = Maze::with_seed(1).unwrap();
//! maze.generator = Some(Box::new(Prims::new(5, 5).unwrap()));
//! maze.generate().unwrap();
//! maze.generate_entrances(true, true).unwrap();
//! assert_eq!(maze.grid.as_ref().unwrap().height(), 11);
//! ```

#![forbid(unsafe_code)]

pub mod generate;
mod grid;
mod maze;
pub mod pyset;
pub mod rng;
pub mod solve;
pub mod transmute;

pub use grid::Grid;
pub use maze::Maze;
pub use rng::Rng;

use std::fmt;

/// A grid position `(row, column)`, 0-based, row 0 at the top.
pub type Cell = (i64, i64);

/// A path through the maze, listing the cells strictly between start and end.
pub type Path = Vec<Cell>;

/// Errors raised by mazelib. Each variant corresponds to the Python
/// exception class the original raises in the same situation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MazeError {
    /// A precondition failed (Python `AssertionError`).
    Assertion(String),
    /// An argument had an invalid value (Python `ValueError`).
    Value(String),
    /// An index was out of range or a sequence was empty (Python `IndexError`).
    Index(String),
    /// A required value was missing (Python `TypeError`).
    Type(String),
}

impl MazeError {
    /// The name of the Python exception class this error corresponds to.
    pub fn python_kind(&self) -> &'static str {
        match self {
            MazeError::Assertion(_) => "AssertionError",
            MazeError::Value(_) => "ValueError",
            MazeError::Index(_) => "IndexError",
            MazeError::Type(_) => "TypeError",
        }
    }

    /// The human-readable message.
    pub fn message(&self) -> &str {
        match self {
            MazeError::Assertion(m)
            | MazeError::Value(m)
            | MazeError::Index(m)
            | MazeError::Type(m) => m,
        }
    }
}

impl fmt::Display for MazeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.python_kind(), self.message())
    }
}

impl std::error::Error for MazeError {}

/// Fail with [`MazeError::Assertion`] unless `cond` holds.
pub(crate) fn ensure(cond: bool, msg: &str) -> Result<(), MazeError> {
    if cond {
        Ok(())
    } else {
        Err(MazeError::Assertion(msg.to_string()))
    }
}

/// The wall cell half-way between two cells (`_midpoint`).
pub(crate) fn midpoint(a: Cell, b: Cell) -> Cell {
    ((a.0 + b.0).div_euclid(2), (a.1 + b.1).div_euclid(2))
}

/// True if `cell` is within one full or half move of `desire` (`_within_one`).
pub(crate) fn within_one(cell: Cell, desire: Cell) -> bool {
    if cell.0 == desire.0 {
        (cell.1 - desire.1).abs() < 2
    } else if cell.1 == desire.1 {
        (cell.0 - desire.0).abs() < 2
    } else {
        false
    }
}

/// `_find_unblocked_neighbors`: cells two squares away that are open and
/// joined to `cell` by an open wall, shuffled with NumPy's generator.
pub(crate) fn unblocked_neighbors(
    grid: &Grid,
    cell: Cell,
    rng: &mut Rng,
) -> Result<Vec<Cell>, MazeError> {
    let mut ns = unblocked_cells(grid, cell)?;
    rng.np.shuffle(&mut ns);
    Ok(ns)
}

/// The unshuffled neighbor list behind [`unblocked_neighbors`].
pub(crate) fn unblocked_cells(grid: &Grid, (r, c): Cell) -> Result<Vec<Cell>, MazeError> {
    let (hh, ww) = (grid.height() as i64, grid.width() as i64);
    let open = |cell: Cell| -> Result<bool, MazeError> { Ok(grid.py_get(cell)? == 0) };
    let mut ns = Vec::with_capacity(4);
    if r > 1 && open((r - 1, c))? && open((r - 2, c))? {
        ns.push((r - 2, c));
    }
    if r < hh - 2 && open((r + 1, c))? && open((r + 2, c))? {
        ns.push((r + 2, c));
    }
    if c > 1 && open((r, c - 1))? && open((r, c - 2))? {
        ns.push((r, c - 2));
    }
    if c < ww - 2 && open((r, c + 1))? && open((r, c + 2))? {
        ns.push((r, c + 2));
    }
    Ok(ns)
}

/// `_find_neighbors` bounded by the grid's own shape: cells two squares
/// away whose wall state is `is_wall`, shuffled with NumPy's generator.
pub(crate) fn neighbors_in_grid(
    grid: &Grid,
    (r, c): Cell,
    is_wall: bool,
    rng: &mut Rng,
) -> Result<Vec<Cell>, MazeError> {
    let (hh, ww) = (grid.height() as i64, grid.width() as i64);
    let want = u8::from(is_wall);
    let mut ns = Vec::with_capacity(4);
    if r > 1 && grid.py_get((r - 2, c))? == want {
        ns.push((r - 2, c));
    }
    if r < hh - 2 && grid.py_get((r + 2, c))? == want {
        ns.push((r + 2, c));
    }
    if c > 1 && grid.py_get((r, c - 2))? == want {
        ns.push((r, c - 2));
    }
    if c < ww - 2 && grid.py_get((r, c + 2))? == want {
        ns.push((r, c + 2));
    }
    rng.np.shuffle(&mut ns);
    Ok(ns)
}
