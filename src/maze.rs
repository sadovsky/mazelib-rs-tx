use crate::generate::MazeGenerator;
use crate::solve::{prune_solutions, MazeSolver};
use crate::transmute::MazeTransmuter;
use crate::{ensure, Cell, Grid, MazeError, Path, Rng};
use std::fmt;

/// A rectangular 2D maze, with the algorithms used to generate, transmute
/// and solve it, its entrances and its solutions (`mazelib.Maze`).
pub struct Maze {
    /// The generating algorithm used by [`Maze::generate`].
    pub generator: Option<Box<dyn MazeGenerator>>,
    /// The maze grid (`1` wall, `0` open).
    pub grid: Option<Grid>,
    /// The start cell.
    pub start: Option<Cell>,
    /// The end cell.
    pub end: Option<Cell>,
    /// Transmuters applied in order by [`Maze::transmute`].
    pub transmuters: Vec<Box<dyn MazeTransmuter>>,
    /// The solving algorithm used by [`Maze::solve`].
    pub solver: Option<Box<dyn MazeSolver>>,
    /// Solutions found by the last [`Maze::solve`].
    pub solutions: Option<Vec<Path>>,
    /// Whether [`Maze::solve`] prunes backtracking out of solutions.
    pub prune: bool,
    /// The random streams every algorithm draws from.
    pub rng: Rng,
}

impl Default for Maze {
    fn default() -> Self {
        Maze::new()
    }
}

/// `s[:i] + ch + s[i+1:]` with Python slice semantics.
fn splice(s: &str, i: i64, ch: char) -> String {
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len() as i64;
    let norm = |i: i64| -> usize {
        if i < 0 {
            (n + i).max(0) as usize
        } else {
            i.min(n) as usize
        }
    };
    let mut out: String = chars[..norm(i)].iter().collect();
    out.push(ch);
    out.extend(&chars[norm(i + 1)..]);
    out
}

/// `rows[r]` with Python list-index semantics.
fn py_index(len: usize, r: i64) -> Result<usize, MazeError> {
    let j = if r < 0 { r + len as i64 } else { r };
    if (0..len as i64).contains(&j) {
        Ok(j as usize)
    } else {
        Err(MazeError::Index("list index out of range".into()))
    }
}

impl Maze {
    /// An empty maze with an unseeded random generator.
    pub fn new() -> Self {
        Maze {
            generator: None,
            grid: None,
            start: None,
            end: None,
            transmuters: Vec::new(),
            solver: None,
            solutions: None,
            prune: true,
            rng: Rng::from_entropy(),
        }
    }

    /// An empty maze whose random streams are seeded with `seed`.
    pub fn with_seed(seed: i64) -> Result<Self, MazeError> {
        let mut m = Maze::new();
        m.set_seed(seed)?;
        Ok(m)
    }

    /// Seed both random streams (`Maze.set_seed`). Seeds outside
    /// `0..2**32` are rejected, as NumPy rejects them.
    pub fn set_seed(&mut self, seed: i64) -> Result<(), MazeError> {
        self.rng = Rng::seeded(seed)?;
        Ok(())
    }

    /// Generate a new maze with the configured generator, clearing the
    /// entrances and solutions.
    pub fn generate(&mut self) -> Result<(), MazeError> {
        let generator = self.generator.as_mut().ok_or_else(|| {
            MazeError::Assertion("No maze-generation algorithm has been set.".into())
        })?;
        self.grid = Some(generator.generate(&mut self.rng)?);
        self.start = None;
        self.end = None;
        self.solutions = None;
        Ok(())
    }

    fn grid_shape(&self) -> Result<(i64, i64), MazeError> {
        let grid = self
            .grid
            .as_ref()
            .ok_or_else(|| MazeError::Assertion("No maze grid yet exists.".into()))?;
        Ok((grid.height() as i64, grid.width() as i64))
    }

    /// Pick entrances, each on the outer wall or inside the maze. Retries
    /// until start and end are not adjacent.
    pub fn generate_entrances(
        &mut self,
        start_outer: bool,
        end_outer: bool,
    ) -> Result<(), MazeError> {
        loop {
            let (start, end) = match (start_outer, end_outer) {
                (true, true) => self.outer_entrances()?,
                (false, false) => self.inner_entrances()?,
                (true, false) => self.opposite_entrances()?,
                (false, true) => {
                    let (first, second) = self.opposite_entrances()?;
                    (second, first)
                }
            };
            self.start = Some(start);
            self.end = Some(end);
            // the start and end shouldn't be right next to each other
            if (start.0 - end.0).abs() + (start.1 - end.1).abs() >= 2 {
                return Ok(());
            }
        }
    }

    /// Entrances on opposite outer walls.
    fn outer_entrances(&mut self) -> Result<(Cell, Cell), MazeError> {
        let (hh, ww) = self.grid_shape()?;
        let py = &mut self.rng.py;
        Ok(match py.randrange_to(4)? {
            0 => (
                (0, py.randrange(1, ww, 2)?),
                (hh - 1, py.randrange(1, ww, 2)?),
            ),
            1 => (
                (hh - 1, py.randrange(1, ww, 2)?),
                (0, py.randrange(1, ww, 2)?),
            ),
            2 => (
                (py.randrange(1, hh, 2)?, 0),
                (py.randrange(1, hh, 2)?, ww - 1),
            ),
            _ => (
                (py.randrange(1, hh, 2)?, ww - 1),
                (py.randrange(1, hh, 2)?, 0),
            ),
        })
    }

    /// Two distinct entrances inside the maze.
    fn inner_entrances(&mut self) -> Result<(Cell, Cell), MazeError> {
        let (hh, ww) = self.grid_shape()?;
        let py = &mut self.rng.py;
        let start = (py.randrange(1, hh, 2)?, py.randrange(1, ww, 2)?);
        let mut end = (py.randrange(1, hh, 2)?, py.randrange(1, ww, 2)?);
        while end == start {
            end = (py.randrange(1, hh, 2)?, py.randrange(1, ww, 2)?);
        }
        Ok((start, end))
    }

    /// One outer entrance and one inner entrance, in that order.
    fn opposite_entrances(&mut self) -> Result<(Cell, Cell), MazeError> {
        let (hh, ww) = self.grid_shape()?;
        let py = &mut self.rng.py;
        let first = match py.randrange_to(4)? {
            0 => (0, py.randrange(1, ww, 2)?),
            1 => (hh - 1, py.randrange(1, ww, 2)?),
            2 => (py.randrange(1, hh, 2)?, 0),
            _ => (py.randrange(1, hh, 2)?, ww - 1),
        };
        let second = (py.randrange(1, hh, 2)?, py.randrange(1, ww, 2)?);
        Ok((first, second))
    }

    /// Monte Carlo generation of a maze of a given difficulty, measured by
    /// solution length. See [`Maze::generate_monte_carlo_with`].
    pub fn generate_monte_carlo(
        &mut self,
        repeat: usize,
        entrances: usize,
        difficulty: f64,
    ) -> Result<(), MazeError> {
        self.generate_monte_carlo_with(repeat, entrances, difficulty, |p| p.len() as f64)
    }

    /// Monte Carlo generation of a maze of a given difficulty.
    ///
    /// 1. Generate `repeat` mazes.
    /// 2. For each, try `entrances` pairs of outer entrances and keep the one
    ///    with the most solutions.
    /// 3. Order the mazes by `reducer` applied to their first solution.
    /// 4. Keep the maze at position `difficulty` (0 to 1) in that order.
    pub fn generate_monte_carlo_with<F>(
        &mut self,
        repeat: usize,
        entrances: usize,
        difficulty: f64,
        reducer: F,
    ) -> Result<(), MazeError>
    where
        F: Fn(&Path) -> f64,
    {
        ensure(
            (0.0..=1.0).contains(&difficulty),
            "Maze difficulty must be between 0 to 1.",
        )?;

        struct Candidate {
            grid: Grid,
            start: Option<Cell>,
            end: Option<Cell>,
            solutions: Vec<Path>,
        }

        let mut mazes: Vec<Candidate> = Vec::new();
        for _ in 0..repeat {
            self.generate()?;
            let mut best: Option<Candidate> = None;
            for _ in 0..entrances {
                self.generate_entrances(true, true)?;
                self.solve()?;
                let solutions = self.solutions.clone().unwrap_or_default();
                // keep the first entrance pair with the most solutions
                if best
                    .as_ref()
                    .is_none_or(|b| solutions.len() > b.solutions.len())
                {
                    best = Some(Candidate {
                        grid: self.grid.clone().expect("generated"),
                        start: self.start,
                        end: self.end,
                        solutions,
                    });
                }
            }
            mazes.push(
                best.ok_or_else(|| MazeError::Value("max() iterable argument is empty".into()))?,
            );
        }

        // sort by the reduced first solution (stable, like Python's sorted)
        let mut keyed = Vec::with_capacity(mazes.len());
        for m in mazes {
            let first = m
                .solutions
                .first()
                .ok_or_else(|| MazeError::Index("list index out of range".into()))?;
            keyed.push((reducer(first), m));
        }
        keyed.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        let posi = ((keyed.len() as f64 - 1.0) * difficulty) as i64;
        let idx = py_index(keyed.len(), posi)?;
        let chosen = keyed.swap_remove(idx).1;
        self.grid = Some(chosen.grid);
        self.start = chosen.start;
        self.end = chosen.end;
        self.solutions = Some(chosen.solutions);
        Ok(())
    }

    /// Apply every transmuter, in order, to the grid.
    pub fn transmute(&mut self) -> Result<(), MazeError> {
        let grid = self
            .grid
            .as_mut()
            .ok_or_else(|| MazeError::Assertion("No maze grid yet exists to transmute.".into()))?;
        for t in &mut self.transmuters {
            t.transmute(grid, self.start, self.end, &mut self.rng)?;
        }
        Ok(())
    }

    /// Solve the maze with the configured solver (pruning the solutions if
    /// [`Maze::prune`] is set).
    pub fn solve(&mut self) -> Result<(), MazeError> {
        let solver = self.solver.as_mut().ok_or_else(|| {
            MazeError::Assertion("No maze-solving algorithm has been set.".into())
        })?;
        let (Some(start), Some(end)) = (self.start, self.end) else {
            return Err(MazeError::Assertion(
                "Start and end times must be set first.".into(),
            ));
        };
        let grid = self
            .grid
            .as_ref()
            .ok_or_else(|| MazeError::Assertion("Maze grid is not set.".into()))?;
        let mut solutions = solver.solve(grid, start, end, &mut self.rng)?;
        if self.prune {
            solutions = prune_solutions(solutions, start, end);
        }
        self.solutions = Some(solutions);
        Ok(())
    }

    /// The maze as text: `#` walls, spaces for open squares, optionally with
    /// the entrances (`S`, `E`) and the first solution (`+`).
    pub fn tostring(&self, entrances: bool, solutions: bool) -> Result<String, MazeError> {
        let Some(grid) = &self.grid else {
            return Ok(String::new());
        };
        let mut txt: Vec<String> = (0..grid.height())
            .map(|r| {
                grid.row(r)
                    .iter()
                    .map(|&v| if v != 0 { '#' } else { ' ' })
                    .collect()
            })
            .collect();

        if entrances {
            if let (Some(start), Some(end)) = (self.start, self.end) {
                for ((r, c), ch) in [(start, 'S'), (end, 'E')] {
                    let i = py_index(txt.len(), r)?;
                    txt[i] = splice(&txt[i], c, ch);
                }
            }
        }

        if solutions {
            if let Some(first) = self.solutions.as_ref().and_then(|s| s.first()) {
                for &(r, c) in first {
                    let i = py_index(txt.len(), r)?;
                    txt[i] = splice(&txt[i], c, '+');
                }
            }
        }
        Ok(txt.join("\n"))
    }
}

impl fmt::Display for Maze {
    /// The maze with its entrances and solution, if any.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.tostring(true, true) {
            Ok(s) => f.write_str(&s),
            Err(_) => Err(fmt::Error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::splice;

    #[test]
    fn splice_like_python() {
        assert_eq!(splice("abcd", 1, 'X'), "aXcd");
        assert_eq!(splice("abcd", 0, 'X'), "Xbcd");
        assert_eq!(splice("abcd", 3, 'X'), "abcX");
        assert_eq!(splice("abcd", 9, 'X'), "abcdX");
        // "abcd"[:-1] + "X" + "abcd"[0:]
        assert_eq!(splice("abcd", -1, 'X'), "abcXabcd");
    }
}
