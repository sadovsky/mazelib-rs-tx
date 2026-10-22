//! Adapter for the slopfork-mazelib-goldens protocol: JSON-lines cases on
//! stdin, one JSON result line per case on stdout. See the goldens repo's
//! `PROTOCOL.md`.

#![forbid(unsafe_code)]

use mazelib::generate::{
    AldousBroder, BacktrackingGenerator, BinaryTree, CellularAutomaton, Division, DungeonRooms,
    Ellers, GrowingTree, HuntAndKill, HuntOrder, Kruskal, MazeGenerator, MazeType, Prims, Room,
    Sidewinder, Skew, TrivialMaze, Wilsons,
};
use mazelib::solve::{
    BacktrackingSolver, Chain, Collision, MazeSolver, RandomMouse, ShortestPath, ShortestPaths,
    Tremaux, Turn,
};
use mazelib::transmute::{CuldeSacFiller, DeadEndFiller, MazeTransmuter, Perturbation};
use mazelib::{Cell, Grid, Maze, MazeError, Path, Rng};
use serde_json::{json, Map, Value};
use std::io::{self, BufRead, Write};

/// Errors the adapter reports: library errors keep their Python kind;
/// malformed cases are reported as `ValueError`.
struct Failure {
    kind: String,
    message: String,
}

impl From<MazeError> for Failure {
    fn from(e: MazeError) -> Self {
        Failure {
            kind: e.python_kind().to_string(),
            message: e.message().to_string(),
        }
    }
}

fn bad(message: impl Into<String>) -> Failure {
    Failure {
        kind: "ValueError".into(),
        message: message.into(),
    }
}

type Res<T> = Result<T, Failure>;

fn empty_map() -> &'static Map<String, Value> {
    static EMPTY: std::sync::OnceLock<Map<String, Value>> = std::sync::OnceLock::new();
    EMPTY.get_or_init(Map::new)
}

/// The `params` object of a spec (missing or null means empty).
fn params(spec: &Value) -> &Map<String, Value> {
    spec.get("params")
        .and_then(Value::as_object)
        .unwrap_or_else(|| empty_map())
}

fn int_of(v: &Value, what: &str) -> Res<i64> {
    v.as_i64()
        .ok_or_else(|| bad(format!("{what} must be an integer")))
}

fn float_param(p: &Map<String, Value>, key: &str, default: f64) -> Res<f64> {
    match p.get(key) {
        None | Some(Value::Null) => Ok(default),
        Some(v) => v
            .as_f64()
            .ok_or_else(|| bad(format!("{key} must be a number"))),
    }
}

fn int_param(p: &Map<String, Value>, key: &str, default: i64) -> Res<i64> {
    match p.get(key) {
        None | Some(Value::Null) => Ok(default),
        Some(v) => v
            .as_i64()
            .or_else(|| v.as_f64().map(|f| f.trunc() as i64))
            .ok_or_else(|| bad(format!("{key} must be a number"))),
    }
}

fn str_param<'a>(p: &'a Map<String, Value>, key: &str) -> Option<&'a str> {
    p.get(key).and_then(Value::as_str)
}

fn cell_in(v: &Value) -> Res<Option<Cell>> {
    match v {
        Value::Null => Ok(None),
        Value::Array(a) if a.len() == 2 => {
            Ok(Some((int_of(&a[0], "cell")?, int_of(&a[1], "cell")?)))
        }
        _ => Err(bad("a cell must be [row, col]")),
    }
}

fn cell_out(c: Option<Cell>) -> Value {
    match c {
        Some((r, c)) => json!([r, c]),
        None => Value::Null,
    }
}

fn grid_in(v: &Value) -> Res<Grid> {
    let rows: Vec<&str> = v
        .as_array()
        .ok_or_else(|| bad("grid must be a list of strings"))?
        .iter()
        .map(|r| r.as_str().ok_or_else(|| bad("grid rows must be strings")))
        .collect::<Res<_>>()?;
    Ok(Grid::from_rows(&rows)?)
}

fn grid_out(g: &Grid) -> Value {
    json!(g.to_rows())
}

fn solutions_in(v: &Value) -> Res<Option<Vec<Path>>> {
    match v {
        Value::Null => Ok(None),
        Value::Array(paths) => paths
            .iter()
            .map(|p| {
                p.as_array()
                    .ok_or_else(|| bad("a path must be a list of cells"))?
                    .iter()
                    .map(|c| cell_in(c)?.ok_or_else(|| bad("path cells cannot be null")))
                    .collect()
            })
            .collect::<Res<Vec<Path>>>()
            .map(Some),
        _ => Err(bad("solutions must be a list of paths")),
    }
}

fn solutions_out(s: &Option<Vec<Path>>) -> Value {
    match s {
        None => Value::Null,
        Some(paths) => Value::Array(
            paths
                .iter()
                .map(|p| Value::Array(p.iter().map(|&c| cell_out(Some(c))).collect()))
                .collect(),
        ),
    }
}

fn rooms_in(v: &Value) -> Res<Option<Vec<Room>>> {
    let Some(list) = v.as_array() else {
        return Ok(None);
    };
    // malformed rooms are skipped, as the original ignores rooms it can't carve
    let rooms = list
        .iter()
        .filter_map(|room| {
            let corners = room.as_array()?;
            if corners.len() != 2 {
                return None;
            }
            let a = cell_in(&corners[0]).ok()??;
            let b = cell_in(&corners[1]).ok()??;
            Some((a, b))
        })
        .collect();
    Ok(Some(rooms))
}

fn make_generator(spec: &Value, rng: &mut Rng) -> Res<Box<dyn MazeGenerator>> {
    let algo = spec.get("algo").and_then(Value::as_str).unwrap_or("");
    let h = int_of(spec.get("h").unwrap_or(&Value::Null), "h")?;
    let w = int_of(spec.get("w").unwrap_or(&Value::Null), "w")?;
    let p = params(spec);
    let hunt = || HuntOrder::parse(str_param(p, "hunt_order").unwrap_or("random"));
    Ok(match algo {
        "AldousBroder" => Box::new(AldousBroder::new(h, w)?),
        "BacktrackingGenerator" => Box::new(BacktrackingGenerator::new(h, w)?),
        "BinaryTree" => {
            let skew = str_param(p, "skew").and_then(Skew::parse);
            Box::new(BinaryTree::new(h, w, skew, rng)?)
        }
        "CellularAutomaton" => Box::new(CellularAutomaton::new(
            h,
            w,
            float_param(p, "complexity", 1.0)?,
            float_param(p, "density", 1.0)?,
        )?),
        "Division" => Box::new(Division::new(h, w)?),
        "DungeonRooms" => {
            let rooms = rooms_in(p.get("rooms").unwrap_or(&Value::Null))?;
            let grid = match p.get("grid") {
                None | Some(Value::Null) => None,
                Some(g) => Some(grid_in(g)?),
            };
            Box::new(DungeonRooms::new(h, w, rooms, grid, hunt())?)
        }
        "Ellers" => Box::new(Ellers::new(
            h,
            w,
            float_param(p, "xskew", 0.5)?,
            float_param(p, "yskew", 0.5)?,
        )?),
        "GrowingTree" => Box::new(GrowingTree::new(
            h,
            w,
            float_param(p, "backtrack_chance", 1.0)?,
        )?),
        "HuntAndKill" => Box::new(HuntAndKill::new(h, w, hunt())?),
        "Kruskal" => Box::new(Kruskal::new(h, w)?),
        "Prims" => Box::new(Prims::new(h, w)?),
        "Sidewinder" => Box::new(Sidewinder::new(h, w, float_param(p, "skew", 0.5)?)?),
        "TrivialMaze" => Box::new(TrivialMaze::new(
            h,
            w,
            MazeType::parse(str_param(p, "maze_type").unwrap_or("spiral")),
        )?),
        "Wilsons" => Box::new(Wilsons::new(h, w, hunt())?),
        other => return Err(bad(format!("unknown generator {other:?}"))),
    })
}

fn make_solver(spec: &Value) -> Res<Box<dyn MazeSolver>> {
    let algo = spec.get("algo").and_then(Value::as_str).unwrap_or("");
    let p = params(spec);
    Ok(match algo {
        "BacktrackingSolver" => Box::new(BacktrackingSolver::new()),
        "Chain" => Box::new(Chain::new(Turn::parse(
            str_param(p, "turn").unwrap_or("right"),
        ))),
        "Collision" => Box::new(Collision::new()),
        "RandomMouse" => Box::new(RandomMouse::new()),
        "ShortestPath" => Box::new(ShortestPath::new()),
        "ShortestPaths" => Box::new(ShortestPaths::new()),
        "Tremaux" => Box::new(Tremaux::new()),
        other => return Err(bad(format!("unknown solver {other:?}"))),
    })
}

fn make_transmuter(spec: &Value) -> Res<Box<dyn MazeTransmuter>> {
    let algo = spec.get("algo").and_then(Value::as_str).unwrap_or("");
    let p = params(spec);
    Ok(match algo {
        "CuldeSacFiller" => Box::new(CuldeSacFiller::new()),
        "DeadEndFiller" => Box::new(DeadEndFiller::new(int_param(p, "iterations", 1)?)),
        "Perturbation" => Box::new(Perturbation::new(
            int_param(p, "repeat", 1)?,
            int_param(p, "new_walls", 1)?,
        )),
        other => return Err(bad(format!("unknown transmuter {other:?}"))),
    })
}

fn arg<'a>(a: &'a Value, key: &str) -> &'a Value {
    a.get(key).unwrap_or(&Value::Null)
}

/// A maze holding the case's grid and entrances.
fn maze_with(m: &mut Maze, a: &Value) -> Res<()> {
    m.grid = Some(grid_in(arg(a, "grid"))?);
    m.start = cell_in(arg(a, "start"))?;
    m.end = cell_in(arg(a, "end"))?;
    Ok(())
}

fn run_op(op: &str, a: &Value, m: &mut Maze) -> Res<Value> {
    match op {
        "generate" => {
            let mut generator = make_generator(a, &mut m.rng)?;
            let grid = generator.generate(&mut m.rng)?;
            Ok(json!({ "grid": grid_out(&grid) }))
        }
        "entrances" => {
            m.grid = Some(grid_in(arg(a, "grid"))?);
            let flag = |k: &str| {
                arg(a, k)
                    .as_bool()
                    .ok_or_else(|| bad(format!("{k} must be a bool")))
            };
            m.generate_entrances(flag("start_outer")?, flag("end_outer")?)?;
            Ok(json!({ "start": cell_out(m.start), "end": cell_out(m.end) }))
        }
        "solve" => {
            maze_with(m, a)?;
            m.solver = Some(make_solver(a)?);
            m.prune = arg(a, "prune").as_bool().unwrap_or(true);
            m.solve()?;
            Ok(json!({ "solutions": solutions_out(&m.solutions) }))
        }
        "transmute" => {
            maze_with(m, a)?;
            m.transmuters = vec![make_transmuter(a)?];
            m.transmute()?;
            Ok(json!({ "grid": grid_out(m.grid.as_ref().expect("set above")) }))
        }
        "tostring" => {
            maze_with(m, a)?;
            m.solutions = solutions_in(arg(a, "solutions"))?;
            let flag = |k: &str| arg(a, k).as_bool().unwrap_or(false);
            Ok(json!({ "text": m.tostring(flag("entrances"), flag("show_solutions"))? }))
        }
        "monte_carlo" => {
            m.generator = Some(make_generator(arg(a, "generator"), &mut m.rng)?);
            m.solver = Some(make_solver(arg(a, "solver"))?);
            let count = |k: &str| -> Res<usize> {
                let n = int_of(arg(a, k), k)?;
                Ok(usize::try_from(n).unwrap_or(0))
            };
            let difficulty = arg(a, "difficulty")
                .as_f64()
                .ok_or_else(|| bad("difficulty must be a number"))?;
            m.generate_monte_carlo(count("repeat")?, count("entrances")?, difficulty)?;
            Ok(json!({
                "grid": grid_out(m.grid.as_ref().expect("generated")),
                "start": cell_out(m.start),
                "end": cell_out(m.end),
                "solutions": solutions_out(&m.solutions),
            }))
        }
        "rng" => run_rng(a, &mut m.rng),
        other => Err(bad(format!("unknown op {other:?}"))),
    }
}

fn run_rng(a: &Value, rng: &mut Rng) -> Res<Value> {
    let calls = arg(a, "calls")
        .as_array()
        .ok_or_else(|| bad("calls must be a list"))?;
    let mut out = Vec::with_capacity(calls.len());
    for call in calls {
        let call = call
            .as_array()
            .ok_or_else(|| bad("a call must be a list"))?;
        let name = call.first().and_then(Value::as_str).unwrap_or("");
        let ints: Vec<i64> = call[1..]
            .iter()
            .map(|v| int_of(v, "rng argument"))
            .collect::<Res<_>>()?;
        let n = |i: usize| -> Res<i64> {
            ints.get(i)
                .copied()
                .ok_or_else(|| bad("missing rng argument"))
        };
        let len =
            |i: usize| -> Res<usize> { usize::try_from(n(i)?).map_err(|_| bad("negative length")) };
        let value = match name {
            "random" => json!(rng.py.random()),
            "randrange" => json!(match ints.len() {
                1 => rng.py.randrange_to(ints[0])?,
                2 => rng.py.randrange(ints[0], ints[1], 1)?,
                3 => rng.py.randrange(ints[0], ints[1], ints[2])?,
                _ => return Err(bad("randrange takes 1 to 3 arguments")),
            }),
            "randint" => json!(rng.py.randint(n(0)?, n(1)?)?),
            "choice" => json!(rng.py.choice_index(len(0)?)?),
            "shuffle" => {
                let mut x: Vec<usize> = (0..len(0)?).collect();
                rng.py.shuffle(&mut x);
                json!(x)
            }
            "np_shuffle" => {
                let mut x: Vec<usize> = (0..len(0)?).collect();
                rng.np.shuffle(&mut x);
                json!(x)
            }
            other => return Err(bad(format!("unknown rng call {other}"))),
        };
        out.push(value);
    }
    Ok(json!({ "out": out }))
}

fn run_case(case: &Value) -> Value {
    let result = (|| -> Res<Value> {
        let mut maze = Maze::new();
        match case.get("seed") {
            None | Some(Value::Null) => {}
            Some(s) => maze.set_seed(int_of(s, "seed")?)?,
        }
        let op = case.get("op").and_then(Value::as_str).unwrap_or("");
        run_op(op, arg(case, "args"), &mut maze)
    })();
    match result {
        Ok(value) => json!({ "ok": true, "value": value }),
        Err(f) => json!({ "ok": false, "error": { "kind": f.kind, "message": f.message } }),
    }
}

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }
        let result = match serde_json::from_str::<Value>(&line) {
            Ok(case) => std::panic::catch_unwind(|| run_case(&case)).unwrap_or_else(|_| {
                json!({ "ok": false, "error": { "kind": "Panic", "message": "adapter panicked" } })
            }),
            Err(e) => json!({ "ok": false, "error": { "kind": "ValueError", "message": e.to_string() } }),
        };
        if writeln!(out, "{result}")
            .and_then(|()| out.flush())
            .is_err()
        {
            break;
        }
    }
}
