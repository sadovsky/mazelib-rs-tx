# Mapping: mazelib (Python) → mazelib (Rust)

The symbol-by-symbol API table is in the [README](../README.md#python--rust-api). This file records the semantic gaps found while translating.

| Python construct | Rust translation | Gap / note |
|---|---|---|
| Global `random` module | `Rng::py: PyRandom` passed as `&mut Rng` | Bit-exact MT19937 + `init_by_array`; `_randbelow` via `getrandbits(bit_length)`; `choice` of a 1-element list still draws a word. |
| Global `numpy.random` (legacy) | `Rng::np: NumpyRandom` | `init_genrand` seeding, `random_interval` masked rejection; shuffles of 0/1 items draw nothing. |
| `from random import shuffle` in `DungeonRooms` | `rng.py.shuffle` in `find_all_passages(.., ShuffleWith::Python)` | The only place a Python (not NumPy) shuffle is used. Easy to miss: it was the one bug found by the goldens. |
| `set` of tuples, iterated | `pyset::PySet` | Emulates CPython 3.13 `setobject.c` (xxHash tuple hash, linear probes + perturbation, resize ×4 at 60% fill, `set_merge` copy/clean-insert fast paths). Used by Prims, DungeonRooms, Perturbation, ShortestPath(s). |
| `list.remove(set)` | remove first *equal* set | Can remove the merged first passage instead of the matched one when one contains the other. |
| `dict` (insertion order) | `Vec<(K, V)>` / fixed buckets | Ellers `set_counts`, Tremaux visit counts. `HashMap` only where order is never observed (Wilsons walk, Tremaux counts lookup). |
| `np.int8` grid | `Grid` over `Vec<u8>` | Values are always 0/1. |
| `np.int8` Ellers sets | `Vec<i64>` | Removes the overflow (KNOWN_BUG). |
| NumPy indexing (negative wraps, OOB raises) | `Grid::py_get` / `py_set` | Used wherever user input reaches an index (rooms, solver/transmuter neighbors, dead-end filling). |
| String slicing in `tostring` | `splice` with Python slice semantics | |
| `//` | `div_euclid` (positive divisor) / `floor_div` | `randrange` with negative steps uses true floor division. |
| `sorted(key=len)`, `min(key=len)` | `sort_by_key` (stable), `min_by_key` (first minimum) | |
| Exceptions | `MazeError::{Assertion, Value, Index, Type}` | Adapter maps to Python class names. |
| `None` sentinels in paths (`ShortestPath`) | `Vec<Option<Cell>>` | |
| `END` / `DEAD_END` / collided `None` (`Collision`) | `enum Step` + `Collided` signal | Collision → BFS fallback (KNOWN_BUG). |
