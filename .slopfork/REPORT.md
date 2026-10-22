# Slopfork report: mazelib (Python) → mazelib (Rust)

- Source: john-science/mazelib@56cd88294a36746bbd324c2c9f7e4ab26095deb3 (mazelib 0.9.17, MIT, © 2014 John Stilley)
- Target: Rust, stable, edition 2021, `#![forbid(unsafe_code)]`, zero-dependency library + `serde_json` adapter
- Mode: transpile · Compare profile: `tx` · Date: 2026-10-22

## Parity
| Module | Target level | Reached | Cases (pass/total) | Deviations |
|---|---|---|---|---|
| RNG: Python `random` | L2 | L2 | rng 63/63 exact, plus every seeded case | — |
| RNG: NumPy legacy `RandomState` | L2 | L2 | rng 63/63 exact, plus every seeded case | — |
| CPython set order (`PySet`) | L2 | L2 | via Prims/DungeonRooms/Perturbation/ShortestPath(s) cases; 900 random ops vs CPython 3.13 | — |
| Grid | L1 | L2 | every case | — |
| Generators (14) | L2 | L2 + L1 | generate 1564/1564 exact, 1572/1572 invariant | Ellers: no `int8` overflow (8 KNOWN_BUGS cases now valid) |
| Solvers (7) | L2 | L2 + L1 | solve 372/372 exact, 384/384 invariant | Collision: BFS fallback where the original crashes (12 KNOWN_BUGS cases now valid) |
| Transmuters (3) | L2 | L2 + L1 | transmute 120/120 exact and invariant | — |
| Maze facade (entrances, tostring, monte carlo) | L2 | L2 + L1 | entrances 200/200, tostring 20/20, monte_carlo 37/37 (exact and invariant) | — |
| Adapter | L2 | L2 | all 2,396 cases | — |

Overall: 33/33 rows of `features.md` at or above their target level.

## Verification details
L0: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo build` clean.
L1: 41/41 source tests ported (`tests/generators.rs`, `solvers.rs`, `transmuters.rs`, `maze.rs`), 0 skipped; plus 15 exact-output tests recorded from the original (`tests/exact.rs`), 2 KNOWN_BUGS regressions, and unit tests for the RNG, CPython hashing/set order and Python string slicing. 67 tests total (including one doctest), all passing.
L2: 2,396 cases. Exact (`tx`): **2,376/2,376**. Invariant-checked: **2,396/2,396** (includes the 20 KNOWN_BUGS cases, which are `tx: invariant`). Distributional: n/a (exact match makes it moot).
L3/L4/L5: not applicable (library, no wire protocol or UI; no module warranted formal methods).

The harness itself was validated first: the original (installed in a venv) reproduces its own goldens (`rng` 63/63, `solve` 372/372, `generate` 1564/1564).

## Known deviations and unknowns
- **Ellers** (`gen/Ellers/v2/h12w17/s0..s7`): the original raises `OverflowError` once set ids exceed `int8`; the port uses `i64` ids and returns a perfect maze. Identical output whenever the original succeeds.
- **Collision** (`solve/Collision/{CellularAutomaton6x6,DungeonRooms10x10}/*`): the original raises `TypeError` as soon as two flood robots meet (any maze with loops). The port detects that point and returns one shortest path from a breadth-first search over open squares. Identical output on loop-free mazes.
- Non-crashing original bugs are preserved on purpose (`ShortestPath`'s `end_edge` check uses the start; `DeadEndFiller` fills one dead end per iteration; `DungeonRooms._carve_door` pairs walls with odd rows).
- The set-order emulation targets CPython 3.13 (the version the goldens were recorded with). Outputs of Prims/DungeonRooms/Perturbation/ShortestPath(s) would differ from a run of the original under a CPython with a different set implementation or tuple hash.
- Unseeded `Maze::new()` seeds from the clock; the original seeds from OS entropy. Not observable through the protocol (only `tostring` cases are unseeded, and it draws nothing).

## Surprises
- The only failure after the first full build (51 DungeonRooms + 18 monte_carlo cases) came from one import line: `DungeonRooms` does `from random import shuffle`, so its own neighbor search shuffles with **Python's** RNG, while every other algorithm shuffles with NumPy's.
- Several outputs depend on **CPython set iteration order** (hash-table layout of tuple sets), not just the RNG: Prims rebuilds its frontier with `list(set(...))` every step. Matching it meant emulating `setobject.c`, including the copy/merge fast paths in `set.union`.
- Python's `random.choice` on a one-element list still consumes a 32-bit word (`_randbelow(1)` calls `getrandbits(1)`); NumPy's shuffle of a one-element list consumes nothing.

## Provenance
- Sealed-path touches: 19 logged, all `allow` (transpile mode, main thread), 0 denied (see `.slopfork/provenance.jsonl`). The cleanroom repos (`mazelib-*-cr`) were never read.
- Leak audit (specs): n/a in transpile mode · Leak audit (code): n/a in transpile mode (the port is a declared derivative work).

## License and attribution
mazelib is MIT-licensed. The port is a derivative work, so it keeps the copyright and permission notice in `NOTICE` and `LICENSE` and states that it is a port of mazelib@56cd882. The goldens repo's data and tooling are MIT-licensed as well. This is not legal advice.
