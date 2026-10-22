# Feature inventory and parity matrix

Source: john-science/mazelib@56cd88294a36746bbd324c2c9f7e4ab26095deb3 · Target: Rust (stable, edition 2021) · Mode: transpile

Status: `todo` → `spec` → `building` → `verified@Lx` (or `deviates` with a note).

| # | Module / feature | Public surface | Depends on | Level | Status | Level reached | Deviations |
|---|---|---|---|---|---|---|---|
| 1 | RNG: Python `random` (MT19937, init_by_array, getrandbits, randrange/randint/choice/shuffle/random) | `rng::PyRandom` | — | L2 | verified@L2 | L2 | None |
| 2 | RNG: NumPy legacy `RandomState` (init_genrand seed, random_interval, shuffle) | `rng::NumpyRandom`, `rng::Rng` | — | L2 | verified@L2 | L2 | None |
| 3 | CPython set iteration order (tuple hash, open addressing, resize, merge) | `pyset::PySet` (internal) | — | L2 | verified@L2 | L2 | None |
| 4 | Grid | `Grid` | — | L1 | verified@L2 | L2 (every case round-trips grids) | None |
| 5 | MazeGenAlgo base (`_find_neighbors`, size assert) | `generate::*` helpers | 1,2,4 | L2 | verified@L2 | L2 (+L1) | None |
| 6 | AldousBroder | `AldousBroder` | 5 | L2 | verified@L2 | L2 (+L1) | None |
| 7 | BacktrackingGenerator | `BacktrackingGenerator` | 5 | L2 | verified@L2 | L2 (+L1) | None |
| 8 | BinaryTree | `BinaryTree` | 5 | L2 | verified@L2 | L2 (+L1) | None |
| 9 | CellularAutomaton | `CellularAutomaton` | 5 | L2 | verified@L2 | L2 (+L1) | None |
| 10 | Division | `Division` | 5 | L2 | verified@L2 | L2 (+L1) | None |
| 11 | DungeonRooms | `DungeonRooms` | 3,5 | L2 | verified@L2 | L2 (+L1) | None. (Uses Python `random.shuffle`, not NumPy's, in its own `_find_unblocked_neighbors`; reproduced.) |
| 12 | Ellers | `Ellers` | 5 | L2 | verified@L2 | L2 (+L1) | Known bug fixed: set ids are `i64`, so mazes needing >127 sets (KNOWN_BUGS `gen/Ellers/v2/h12w17/*`) return a valid perfect maze instead of `OverflowError`. |
| 13 | GrowingTree | `GrowingTree` | 5 | L2 | verified@L2 | L2 (+L1) | None |
| 14 | HuntAndKill | `HuntAndKill` | 5 | L2 | verified@L2 | L2 (+L1) | None |
| 15 | Kruskal | `Kruskal` | 5 | L2 | verified@L2 | L2 (+L1) | None |
| 16 | Prims | `Prims` | 3,5 | L2 | verified@L2 | L2 (+L1) | None |
| 17 | Sidewinder | `Sidewinder` | 5 | L2 | verified@L2 | L2 (+L1) | None |
| 18 | TrivialMaze | `TrivialMaze` | 5 | L2 | verified@L2 | L2 (+L1) | None |
| 19 | Wilsons | `Wilsons` | 5 | L2 | verified@L2 | L2 (+L1) | None |
| 20 | MazeSolveAlgo base (preprocess asserts, neighbors, prune) | `solve::*` helpers | 1,2,4 | L2 | verified@L2 | L2 (+L1) | None |
| 21 | BacktrackingSolver | `BacktrackingSolver` | 20 | L2 | verified@L2 | L2 (+L1) | None |
| 22 | Chain | `Chain` | 20 | L2 | verified@L2 | L2 (+L1) | None |
| 23 | Collision | `Collision` | 20 | L2 | verified@L2 | L2 (+L1) | Known bug fixed: where the original raises `TypeError` on a robot collision (mazes with loops; KNOWN_BUGS `solve/Collision/*`), return one BFS shortest path over open squares. |
| 24 | RandomMouse | `RandomMouse` | 20 | L2 | verified@L2 | L2 (+L1) | None |
| 25 | ShortestPath | `ShortestPath` | 3,20 | L2 | verified@L2 | L2 (+L1) | None. (Keeps the original's `end_edge = _on_edge(start)` bug.) |
| 26 | ShortestPaths | `ShortestPaths` | 3,20 | L2 | verified@L2 | L2 (+L1) | None |
| 27 | Tremaux | `Tremaux` | 20 | L2 | verified@L2 | L2 (+L1) | None |
| 28 | MazeTransmuteAlgo base | `transmute::*` helpers | 1,2,4 | L2 | verified@L2 | L2 | None |
| 29 | CuldeSacFiller | `CuldeSacFiller` | 28 | L2 | verified@L2 | L2 (+L1) | None |
| 30 | DeadEndFiller | `DeadEndFiller` | 28 | L2 | verified@L2 | L2 (+L1) | None. (Keeps the one-dead-end-per-iteration behavior.) |
| 31 | Perturbation | `Perturbation` | 3,28 | L2 | verified@L2 | L2 (+L1) | None |
| 32 | Maze facade (seed, generate, entrances, solve, transmute, tostring, monte carlo) | `Maze` | all | L2 | verified@L2 | L2 (+L1) | None |
| 33 | Adapter binary | `mazelib-adapter` | 32 | L2 | verified@L2 | L2 | None |

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
| test/test_generators.py (21 tests) | tests/generators.rs | ported, passing (+ Ellers >127-sets regression) |
| test/test_solvers.py (8 tests) | tests/solvers.rs | ported, passing (+ Collision-with-loops regression) |
| test/test_transmuters.py (3 tests) | tests/transmuters.rs | ported, passing |
| test/test_maze.py (9 tests) | tests/maze.rs | ported, passing (+ seed-range test) |
| — | tests/exact.rs | 15 exact-output tests recorded from the original (one per generator + entrances/ShortestPaths) |
| — | `src/pyset.rs`, `src/rng.rs` unit tests | CPython hash/set-order and RNG spot checks |

## L2 results (goldens, profile `tx`)

| Case set | Exact (compare) | Not compared (`tx: invariant`) | Invariant check |
|---|---|---|---|
| rng | 63/63 | 0 | 63/63 |
| generate | 1564/1564 | 8 (Ellers KNOWN_BUGS) | 1572/1572 |
| entrances | 200/200 | 0 | 200/200 |
| solve | 372/372 | 12 (Collision KNOWN_BUGS) | 384/384 |
| transmute | 120/120 | 0 | 120/120 |
| tostring | 20/20 | 0 | 20/20 |
| monte_carlo | 37/37 | 0 | 37/37 |
| **total** | **2376/2376** | **20** | **2396/2396** |
