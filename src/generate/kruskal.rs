use super::{Dims, MazeGenerator};
use crate::{Cell, Grid, MazeError, Rng};

/// Kruskal's algorithm: knock down walls in random order whenever they
/// separate two different trees of the forest.
#[derive(Clone, Debug)]
pub struct Kruskal {
    dims: Dims,
}

impl Kruskal {
    /// A generator for an `h` x `w` maze.
    pub fn new(h: i64, w: i64) -> Result<Self, MazeError> {
        Ok(Kruskal {
            dims: Dims::new(h, w)?,
        })
    }
}

impl MazeGenerator for Kruskal {
    fn generate(&mut self, rng: &mut Rng) -> Result<Grid, MazeError> {
        let d = self.dims;
        let mut grid = d.grid(1);

        let mut forest: Vec<Vec<Cell>> = Vec::new();
        for row in (1..d.hh - 1).step_by(2) {
            for col in (1..d.ww - 1).step_by(2) {
                forest.push(vec![(row, col)]);
                grid.set((row, col), 0);
            }
        }

        let mut edges: Vec<Cell> = Vec::new();
        for row in (2..d.hh - 1).step_by(2) {
            for col in (1..d.ww - 1).step_by(2) {
                edges.push((row, col));
            }
        }
        for row in (1..d.hh - 1).step_by(2) {
            for col in (2..d.ww - 1).step_by(2) {
                edges.push((row, col));
            }
        }
        rng.np.shuffle(&mut edges);

        // like the original, a cell's tree index is the sum of the indices
        // of the trees that contain it (there is exactly one)
        let tree_of = |forest: &[Vec<Cell>], cell: Cell| -> usize {
            forest
                .iter()
                .enumerate()
                .filter(|(_, t)| t.contains(&cell))
                .map(|(i, _)| i)
                .sum()
        };

        let mut edges = edges.into_iter();
        while forest.len() > 1 {
            let (ce_row, ce_col) = edges
                .next()
                .ok_or_else(|| MazeError::Index("list index out of range".into()))?;
            let (a, b) = if ce_row % 2 == 0 {
                ((ce_row - 1, ce_col), (ce_row + 1, ce_col))
            } else {
                ((ce_row, ce_col - 1), (ce_row, ce_col + 1))
            };
            let tree1 = tree_of(&forest, a);
            let tree2 = tree_of(&forest, b);

            if tree1 != tree2 {
                let mut new_tree = forest[tree1].clone();
                new_tree.extend_from_slice(&forest[tree2]);
                let temp1 = forest[tree1].clone();
                let temp2 = forest[tree2].clone();
                forest.retain(|x| *x != temp1);
                forest.retain(|x| *x != temp2);
                forest.push(new_tree);
                grid.set((ce_row, ce_col), 0);
            }
        }
        Ok(grid)
    }
}
