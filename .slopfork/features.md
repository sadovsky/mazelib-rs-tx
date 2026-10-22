# Feature inventory and parity matrix

Source: john-science/mazelib@56cd88294a36746bbd324c2c9f7e4ab26095deb3 · Target: Rust (stable, edition 2021) · Mode: transpile

Status: `todo` → `spec` → `building` → `verified@Lx` (or `deviates` with a note).

| # | Module / feature | Public surface | Depends on | Level | Status | Level reached | Deviations |
|---|---|---|---|---|---|---|---|
| 1 | RNG: Python `random` (MT19937, init_by_array, getrandbits, randrange/randint/choice/shuffle/random) | `rng::PyRandom` | — | L2 | todo | — | |
| 2 | RNG: NumPy legacy `RandomState` (init_genrand seed, random_interval, shuffle) | `rng::NumpyRandom`, `rng::Rng` | — | L2 | todo | — | |
| 3 | CPython set iteration order (tuple hash, open addressing, resize, merge) | `pyset::PySet` (internal) | — | L2 | todo | — | |
| 4 | Grid | `Grid` | — | L1 | todo | — | |
| 5 | MazeGenAlgo base (`_find_neighbors`, size assert) | `generate::*` helpers | 1,2,4 | L2 | todo | — | |
| 6 | AldousBroder | `AldousBroder` | 5 | L2 | todo | — | |
| 7 | BacktrackingGenerator | `BacktrackingGenerator` | 5 | L2 | todo | — | |
| 8 | BinaryTree | `BinaryTree` | 5 | L2 | todo | — | |
| 9 | CellularAutomaton | `CellularAutomaton` | 5 | L2 | todo | — | |
| 10 | Division | `Division` | 5 | L2 | todo | — | |
| 11 | DungeonRooms | `DungeonRooms` | 3,5 | L2 | todo | — | |
| 12 | Ellers | `Ellers` | 5 | L2 | todo | — | |
| 13 | GrowingTree | `GrowingTree` | 5 | L2 | todo | — | |
| 14 | HuntAndKill | `HuntAndKill` | 5 | L2 | todo | — | |
| 15 | Kruskal | `Kruskal` | 5 | L2 | todo | — | |
| 16 | Prims | `Prims` | 3,5 | L2 | todo | — | |
| 17 | Sidewinder | `Sidewinder` | 5 | L2 | todo | — | |
| 18 | TrivialMaze | `TrivialMaze` | 5 | L2 | todo | — | |
| 19 | Wilsons | `Wilsons` | 5 | L2 | todo | — | |
| 20 | MazeSolveAlgo base (preprocess asserts, neighbors, prune) | `solve::*` helpers | 1,2,4 | L2 | todo | — | |
| 21 | BacktrackingSolver | `BacktrackingSolver` | 20 | L2 | todo | — | |
| 22 | Chain | `Chain` | 20 | L2 | todo | — | |
| 23 | Collision | `Collision` | 20 | L2 | todo | — | |
| 24 | RandomMouse | `RandomMouse` | 20 | L2 | todo | — | |
| 25 | ShortestPath | `ShortestPath` | 3,20 | L2 | todo | — | |
| 26 | ShortestPaths | `ShortestPaths` | 3,20 | L2 | todo | — | |
| 27 | Tremaux | `Tremaux` | 20 | L2 | todo | — | |
| 28 | MazeTransmuteAlgo base | `transmute::*` helpers | 1,2,4 | L2 | todo | — | |
| 29 | CuldeSacFiller | `CuldeSacFiller` | 28 | L2 | todo | — | |
| 30 | DeadEndFiller | `DeadEndFiller` | 28 | L2 | todo | — | |
| 31 | Perturbation | `Perturbation` | 3,28 | L2 | todo | — | |
| 32 | Maze facade (seed, generate, entrances, solve, transmute, tostring, monte carlo) | `Maze` | all | L2 | todo | — | |
| 33 | Adapter binary | `mazelib-adapter` | 32 | L2 | todo | — | |

## Cross-cutting decisions
| Concern | Source behavior | Target strategy |
|---|---|---|
| Randomness / RNG | Global `random` (MT19937, `getrandbits`-based `_randbelow`) and global `numpy.random` legacy `RandomState` (only `shuffle` is used, on Python lists of tuples). | Port both MT19937 streams bit for bit into `rng::Rng { py, np }`; pass `&mut Rng` explicitly instead of global state. Every draw happens in the same order as the original (including `choice` on 1-element lists, which still consumes a word, and NumPy shuffles of 0/1-element lists, which consume nothing). |
| Set iteration order | `DungeonRooms`, `Perturbation`, `Prims`, `ShortestPath(s)` iterate Python `set`s of tuples, so their output depends on CPython's set layout. | Emulate CPython 3.13 `setobject.c` (tuple xxHash, linear probing + perturbation, resize policy, `set_merge` fast paths) in `pyset`. Verified against CPython in a test. |
| Dict order | Insertion-ordered dicts (Ellers `set_counts`, Tremaux visit counts). | `Vec` of pairs / insertion-ordered lookups. |
| Numeric types | Python unbounded ints; grids are NumPy `int8`. Ellers stores set ids in `int8` and overflows (KNOWN BUG). | `i64` coordinates, `u8` grid cells; Ellers set ids in `i64` (no overflow, so the known-bug cases return a valid maze). |
| Indexing | NumPy/Python negative indices wrap; out-of-range raises `IndexError`. | `Grid` exposes Python-semantics checked access where inputs can reach it (user rooms, solver inputs); internal loops use plain indexing proven in range. |
| Errors | `AssertionError`, `ValueError`, `IndexError`, `TypeError`, `OverflowError`. | `MazeError` enum; the adapter maps variants to Python class names. |
| I/O, time, concurrency | None in the library. | None. Unseeded `Rng::from_entropy` uses the system clock. |

## Source tests (L1)
| Source test | Target test | Status |
|---|---|---|
| test/test_generators.py | tests/generators.rs | todo |
| test/test_solvers.py | tests/solvers.rs | todo |
| test/test_transmuters.py | tests/transmuters.rs | todo |
| test/test_maze.py | tests/maze.rs | todo |
