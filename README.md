# mazelib-rs-tx

A Rust port of [mazelib](https://github.com/john-science/mazelib) (Python, MIT, © 2014 John Stilley) at `56cd882` (0.9.17), made with the [slopfork](https://github.com/sadovsky/slopfork) process in **transpile** mode. See [`NOTICE`](NOTICE).

For a given seed it produces **the same mazes, entrances and solutions as the original**: it reimplements Python's `random` (MT19937, `getrandbits`-based `randrange`/`choice`/`shuffle`, 53-bit `random()`), NumPy's legacy `RandomState` (`seed`, `shuffle`) and CPython 3.13's set iteration order, and makes the same random calls in the same order. All 2,396 cases of [slopfork-mazelib-goldens](https://github.com/sadovsky/slopfork-mazelib-goldens) pass under the exact `tx` profile.

- Library crate `mazelib`: stable Rust, edition 2021, `#![forbid(unsafe_code)]`, **no dependencies**.
- Binary `mazelib-adapter`: the goldens' JSON-lines adapter protocol (uses `serde_json`).

```rust
use mazelib::generate::Prims;
use mazelib::solve::ShortestPath;
use mazelib::Maze;

let mut m = Maze::with_seed(123)?;           // Maze(123)
m.generator = Some(Box::new(Prims::new(10, 12)?));
m.generate()?;
m.generate_entrances(true, true)?;
m.solver = Some(Box::new(ShortestPath::new()));
m.solve()?;
println!("{m}");                              // str(m)
# Ok::<(), mazelib::MazeError>(())
```

## Python → Rust API

| Python (`mazelib`) | Rust (`mazelib` crate) | Notes |
|---|---|---|
| `Maze(seed=None)` | `Maze::new()`, `Maze::with_seed(seed)?` | Unseeded mazes seed from the clock. |
| `Maze.set_seed(seed)` | `maze.set_seed(seed)?` | `Err(MazeError::Value)` outside `0..2**32` (NumPy's rule). |
| `m.generator = Prims(h, w)` | `m.generator = Some(Box::new(Prims::new(h, w)?))` | Constructors return `Err(MazeError::Assertion)` below 3x3. |
| `m.generate()` | `m.generate()?` | |
| `m.generate_entrances(start_outer, end_outer)` | `m.generate_entrances(start_outer, end_outer)?` | |
| `m.generate_monte_carlo(repeat, entrances, difficulty, reducer=len)` | `m.generate_monte_carlo(repeat, entrances, difficulty)?`, `m.generate_monte_carlo_with(.., reducer)?` | `reducer: Fn(&Path) -> f64`. |
| `m.transmuters = [X()]`; `m.transmute()` | `m.transmuters = vec![Box::new(X::new())]`; `m.transmute()?` | |
| `m.solver = X()`; `m.solve()`; `m.prune` | `m.solver = Some(Box::new(X::new()))`; `m.solve()?`; `m.prune` | |
| `m.tostring(entrances, solutions)`, `str(m)` | `m.tostring(entrances, solutions)?`, `m.to_string()` / `format!("{m}")` | |
| `m.grid` (`np.ndarray[int8]`) | `m.grid: Option<Grid>` | `Grid` is row-major `Vec<u8>`; `Grid::from_rows`, `to_rows`, `at`, `py_get`. |
| `m.start`, `m.end` (tuples) | `Option<Cell>`, `Cell = (i64, i64)` | |
| `m.solutions` | `Option<Vec<Path>>`, `Path = Vec<Cell>` | |
| `MazeGenAlgo.generate()` | `trait MazeGenerator { fn generate(&mut self, &mut Rng) }` | RNG is explicit instead of global. |
| `MazeSolveAlgo.solve(grid, start, end)` | `trait MazeSolver { fn solve(&mut self, &Grid, Cell, Cell, &mut Rng) }` | |
| `MazeSolveAlgo.prune_solutions(sols)` | `solve::prune_solutions(sols, start, end)` | |
| `MazeTransmuteAlgo.transmute(grid, start, end)` | `trait MazeTransmuter { fn transmute(&mut self, &mut Grid, Option<Cell>, Option<Cell>, &mut Rng) }` | |
| `AldousBroder(h, w)` | `AldousBroder::new(h, w)?` | |
| `BacktrackingGenerator(h, w)` | `BacktrackingGenerator::new(h, w)?` | |
| `BinaryTree(h, w, skew=None)` | `BinaryTree::new(h, w, Option<Skew>, &mut rng)?` | Draws a random skew when `None`, like the original constructor. `Skew::parse("NW")`. |
| `CellularAutomaton(h, w, complexity=1.0, density=1.0)` | `CellularAutomaton::new(h, w, complexity, density)?` | |
| `Division(h, w)` | `Division::new(h, w)?` | |
| `DungeonRooms(h, w, rooms=None, grid=None, hunt_order="random")` | `DungeonRooms::new(h, w, Option<Vec<Room>>, Option<Grid>, HuntOrder)?` | `Room = (Cell, Cell)`; `reconnect_maze(&mut rng)`, `grid()`. Negative sizes: `Err(MazeError::Value)`. |
| `Ellers(h, w, xskew=0.5, yskew=0.5)` | `Ellers::new(h, w, xskew, yskew)?` | |
| `GrowingTree(h, w, backtrack_chance=1.0)` | `GrowingTree::new(h, w, backtrack_chance)?` | |
| `HuntAndKill(h, w, hunt_order="random")` | `HuntAndKill::new(h, w, HuntOrder)?` | `HuntOrder::parse(s)` keeps the original's string handling. |
| `Kruskal(h, w)` | `Kruskal::new(h, w)?` | |
| `Prims(h, w)` | `Prims::new(h, w)?` | |
| `Sidewinder(h, w, skew=0.5)` | `Sidewinder::new(h, w, skew)?` | |
| `TrivialMaze(h, w, maze_type="spiral")` | `TrivialMaze::new(h, w, MazeType)?` | `MazeType::parse(s)`. |
| `Wilsons(h, w, hunt_order="random")` | `Wilsons::new(h, w, HuntOrder)?` | |
| `BacktrackingSolver()` | `BacktrackingSolver::new()` | |
| `Chain(turn="right")` | `Chain::new(Turn)` | `Turn::parse(s)`. |
| `Collision()` | `Collision::new()` | See deviations. |
| `RandomMouse()` | `RandomMouse::new()` | |
| `ShortestPath()` | `ShortestPath::new()` | |
| `ShortestPaths()` | `ShortestPaths::new()` | |
| `Tremaux()` | `Tremaux::new()` | |
| `CuldeSacFiller()` | `CuldeSacFiller::new()` | |
| `DeadEndFiller(iterations=1)` | `DeadEndFiller::new(iterations)` | |
| `Perturbation(repeat=1, new_walls=1)` | `Perturbation::new(repeat, new_walls)` | |
| `random` / `numpy.random` globals | `Rng { py: PyRandom, np: NumpyRandom }` | Public, usable on their own (`rng::PyRandom::randrange`, ...). |

### Errors

The library returns `Result<_, MazeError>`; each variant names the Python exception the original raises (`MazeError::python_kind()`):

| Rust | Python |
|---|---|
| `MazeError::Assertion` | `AssertionError` |
| `MazeError::Value` | `ValueError` |
| `MazeError::Index` | `IndexError` |
| `MazeError::Type` | `TypeError` |

## Deviations from the original

Both are inputs where the original crashes (the goldens' `KNOWN_BUGS.json`); the port returns a valid result instead:

- **Ellers** stores set ids in an `int8` array and raises `OverflowError` once a maze needs more than 127 sets (e.g. 12x17 with `xskew=0.9`). The port uses `i64` set ids. Every other output is unchanged.
- **Collision** raises `TypeError` as soon as two of its flood robots meet (any maze with loops, e.g. from `DungeonRooms` or `CellularAutomaton`). When that would happen, the port returns one shortest path found by a breadth-first search over open squares. On loop-free mazes it behaves exactly like the original.

Bugs that don't crash are kept as they are, e.g. `ShortestPath` checks whether the *start* is on the edge when it means the *end* (so it sometimes returns no solution), `DeadEndFiller` fills one dead end per iteration, and `DungeonRooms` places doors using odd rows as columns.

## Development

```
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
scripts/verify.sh ../slopfork-mazelib-goldens tx   # harness: compare + check on all case sets
```

The adapter is `cargo run --release --bin mazelib-adapter` (JSON lines on stdin; see the goldens' `PROTOCOL.md`). Slopfork state (parity matrix, report, metrics) is in [`.slopfork/`](.slopfork).
