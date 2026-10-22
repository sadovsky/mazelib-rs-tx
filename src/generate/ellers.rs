use super::{Dims, MazeGenerator};
use crate::{Grid, MazeError, Rng};

/// Eller's algorithm, row by row with set merging.
///
/// 1. Put the cells of the first row each in their own set.
/// 2. Join adjacent cells that are not already in the same set.
/// 3. For each set in the row, cut at least one passage down.
/// 4. Put any unconnected cells in the next row into their own set.
/// 5. Repeat until the last row, where all adjacent disjoint cells join.
///
/// The original stores set ids in an `int8` array and overflows (raising
/// `OverflowError`) on mazes that need more than 127 sets; this port uses
/// `i64` and returns a valid maze instead.
#[derive(Clone, Debug)]
pub struct Ellers {
    dims: Dims,
    xskew: f64,
    yskew: f64,
}

/// Row-major set ids, `-1` for "no set" (a wall).
struct Sets {
    width: usize,
    ids: Vec<i64>,
}

impl Sets {
    fn get(&self, r: i64, c: i64) -> i64 {
        self.ids[r as usize * self.width + c as usize]
    }
    fn set(&mut self, r: i64, c: i64, v: i64) {
        self.ids[r as usize * self.width + c as usize] = v;
    }
}

impl Ellers {
    /// A generator for an `h` x `w` maze. Skews are clamped to `[0, 1]`
    /// (defaults: `0.5`, `0.5`).
    pub fn new(h: i64, w: i64, xskew: f64, yskew: f64) -> Result<Self, MazeError> {
        Ok(Ellers {
            dims: Dims::new(h, w)?,
            xskew: xskew.clamp(0.0, 1.0),
            yskew: yskew.clamp(0.0, 1.0),
        })
    }

    fn init_row(&self, sets: &mut Sets, row: i64, mut max_set_number: i64) -> i64 {
        for c in (1..self.dims.ww).step_by(2) {
            if sets.get(row, c) < 0 {
                sets.set(row, c, max_set_number);
                max_set_number += 1;
            }
        }
        max_set_number
    }

    fn merge_one_row(&self, sets: &mut Sets, r: i64, rng: &mut Rng) {
        for c in (1..self.dims.ww - 2).step_by(2) {
            if rng.py.random() < self.xskew && sets.get(r, c) != sets.get(r, c + 2) {
                let to = sets.get(r, c);
                sets.set(r, c + 1, to);
                self.merge_sets(sets, sets.get(r, c + 2), to, r);
            }
        }
    }

    fn merge_down_a_row(
        &self,
        sets: &mut Sets,
        start_row: i64,
        rng: &mut Rng,
    ) -> Result<(), MazeError> {
        let d = self.dims;
        if start_row == d.hh - 2 {
            return Ok(());
        }

        // cells of each set in this row, in first-seen order
        let mut set_counts: Vec<(i64, Vec<i64>)> = Vec::new();
        for c in (1..d.ww).step_by(2) {
            let s = sets.get(start_row, c);
            match set_counts.iter_mut().find(|(k, _)| *k == s) {
                Some((_, cols)) => cols.push(c),
                None => set_counts.push((s, vec![c])),
            }
        }

        // merge down randomly, but at least once per set
        for (s, cols) in &set_counts {
            let c = rng.py.choice(cols)?;
            sets.set(start_row + 1, c, *s);
            sets.set(start_row + 2, c, *s);
        }

        for c in (1..d.ww - 2).step_by(2) {
            if rng.py.random() < self.yskew {
                let s = sets.get(start_row, c);
                if sets.get(start_row + 1, c) == -1 {
                    sets.set(start_row + 1, c, s);
                    sets.set(start_row + 2, c, s);
                }
            }
        }
        Ok(())
    }

    /// Relabel `from_set` as `to_set` in rows `1..=max_row`.
    fn merge_sets(&self, sets: &mut Sets, from_set: i64, to_set: i64, max_row: i64) {
        for r in 1..=max_row {
            for c in 1..self.dims.ww - 1 {
                if sets.get(r, c) == from_set {
                    sets.set(r, c, to_set);
                }
            }
        }
    }

    fn process_last_row(&self, sets: &mut Sets) {
        let r = self.dims.hh - 2;
        for c in (1..self.dims.ww - 2).step_by(2) {
            if sets.get(r, c) != sets.get(r, c + 2) {
                let to = sets.get(r, c);
                sets.set(r, c + 1, to);
                self.merge_sets(sets, sets.get(r, c + 2), to, self.dims.hh - 1);
            }
        }
    }
}

impl MazeGenerator for Ellers {
    fn generate(&mut self, rng: &mut Rng) -> Result<Grid, MazeError> {
        let d = self.dims;
        let mut sets = Sets {
            width: d.ww as usize,
            ids: vec![-1; (d.hh * d.ww) as usize],
        };

        let mut max_set_number = 0;
        for r in (1..d.hh - 1).step_by(2) {
            max_set_number = self.init_row(&mut sets, r, max_set_number);
            self.merge_one_row(&mut sets, r, rng);
            self.merge_down_a_row(&mut sets, r, rng)?;
        }
        self.init_row(&mut sets, d.hh - 2, max_set_number);
        self.process_last_row(&mut sets);

        let mut grid = d.grid(0);
        for r in 0..d.hh {
            for c in 0..d.ww {
                if sets.get(r, c) == -1 {
                    grid.set((r, c), 1);
                }
            }
        }
        Ok(grid)
    }
}
