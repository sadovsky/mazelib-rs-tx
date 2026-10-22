//! Maze-transmuting algorithms (`mazelib.transmute`): they edit an existing
//! grid in place.

use crate::{Cell, Grid, MazeError, Rng};

mod cul_de_sac_filler;
mod dead_end_filler;
mod perturbation;

pub use cul_de_sac_filler::CuldeSacFiller;
pub use dead_end_filler::DeadEndFiller;
pub use perturbation::Perturbation;

/// A maze-transmuting algorithm (`MazeTransmuteAlgo`).
pub trait MazeTransmuter {
    /// Modify `grid` in place. `start` and `end` are the maze entrances, if set.
    fn transmute(
        &mut self,
        grid: &mut Grid,
        start: Option<Cell>,
        end: Option<Cell>,
        rng: &mut Rng,
    ) -> Result<(), MazeError>;
}
