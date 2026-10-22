use super::{Dims, MazeGenerator};
use crate::{Grid, MazeError, Rng};

const VERTICAL: i64 = 0;

/// The Recursive Division algorithm.
///
/// 1. Start with an empty grid.
/// 2. Build a wall that bisects the region, with a single passage through it.
/// 3. Repeat on the regions on either side of the wall.
#[derive(Clone, Debug)]
pub struct Division {
    dims: Dims,
}

impl Division {
    /// A generator for an `h` x `w` maze.
    pub fn new(h: i64, w: i64) -> Result<Self, MazeError> {
        Ok(Division {
            dims: Dims::new(h, w)?,
        })
    }
}

impl MazeGenerator for Division {
    fn generate(&mut self, rng: &mut Rng) -> Result<Grid, MazeError> {
        let d = self.dims;
        let mut grid = d.grid(0);
        for c in 0..d.ww {
            grid.set((0, c), 1);
            grid.set((d.hh - 1, c), 1);
        }
        for r in 0..d.hh {
            grid.set((r, 0), 1);
            grid.set((r, d.ww - 1), 1);
        }

        // regions are ((min_y, min_x), (max_y, max_x))
        let mut region_stack = vec![((1, 1), (d.hh - 2, d.ww - 2))];
        while let Some(((min_y, min_x), (max_y, max_x))) = region_stack.pop() {
            let height = max_y - min_y + 1;
            let width = max_x - min_x + 1;
            if height <= 1 || width <= 1 {
                continue;
            }

            let cut_direction = if width < height {
                1 // horizontal
            } else if width > height {
                VERTICAL
            } else {
                if width == 2 {
                    continue;
                }
                rng.py.randrange_to(2)?
            };

            // the cut can't be on the edge of the region
            let cut_length = if cut_direction == VERTICAL {
                width
            } else {
                height
            };
            if cut_length < 3 {
                continue;
            }
            let cut_posi = rng.py.randrange(1, cut_length, 2)?;
            let door_span = if cut_direction == VERTICAL {
                height
            } else {
                width
            };
            let door_posi = rng.py.randrange(0, door_span, 2)?;

            if cut_direction == VERTICAL {
                for row in min_y..=max_y {
                    grid.set((row, min_x + cut_posi), 1);
                }
                grid.set((min_y + door_posi, min_x + cut_posi), 0);
                region_stack.push(((min_y, min_x), (max_y, min_x + cut_posi - 1)));
                region_stack.push(((min_y, min_x + cut_posi + 1), (max_y, max_x)));
            } else {
                for col in min_x..=max_x {
                    grid.set((min_y + cut_posi, col), 1);
                }
                grid.set((min_y + cut_posi, min_x + door_posi), 0);
                region_stack.push(((min_y, min_x), (min_y + cut_posi - 1, max_x)));
                region_stack.push(((min_y + cut_posi + 1, min_x), (max_y, max_x)));
            }
        }
        Ok(grid)
    }
}
