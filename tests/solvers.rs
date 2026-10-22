//! Port of mazelib's `test/test_solvers.py`.

use mazelib::generate::Prims;
use mazelib::solve::*;
use mazelib::{Cell, Maze, Path};

fn one_away(a: Cell, b: Cell) -> bool {
    (a.0 == b.0 && (a.1 - b.1).abs() == 1) || (a.1 == b.1 && (a.0 - b.0).abs() == 1)
}

fn duplicates_in_solution(solution: &Path) -> bool {
    (0..solution.len().saturating_sub(1)).any(|i| solution[i + 1..].contains(&solution[i]))
}

fn solution_is_sane(solution: &Path) -> bool {
    assert!(!solution.is_empty());
    solution.windows(2).all(|w| one_away(w[0], w[1]))
}

fn create_maze_with_varied_entrances(start_outer: bool, end_outer: bool, seed: i64) -> Maze {
    // (sic) the original's default-seed expression
    let seed = if seed != 0 {
        seed
    } else if start_outer {
        4445
    } else if end_outer {
        3
    } else {
        0
    };
    let mut m = Maze::with_seed(seed).unwrap();
    m.generator = Some(Box::new(Prims::new(6, 7).unwrap()));
    m.generate().unwrap();
    m.generate_entrances(start_outer, end_outer).unwrap();
    m
}

fn check_solver(
    seed_start: i64,
    seed_step: i64,
    pass_seed: bool,
    make: fn() -> Box<dyn MazeSolver>,
) {
    let mut seed = seed_start;
    for s in [true, false] {
        for e in [true, false] {
            seed += seed_step;
            let mut m = create_maze_with_varied_entrances(s, e, if pass_seed { seed } else { 0 });
            m.solver = Some(make());
            m.solve().unwrap();
            for sol in m.solutions.as_ref().unwrap() {
                assert!(!duplicates_in_solution(sol));
                assert!(one_away(m.start.unwrap(), sol[0]));
                assert!(one_away(m.end.unwrap(), *sol.last().unwrap()));
                assert!(solution_is_sane(sol));
            }
        }
    }
}

#[test]
fn prune_solution_cases() {
    let (start, end) = ((0, 1), (0, 5));
    let prune = |s: Path| prune_solution(s, start, end);
    let sol: Path = vec![(1, 1), (1, 2), (1, 3), (1, 4), (1, 5)];
    assert_eq!(prune(sol.clone()), sol);

    let sol1 = vec![
        (1, 1),
        (1, 2),
        (1, 3),
        (1, 4),
        (2, 4),
        (3, 4),
        (2, 4),
        (1, 4),
        (1, 5),
    ];
    assert_eq!(prune(sol1), sol);

    let sol2 = vec![
        (1, 1),
        (1, 2),
        (1, 3),
        (1, 4),
        (2, 4),
        (3, 4),
        (2, 4),
        (1, 4),
        (2, 4),
        (3, 4),
        (2, 4),
        (1, 4),
        (1, 5),
    ];
    assert_eq!(prune(sol2), sol);

    let sol3 = vec![(1, 1), (1, 2), (1, 3), (1, 4), (1, 5), (0, 5)];
    assert_eq!(prune(sol3), sol);
    let sol4 = vec![(0, 1), (1, 1), (1, 2), (1, 3), (1, 4), (1, 5)];
    assert_eq!(prune(sol4), sol);
    let sol5 = vec![(0, 1), (1, 1), (1, 2), (1, 3), (1, 4), (1, 5), (0, 5)];
    assert_eq!(prune(sol5), sol);
    let sol6 = vec![
        (0, 1),
        (0, 1),
        (0, 1),
        (1, 1),
        (1, 2),
        (1, 3),
        (1, 4),
        (1, 5),
        (0, 5),
        (0, 5),
    ];
    assert_eq!(prune(sol6), sol);
    let sol7 = vec![
        (0, 1),
        (0, 1),
        (0, 1),
        (1, 1),
        (1, 2),
        (1, 3),
        (1, 4),
        (2, 4),
        (3, 4),
        (2, 4),
        (1, 4),
        (1, 5),
        (0, 5),
        (0, 5),
    ];
    assert_eq!(prune(sol7.clone()), sol);
    assert_eq!(prune(sol7.repeat(100)), sol);

    assert_eq!(prune(vec![]), Vec::<Cell>::new());
    assert_eq!(prune(vec![(1, 1)]), vec![(1, 1)]);
    assert_eq!(prune(vec![(1, 1), (1, 2)]), vec![(1, 1), (1, 2)]);
    assert_eq!(prune([(1, 1), (1, 2)].repeat(100)), vec![(1, 1), (1, 2)]);
}

#[test]
fn backtracking_solver() {
    check_solver(3456, 13, true, || Box::new(BacktrackingSolver::new()));
}

#[test]
fn chain() {
    check_solver(1005, 19, true, || Box::new(Chain::new(Turn::Right)));
}

#[test]
fn collision() {
    check_solver(238, 2, false, || Box::new(Collision::new()));
}

#[test]
fn random_mouse() {
    check_solver(1074, 27, false, || Box::new(RandomMouse::new()));
}

#[test]
fn shortest_path() {
    check_solver(4321, 33, true, || Box::new(ShortestPath::new()));
}

#[test]
fn shortest_paths() {
    check_solver(6109, 11, false, || Box::new(ShortestPaths::new()));
}

#[test]
fn tremaux() {
    check_solver(2121, 21, false, || Box::new(Tremaux::new()));
}

#[test]
fn collision_on_a_maze_with_loops_returns_a_shortest_path() {
    // the original raises TypeError when its robots collide (KNOWN_BUGS.json)
    let grid = mazelib::Grid::from_rows(&[
        "#######", "#.....#", "#.#.#.#", "#.....#", "#.#.#.#", "#.....#", "#######",
    ])
    .unwrap();
    let mut m = Maze::with_seed(0).unwrap();
    m.grid = Some(grid);
    m.start = Some((1, 0));
    m.end = Some((5, 6));
    m.solver = Some(Box::new(Collision::new()));
    m.solve().unwrap();
    let sols = m.solutions.unwrap();
    assert_eq!(sols.len(), 1);
    assert_eq!(sols[0].len(), 9);
    assert!(solution_is_sane(&sols[0]));
}
