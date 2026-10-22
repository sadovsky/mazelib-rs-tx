use super::MazeTransmuter;
use crate::{unblocked_neighbors, Cell, Grid, MazeError, Rng};

/// The Cul-de-Sac Filler (really a loop breaker).
///
/// 1. Find cells whose passages go in exactly two directions.
/// 2. From each, travel both ways to the first intersection.
/// 3. If both ways reach the same intersection, it is a loop: wall off the
///    starting cell.
#[derive(Clone, Debug, Default)]
pub struct CuldeSacFiller;

impl CuldeSacFiller {
    /// A new transmuter.
    pub fn new() -> Self {
        CuldeSacFiller
    }
}

/// Follow a corridor from `first` through `current` to the next
/// intersection (or dead end).
fn find_next_intersection(
    grid: &Grid,
    first: Cell,
    mut current: Cell,
    rng: &mut Rng,
) -> Result<Cell, MazeError> {
    let mut previous = first;
    let mut ns = unblocked_neighbors(grid, current, rng)?;
    while ns.len() == 2 {
        // travel away from where you came from
        let next = if ns[0] == previous { ns[1] } else { ns[0] };
        previous = current;
        current = next;
        // looped all the way round without any intersection
        if current == first {
            return Ok(previous);
        }
        ns = unblocked_neighbors(grid, current, rng)?;
    }
    Ok(current)
}

impl MazeTransmuter for CuldeSacFiller {
    fn transmute(
        &mut self,
        grid: &mut Grid,
        start: Option<Cell>,
        end: Option<Cell>,
        rng: &mut Rng,
    ) -> Result<(), MazeError> {
        for r in (1..grid.height() as i64).step_by(2) {
            for c in (1..grid.width() as i64).step_by(2) {
                if start == Some((r, c)) || end == Some((r, c)) {
                    // don't block off an exit
                    continue;
                }
                if grid.at((r, c)) == 1 {
                    continue;
                }
                let ns = unblocked_neighbors(grid, (r, c), rng)?;
                if ns.len() != 2 {
                    continue;
                }
                let end1 = find_next_intersection(grid, (r, c), ns[0], rng)?;
                let end2 = find_next_intersection(grid, (r, c), ns[1], rng)?;
                if end1 == end2 {
                    grid.set((r, c), 1);
                }
            }
        }
        Ok(())
    }
}
