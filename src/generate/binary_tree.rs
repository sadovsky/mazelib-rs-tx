use super::{Dims, MazeGenerator};
use crate::{Cell, Grid, MazeError, Rng};

/// Which two walls the Binary Tree algorithm may knock down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Skew {
    /// North-west.
    NW,
    /// North-east.
    NE,
    /// South-west.
    SW,
    /// South-east.
    SE,
}

impl Skew {
    /// In the original's dictionary order, used for the random default.
    const ALL: [Skew; 4] = [Skew::NW, Skew::NE, Skew::SW, Skew::SE];

    /// Parse `"NW"`, `"NE"`, `"SW"` or `"SE"` (exact match).
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "NW" => Some(Skew::NW),
            "NE" => Some(Skew::NE),
            "SW" => Some(Skew::SW),
            "SE" => Some(Skew::SE),
            _ => None,
        }
    }

    fn offsets(self) -> [Cell; 2] {
        match self {
            Skew::NW => [(1, 0), (0, -1)],
            Skew::NE => [(1, 0), (0, 1)],
            Skew::SW => [(-1, 0), (0, -1)],
            Skew::SE => [(-1, 0), (0, 1)],
        }
    }
}

/// The Binary Tree algorithm: for every cell, knock down one of two walls
/// in the skewed directions.
#[derive(Clone, Debug)]
pub struct BinaryTree {
    dims: Dims,
    skew: Skew,
}

impl BinaryTree {
    /// A generator for an `h` x `w` maze. With no `skew`, one is drawn at
    /// random now, as the original's constructor does.
    pub fn new(h: i64, w: i64, skew: Option<Skew>, rng: &mut Rng) -> Result<Self, MazeError> {
        let dims = Dims::new(h, w)?;
        let skew = match skew {
            Some(s) => s,
            None => rng.py.choice(&Skew::ALL)?,
        };
        Ok(BinaryTree { dims, skew })
    }

    /// The skew in use.
    pub fn skew(&self) -> Skew {
        self.skew
    }

    fn find_neighbor(&self, (row, col): Cell, rng: &mut Rng) -> Result<Cell, MazeError> {
        let d = self.dims;
        let neighbors: Vec<Cell> = self
            .skew
            .offsets()
            .iter()
            .map(|&(dr, dc)| (row + dr, col + dc))
            .filter(|&(r, c)| r > 0 && r < d.hh - 1 && c > 0 && c < d.ww - 1)
            .collect();
        if neighbors.is_empty() {
            Ok((row, col))
        } else {
            rng.py.choice(&neighbors)
        }
    }
}

impl MazeGenerator for BinaryTree {
    fn generate(&mut self, rng: &mut Rng) -> Result<Grid, MazeError> {
        let d = self.dims;
        let mut grid = d.grid(1);
        for row in (1..d.hh).step_by(2) {
            for col in (1..d.ww).step_by(2) {
                grid.set((row, col), 0);
                let n = self.find_neighbor((row, col), rng)?;
                grid.set(n, 0);
            }
        }
        Ok(grid)
    }
}
