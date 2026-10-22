//! Maze-generating algorithms (`mazelib.generate`).

use crate::{ensure, Cell, Grid, MazeError, Rng};

mod aldous_broder;
mod backtracking_generator;
mod binary_tree;
mod cellular_automaton;
mod division;
mod dungeon_rooms;
mod ellers;
mod growing_tree;
mod hunt_and_kill;
mod kruskal;
mod prims;
mod sidewinder;
mod trivial_maze;
mod wilsons;

pub use aldous_broder::AldousBroder;
pub use backtracking_generator::BacktrackingGenerator;
pub use binary_tree::{BinaryTree, Skew};
pub use cellular_automaton::CellularAutomaton;
pub use division::Division;
pub(crate) use dungeon_rooms::{find_all_passages, fix_disjoint_passages, ShuffleWith};
pub use dungeon_rooms::{DungeonRooms, Room};
pub use ellers::Ellers;
pub use growing_tree::GrowingTree;
pub use hunt_and_kill::HuntAndKill;
pub use kruskal::Kruskal;
pub use prims::Prims;
pub use sidewinder::Sidewinder;
pub use trivial_maze::{MazeType, TrivialMaze};
pub use wilsons::Wilsons;

/// How hunt-based generators pick the next cell to walk from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum HuntOrder {
    /// Pick cells at random (the default).
    #[default]
    Random,
    /// Scan the grid row by row.
    Serpentine,
}

impl HuntOrder {
    /// Parse like the original: `"serpentine"` (case-insensitive, trimmed)
    /// selects [`HuntOrder::Serpentine`]; anything else is [`HuntOrder::Random`].
    pub fn parse(s: &str) -> Self {
        if s.trim().to_lowercase() == "serpentine" {
            HuntOrder::Serpentine
        } else {
            HuntOrder::Random
        }
    }
}

/// A maze-generating algorithm (`MazeGenAlgo`).
pub trait MazeGenerator {
    /// Generate a new maze grid.
    fn generate(&mut self, rng: &mut Rng) -> Result<Grid, MazeError>;
}

/// Maze dimensions shared by every generator: `h` x `w` cells in an
/// `H` x `W` grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Dims {
    pub h: i64,
    pub w: i64,
    pub hh: i64,
    pub ww: i64,
}

impl Dims {
    /// `MazeGenAlgo.__init__`: mazes must be at least 3x3 cells.
    pub fn new(h: i64, w: i64) -> Result<Self, MazeError> {
        ensure(w >= 3 && h >= 3, "Mazes cannot be smaller than 3x3.")?;
        Ok(Dims {
            h,
            w,
            hh: 2 * h + 1,
            ww: 2 * w + 1,
        })
    }

    /// A full-size grid with every square set to `value`.
    pub fn grid(&self, value: u8) -> Grid {
        Grid::filled(self.hh as usize, self.ww as usize, value)
    }

    /// `_find_neighbors`: the cells two squares away whose wall state is
    /// `is_wall`, shuffled with NumPy's generator.
    pub fn find_neighbors(
        &self,
        (r, c): Cell,
        grid: &Grid,
        is_wall: bool,
        rng: &mut Rng,
    ) -> Vec<Cell> {
        let want = u8::from(is_wall);
        let mut ns = Vec::with_capacity(4);
        if r > 1 && grid.at((r - 2, c)) == want {
            ns.push((r - 2, c));
        }
        if r < self.hh - 2 && grid.at((r + 2, c)) == want {
            ns.push((r + 2, c));
        }
        if c > 1 && grid.at((r, c - 2)) == want {
            ns.push((r, c - 2));
        }
        if c < self.ww - 2 && grid.at((r, c + 2)) == want {
            ns.push((r, c + 2));
        }
        rng.np.shuffle(&mut ns);
        ns
    }

    /// A random cell (odd coordinates): `(randrange(1, H, 2), randrange(1, W, 2))`.
    pub fn random_cell(&self, rng: &mut Rng) -> Result<Cell, MazeError> {
        let r = rng.py.randrange(1, self.hh, 2)?;
        let c = rng.py.randrange(1, self.ww, 2)?;
        Ok((r, c))
    }
}
