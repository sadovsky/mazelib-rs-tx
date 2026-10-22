use crate::{Cell, MazeError};
use std::fmt;

/// A rectangular maze grid stored row-major: `1` is a wall, `0` is open.
///
/// A maze of `h` x `w` cells has `2h+1` rows and `2w+1` columns; cells sit
/// at odd coordinates and the walls between them at even ones.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Grid {
    height: usize,
    width: usize,
    cells: Vec<u8>,
}

impl Grid {
    /// A grid of the given shape with every square set to `value`.
    pub fn filled(height: usize, width: usize, value: u8) -> Self {
        Grid {
            height,
            width,
            cells: vec![value; height * width],
        }
    }

    /// Build a grid from rows of `#` (wall) and anything else (open).
    pub fn from_rows<S: AsRef<str>>(rows: &[S]) -> Result<Self, MazeError> {
        let height = rows.len();
        let width = rows.first().map_or(0, |r| r.as_ref().chars().count());
        let mut cells = Vec::with_capacity(height * width);
        for row in rows {
            let before = cells.len();
            cells.extend(row.as_ref().chars().map(|ch| u8::from(ch == '#')));
            if cells.len() - before != width {
                return Err(MazeError::Value("grid rows have different lengths".into()));
            }
        }
        Ok(Grid {
            height,
            width,
            cells,
        })
    }

    /// The grid as rows of `#` (wall) and `.` (open).
    pub fn to_rows(&self) -> Vec<String> {
        (0..self.height)
            .map(|r| {
                self.row(r)
                    .iter()
                    .map(|&v| if v != 0 { '#' } else { '.' })
                    .collect()
            })
            .collect()
    }

    /// Number of rows.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Number of columns.
    pub fn width(&self) -> usize {
        self.width
    }

    /// `(height, width)`, like NumPy's `shape`.
    pub fn shape(&self) -> (usize, usize) {
        (self.height, self.width)
    }

    /// Row `r` as a slice.
    pub fn row(&self, r: usize) -> &[u8] {
        &self.cells[r * self.width..(r + 1) * self.width]
    }

    /// The raw row-major cells.
    pub fn as_slice(&self) -> &[u8] {
        &self.cells
    }

    /// The value at a cell known to be inside the grid.
    ///
    /// # Panics
    /// If the cell is outside the grid.
    pub fn at(&self, (r, c): Cell) -> u8 {
        self.cells[self.offset(r, c)]
    }

    /// Set a cell known to be inside the grid.
    ///
    /// # Panics
    /// If the cell is outside the grid.
    pub fn set(&mut self, (r, c): Cell, value: u8) {
        let i = self.offset(r, c);
        self.cells[i] = value;
    }

    fn offset(&self, r: i64, c: i64) -> usize {
        assert!(
            r >= 0 && c >= 0 && (r as usize) < self.height && (c as usize) < self.width,
            "cell ({r}, {c}) outside a {}x{} grid",
            self.height,
            self.width
        );
        r as usize * self.width + c as usize
    }

    /// Resolve a NumPy-style index (negative counts from the end).
    fn py_offset(&self, r: i64, c: i64) -> Result<usize, MazeError> {
        fn axis(i: i64, n: usize) -> Option<usize> {
            let n = n as i64;
            let j = if i < 0 { i + n } else { i };
            (0..n).contains(&j).then_some(j as usize)
        }
        match (axis(r, self.height), axis(c, self.width)) {
            (Some(r), Some(c)) => Ok(r * self.width + c),
            _ => Err(MazeError::Index(format!(
                "index ({r}, {c}) is out of bounds for a grid of shape ({}, {})",
                self.height, self.width
            ))),
        }
    }

    /// `grid[r, c]` with NumPy indexing semantics.
    pub fn py_get(&self, (r, c): Cell) -> Result<u8, MazeError> {
        Ok(self.cells[self.py_offset(r, c)?])
    }

    /// `grid[r, c] = value` with NumPy indexing semantics.
    pub fn py_set(&mut self, (r, c): Cell, value: u8) -> Result<(), MazeError> {
        let i = self.py_offset(r, c)?;
        self.cells[i] = value;
        Ok(())
    }

    /// True if `cell` lies on the outer border of the grid.
    pub fn on_edge(&self, (r, c): Cell) -> bool {
        r == 0 || r == self.height as i64 - 1 || c == 0 || c == self.width as i64 - 1
    }
}

impl fmt::Debug for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Grid {}x{}", self.height, self.width)?;
        for row in self.to_rows() {
            writeln!(f, "{row}")?;
        }
        Ok(())
    }
}
