use super::MazeTransmuter;
use crate::generate::{find_all_passages, fix_disjoint_passages, ShuffleWith};
use crate::{neighbors_in_grid, Cell, Grid, MazeError, Rng};

/// Perturbation: add a few random walls, then reconnect the maze by opening
/// random walls; repeat.
#[derive(Clone, Debug)]
pub struct Perturbation {
    repeat: i64,
    new_walls: i64,
}

impl Perturbation {
    /// A new transmuter (defaults: `repeat = 1`, `new_walls = 1`).
    pub fn new(repeat: i64, new_walls: i64) -> Self {
        Perturbation { repeat, new_walls }
    }
}

impl Default for Perturbation {
    fn default() -> Self {
        Perturbation::new(1, 1)
    }
}

/// Close one random open wall square (gives up after many tries).
fn add_a_random_wall(grid: &mut Grid, rng: &mut Rng) -> Result<(), MazeError> {
    let (hh, ww) = (grid.height() as i64, grid.width() as i64);
    let limit = 2 * hh * ww;
    let mut tries = 0;
    loop {
        let row = rng.py.randrange(1, hh - 1, 1)?;
        let col = if row % 2 == 0 {
            rng.py.randrange(1, ww - 1, 2)?
        } else {
            rng.py.randrange(2, ww - 1, 2)?
        };
        if grid.py_get((row, col))? == 0 {
            grid.py_set((row, col), 1)?;
            return Ok(());
        }
        tries += 1;
        if tries > limit {
            return Ok(());
        }
    }
}

impl MazeTransmuter for Perturbation {
    fn transmute(
        &mut self,
        grid: &mut Grid,
        _start: Option<Cell>,
        _end: Option<Cell>,
        rng: &mut Rng,
    ) -> Result<(), MazeError> {
        for _ in 0..self.repeat {
            for _ in 0..self.new_walls {
                add_a_random_wall(grid, rng)?;
            }
            let passages = find_all_passages(grid, ShuffleWith::Numpy, rng)?;
            fix_disjoint_passages(
                grid,
                passages,
                |cell, grid, rng| neighbors_in_grid(grid, cell, false, rng),
                rng,
            )?;
        }
        Ok(())
    }
}
