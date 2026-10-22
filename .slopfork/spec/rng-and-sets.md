# Spec: RNG and set-order layer

Provenance: derived-from-source (transpile) · Audit: n/a (transpile)

## Purpose
Reproduce every source of nondeterminism in mazelib bit for bit, so `tx` cases match exactly.

## Behavior
1. `Maze.set_seed(s)`: `random.seed(s)` (CPython `init_by_array` over the 32-bit little-endian words of `abs(s)`, `[0]` for 0), then `numpy.random.seed(s)` (`init_genrand(s)`; `ValueError` unless `0 <= s < 2**32`).
2. `random.getrandbits(k)`, `k <= 32`: `genrand_uint32() >> (32 - k)`; larger `k`: 32-bit words, least significant first, the last one shifted right.
3. `_randbelow(n)`: `k = n.bit_length()`; draw `getrandbits(k)` until `< n`. `n == 1` still consumes a word.
4. `randrange(start, stop, step)`: CPython 3.13 algorithm, `ValueError` on empty ranges. `randint(a, b) = randrange(a, b + 1)`.
5. `choice(seq)`: `IndexError` (no draw) if empty, else `seq[_randbelow(len)]`.
6. `shuffle(x)`: for `i` from `len-1` down to 1, `j = _randbelow(i + 1)`, swap.
7. `random()`: `(a >> 5) * 2**26 + (b >> 6)` over `2**53`.
8. NumPy `shuffle(x)`: for `i` from `len-1` down to 1, `j = random_interval(i)` (mask to the next power of two minus one, reject `> i`), swap.
9. Python `set` iteration order: CPython 3.13 `setobject.c` with `tuplehash` (xxHash64 lanes) and `hash(int)`.

## Edge cases
| Input | Expected |
|---|---|
| seed -7 | `ValueError` from the NumPy seed |
| seed 4294967301 | `ValueError` from the NumPy seed |
| `randrange(0)` | `ValueError` |
