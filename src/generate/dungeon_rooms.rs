use super::{Dims, HuntOrder, MazeGenerator};
use crate::pyset::PySet;
use crate::{ensure, midpoint, unblocked_cells, Cell, Grid, MazeError, Rng};

/// A room: its top-left and bottom-right **grid** coordinates (ideally odd).
pub type Room = (Cell, Cell);

/// The Dungeon Rooms algorithm: Hunt-and-Kill around pre-carved rooms,
/// followed by a pass that reconnects any disjoint passages.
#[derive(Clone, Debug)]
pub struct DungeonRooms {
    dims: Dims,
    backup_grid: Grid,
    grid: Grid,
    rooms: Option<Vec<Room>>,
    hunt_order: HuntOrder,
}

impl DungeonRooms {
    /// A generator for an `h` x `w` maze. A pre-built `grid` overrides `h`
    /// and `w`; `rooms` are carved (with one door each) before generating.
    pub fn new(
        h: i64,
        w: i64,
        rooms: Option<Vec<Room>>,
        grid: Option<Grid>,
        hunt_order: HuntOrder,
    ) -> Result<Self, MazeError> {
        let (h, w, backup_grid) = match grid {
            Some(g) => (
                (g.height() as i64 - 1).div_euclid(2),
                (g.width() as i64 - 1).div_euclid(2),
                g,
            ),
            None => {
                if 2 * h + 1 < 0 || 2 * w + 1 < 0 {
                    return Err(MazeError::Value(
                        "negative dimensions are not allowed".into(),
                    ));
                }
                (
                    h,
                    w,
                    Grid::filled((2 * h + 1) as usize, (2 * w + 1) as usize, 1),
                )
            }
        };
        let dims = Dims::new(h, w)?;
        Ok(DungeonRooms {
            dims,
            grid: backup_grid.clone(),
            backup_grid,
            rooms,
            hunt_order,
        })
    }

    fn carve_rooms(&mut self, rng: &mut Rng) {
        let Some(rooms) = self.rooms.clone() else {
            return;
        };
        for (top_left, bottom_right) in rooms {
            // an invalid room is simply ignored (whatever was carved stays)
            let _ = self
                .carve_room(top_left, bottom_right)
                .and_then(|()| self.carve_door(top_left, bottom_right, rng));
        }
    }

    fn carve_room(&mut self, top_left: Cell, bottom_right: Cell) -> Result<(), MazeError> {
        for row in top_left.0..=bottom_right.0 {
            for col in top_left.1..=bottom_right.1 {
                self.grid.py_set((row, col), 0)?;
            }
        }
        Ok(())
    }

    /// Open one door in a room, unless a corner sits on an even coordinate.
    fn carve_door(&mut self, tl: Cell, br: Cell, rng: &mut Rng) -> Result<(), MazeError> {
        if [tl.0, tl.1, br.0, br.1]
            .iter()
            .any(|i| i.rem_euclid(2) == 0)
        {
            return Ok(());
        }
        let odd_rows: Vec<i64> = (tl.0 - 1..br.0 + 2)
            .filter(|i| i.rem_euclid(2) == 1)
            .collect();
        let odd_cols: Vec<i64> = (tl.1 - 1..br.1 + 2)
            .filter(|i| i.rem_euclid(2) == 1)
            .collect();

        // The original pairs the top/bottom walls with odd *rows*; kept as is.
        let mut doors: Vec<Cell> = Vec::new();
        if tl.0 > 2 {
            doors.extend(odd_rows.iter().map(|&r| (tl.0 - 1, r)));
        }
        if tl.1 > 2 {
            doors.extend(odd_cols.iter().map(|&c| (c, tl.1 - 1)));
        }
        if br.0 < self.grid.height() as i64 - 2 {
            doors.extend(odd_rows.iter().map(|&r| (br.0 + 1, r)));
        }
        if br.1 < self.grid.width() as i64 - 2 {
            doors.extend(odd_cols.iter().map(|&c| (c, br.1 + 1)));
        }
        let door = rng.py.choice(&doors)?;
        self.grid.py_set(door, 0)
    }

    /// A random walk from a visited cell until there are no unvisited neighbors.
    fn walk(&mut self, start: Cell, rng: &mut Rng) -> Result<(), MazeError> {
        if self.grid.py_get(start)? != 0 {
            return Ok(());
        }
        let d = self.dims;
        let mut current = start;
        let mut unvisited = d.find_neighbors(current, &self.grid, true, rng);
        while !unvisited.is_empty() {
            let neighbor = rng.py.choice(&unvisited)?;
            self.grid.set(neighbor, 0);
            self.grid.set(midpoint(neighbor, current), 0);
            current = neighbor;
            unvisited = d.find_neighbors(current, &self.grid, true, rng);
        }
        Ok(())
    }

    fn hunt(&self, count: i64, rng: &mut Rng) -> Result<Cell, MazeError> {
        let d = self.dims;
        match self.hunt_order {
            HuntOrder::Serpentine => {
                let mut cell = (1, 1);
                loop {
                    cell = (cell.0, cell.1 + 2);
                    if cell.1 > d.ww - 2 {
                        cell = (cell.0 + 2, 1);
                        if cell.0 > d.hh - 2 {
                            return Ok((-1, -1));
                        }
                    }
                    if self.grid.py_get(cell)? == 0
                        && !d.find_neighbors(cell, &self.grid, true, rng).is_empty()
                    {
                        return Ok(cell);
                    }
                }
            }
            HuntOrder::Random => {
                if count >= d.hh * d.ww {
                    Ok((-1, -1))
                } else {
                    d.random_cell(rng)
                }
            }
        }
    }

    /// A random starting cell that is not already open (not inside a room).
    fn choose_start(&self, rng: &mut Rng) -> Result<Cell, MazeError> {
        let d = self.dims;
        let mut current = d.random_cell(rng)?;
        let limit = d.hh * d.ww * 2;
        let mut num_tries = 1;
        while num_tries < limit {
            current = d.random_cell(rng)?;
            if self.grid.py_get(current)? == 1 {
                break;
            }
            num_tries += 1;
        }
        ensure(
            num_tries < limit,
            "The grid input to DungeonRooms was invalid.",
        )?;
        Ok(current)
    }

    /// Open walls until every passage is connected.
    fn reconnect_maze(&mut self, rng: &mut Rng) -> Result<(), MazeError> {
        let passages = find_all_passages(&self.grid, ShuffleWith::Python, rng)?;
        let d = self.dims;
        fix_disjoint_passages(
            &mut self.grid,
            passages,
            |cell, grid, rng| Ok(d.find_neighbors(cell, grid, false, rng)),
            rng,
        )
    }
}

/// Which generator shuffles the neighbor lists in [`find_all_passages`]:
/// `DungeonRooms` imports Python's `random.shuffle`, while the transmuters
/// use NumPy's.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShuffleWith {
    Python,
    Numpy,
}

/// Group every cell with the open cells it connects to; disjoint passages
/// end up in different sets (`_find_all_passages`).
pub(crate) fn find_all_passages(
    grid: &Grid,
    shuffle: ShuffleWith,
    rng: &mut Rng,
) -> Result<Vec<PySet<Cell>>, MazeError> {
    let mut passages: Vec<PySet<Cell>> = Vec::new();
    for r in (1..grid.height() as i64).step_by(2) {
        for c in (1..grid.width() as i64).step_by(2) {
            let mut ns = unblocked_cells(grid, (r, c))?;
            match shuffle {
                ShuffleWith::Python => rng.py.shuffle(&mut ns),
                ShuffleWith::Numpy => rng.np.shuffle(&mut ns),
            }
            ns.push((r, c));
            let current = PySet::from_items(ns);

            match passages.iter().position(|p| current.intersects(p)) {
                Some(i) => passages[i] = passages[i].union(&current),
                None => passages.push(current),
            }
        }
    }
    Ok(join_intersecting_sets(passages))
}

/// Combine sets that intersect (`_join_intersecting_sets`).
fn join_intersecting_sets(sets: Vec<PySet<Cell>>) -> Vec<PySet<Cell>> {
    let mut sets: Vec<Option<PySet<Cell>>> = sets.into_iter().map(Some).collect();
    for i in 0..sets.len().saturating_sub(1) {
        if sets[i].is_none() {
            continue;
        }
        for j in i + 1..sets.len() {
            let Some(other) = &sets[j] else {
                continue;
            };
            let current = sets[i].as_ref().expect("checked above");
            if current.intersects(other) {
                sets[i] = Some(current.union(other));
                sets[j] = None;
            }
        }
    }
    sets.into_iter().flatten().collect()
}

/// Knock down walls between the first passage and the others until only
/// one passage is left (`_fix_disjoint_passages`).
pub(crate) fn fix_disjoint_passages<F>(
    grid: &mut Grid,
    mut passages: Vec<PySet<Cell>>,
    mut open_neighbors: F,
    rng: &mut Rng,
) -> Result<(), MazeError>
where
    F: FnMut(Cell, &Grid, &mut Rng) -> Result<Vec<Cell>, MazeError>,
{
    while passages.len() > 1 {
        let mut found = false;
        while !found {
            // a random cell in the first passage
            let cell = rng.py.choice(&passages[0].to_vec())?;
            let neighbors = open_neighbors(cell, grid, rng)?;
            // does it neighbor any other passage?
            for idx in 1..passages.len() {
                let hit = neighbors.iter().find(|n| passages[idx].contains(n));
                if let Some(&n) = hit {
                    grid.py_set(midpoint(n, cell), 0)?;
                    passages[0] = passages[0].union(&passages[idx]);
                    // `list.remove` drops the first *equal* set, which can be
                    // the merged first passage itself
                    let first_equal = passages
                        .iter()
                        .position(|p| p.set_eq(&passages[idx]))
                        .unwrap_or(idx);
                    passages.remove(first_equal);
                    found = true;
                    break;
                }
            }
        }
    }
    Ok(())
}

impl MazeGenerator for DungeonRooms {
    fn generate(&mut self, rng: &mut Rng) -> Result<Grid, MazeError> {
        self.grid = self.backup_grid.clone();
        self.carve_rooms(rng);

        let mut current = self.choose_start(rng)?;
        self.grid.py_set(current, 0)?;

        // many random walks, to fill the maze
        let mut num_trials = 0;
        while current != (-1, -1) {
            self.walk(current, rng)?;
            current = self.hunt(num_trials, rng)?;
            num_trials += 1;
        }

        self.reconnect_maze(rng)?;
        Ok(self.grid.clone())
    }
}
