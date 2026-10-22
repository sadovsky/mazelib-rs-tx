use super::MazeTransmuter;
use crate::{unblocked_neighbors, within_one, Cell, Grid, MazeError, Rng};

/// The Dead-End Filler: wall off dead ends, `iterations` times.
///
/// As in the original, each iteration fills one dead end (filling a dead end
/// walls off its exits, so the walk onward stops at once).
#[derive(Clone, Debug)]
pub struct DeadEndFiller {
    iterations: i64,
}

impl DeadEndFiller {
    /// A new transmuter; `iterations <= 0` means 100 (default: 1).
    pub fn new(iterations: i64) -> Self {
        DeadEndFiller {
            iterations: if iterations > 0 { iterations } else { 100 },
        }
    }
}

impl Default for DeadEndFiller {
    fn default() -> Self {
        DeadEndFiller::new(1)
    }
}

struct Filler<'a> {
    grid: &'a mut Grid,
    start: Cell,
    end: Cell,
}

impl Filler<'_> {
    /// Fill dead ends; returns whether any was found.
    fn fill_dead_ends(&mut self, rng: &mut Rng) -> Result<bool, MazeError> {
        let mut dead_end = self.find_dead_end(rng)?;
        let mut found = false;
        while dead_end != (-1, -1) {
            found = true;
            self.fill_dead_end(dead_end)?;

            let ns = unblocked_neighbors(self.grid, dead_end, rng)?;
            if ns.is_empty() {
                break;
            }
            // keep going while the next cell is also a dead end
            if ns.len() == 1 && self.is_dead_end(ns[0], rng)? {
                dead_end = ns[0];
                continue;
            }
            dead_end = self.find_dead_end(rng)?;
        }
        Ok(found)
    }

    fn fill_dead_end(&mut self, (r, c): Cell) -> Result<(), MazeError> {
        for cell in [(r, c), (r - 1, c), (r + 1, c), (r, c - 1), (r, c + 1)] {
            self.grid.py_set(cell, 1)?;
        }
        Ok(())
    }

    fn find_dead_end(&self, rng: &mut Rng) -> Result<Cell, MazeError> {
        for r in (1..self.grid.height() as i64).step_by(2) {
            for c in (1..self.grid.width() as i64).step_by(2) {
                if within_one((r, c), self.start) || within_one((r, c), self.end) {
                    continue;
                }
                if self.is_dead_end((r, c), rng)? {
                    return Ok((r, c));
                }
            }
        }
        Ok((-1, -1))
    }

    /// An open cell with zero or one open neighbors.
    fn is_dead_end(&self, cell: Cell, rng: &mut Rng) -> Result<bool, MazeError> {
        let ns = unblocked_neighbors(self.grid, cell, rng)?;
        Ok(self.grid.py_get(cell)? != 1 && ns.len() < 2)
    }
}

impl MazeTransmuter for DeadEndFiller {
    fn transmute(
        &mut self,
        grid: &mut Grid,
        start: Option<Cell>,
        end: Option<Cell>,
        rng: &mut Rng,
    ) -> Result<(), MazeError> {
        let missing = || MazeError::Type("cannot unpack non-iterable NoneType object".into());
        let start = start.ok_or_else(missing)?;
        let start_save = grid.py_get(start)?;
        grid.py_set(start, 0)?;
        let end = end.ok_or_else(missing)?;
        let end_save = grid.py_get(end)?;
        grid.py_set(end, 0)?;

        let mut filler = Filler { grid, start, end };
        let mut found = true;
        let mut i = 0;
        while found && i < self.iterations {
            i += 1;
            found = filler.fill_dead_ends(rng)?;
        }

        // restore the entrances
        grid.py_set(start, start_save)?;
        grid.py_set(end, end_save)?;
        Ok(())
    }
}
