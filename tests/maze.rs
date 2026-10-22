//! Port of mazelib's `test/test_maze.py`.

use mazelib::generate::Prims;
use mazelib::solve::Collision;
use mazelib::{Cell, Grid, Maze, MazeError, Path};

fn on_edge(grid: &Grid, (r, c): Cell) -> bool {
    r == 0 || r == grid.height() as i64 - 1 || c == 0 || c == grid.width() as i64 - 1
}

fn num_turns(path: &Path) -> f64 {
    if path.len() < 3 {
        return 0.0;
    }
    (1..path.len() - 1)
        .filter(|&i| {
            let same_col = path[i - 1].0 == path[i].0 && path[i].0 == path[i + 1].0;
            let same_row = path[i - 1].1 == path[i].1 && path[i].1 == path[i + 1].1;
            !same_row && !same_col
        })
        .count() as f64
}

fn prims_maze(seed: i64, h: i64, w: i64) -> Maze {
    let mut m = Maze::with_seed(seed).unwrap();
    m.generator = Some(Box::new(Prims::new(h, w).unwrap()));
    m
}

#[test]
fn grid_size() {
    let mut m = prims_maze(1337, 4, 5);
    m.generate().unwrap();
    assert_eq!(m.grid.unwrap().shape(), (9, 11));
}

#[test]
fn inner_entrances() {
    for i in 0..10 {
        let mut m = prims_maze(999 + i, 4, 5);
        m.generate().unwrap();
        m.generate_entrances(false, false).unwrap();
        let g = m.grid.as_ref().unwrap();
        assert!(!on_edge(g, m.start.unwrap()));
        assert!(!on_edge(g, m.end.unwrap()));
    }
}

#[test]
fn outer_entrances() {
    for _ in 0..10 {
        let mut m = prims_maze(1201, 3, 3);
        m.generate().unwrap();
        m.generate_entrances(true, true).unwrap();
        let g = m.grid.as_ref().unwrap();
        assert!(on_edge(g, m.start.unwrap()));
        assert!(on_edge(g, m.end.unwrap()));
    }
}

#[test]
fn generator_wipe() {
    for _ in 0..10 {
        let mut m = prims_maze(1034, 4, 5);
        m.generate().unwrap();
        m.generate_entrances(true, true).unwrap();
        m.generate().unwrap();
        assert!(m.start.is_none() && m.end.is_none() && m.solutions.is_none());
    }
}

#[test]
fn monte_carlo() {
    let mut m = prims_maze(104, 4, 5);
    m.solver = Some(Box::new(Collision::new()));
    m.generate_monte_carlo(3, 3, 1.0).unwrap();
    let g = m.grid.as_ref().unwrap();
    assert_eq!(g.shape(), (9, 11));
    assert!(on_edge(g, m.start.unwrap()));
    assert!(on_edge(g, m.end.unwrap()));
}

#[test]
fn monte_carlo_reducer() {
    let mut m = prims_maze(105, 4, 5);
    m.solver = Some(Box::new(Collision::new()));
    m.generate_monte_carlo_with(3, 3, 1.0, num_turns).unwrap();
    let g = m.grid.as_ref().unwrap();
    assert_eq!(g.shape(), (9, 11));
    assert!(on_edge(g, m.start.unwrap()));
    assert!(on_edge(g, m.end.unwrap()));
}

#[test]
fn maze_to_string() {
    let mut m = prims_maze(106, 3, 3);
    m.grid = Some(
        Grid::from_rows(&[
            "#######", "#.....#", "#.#.###", "#.#...#", "###.###", "#.....#", "#######",
        ])
        .unwrap(),
    );
    m.start = Some((5, 0));
    m.end = Some((3, 6));
    m.solutions = Some(vec![vec![
        (5, 1),
        (5, 2),
        (5, 3),
        (5, 4),
        (5, 5),
        (4, 5),
        (3, 5),
    ]]);

    let text = m.to_string();
    let s: Vec<&str> = text.split('\n').collect();
    assert_eq!(s[0].trim(), "#######");
    assert_eq!(s[2].trim(), "# # ###");
    assert_eq!(s[3].trim(), "# #  +E");
    assert_eq!(s[5].trim(), "S+++++#");
    assert_eq!(s[6].trim(), "#######");
}

#[test]
fn invalid_inputs() {
    let mut m = Maze::with_seed(107).unwrap();
    assert!(matches!(m.generate(), Err(MazeError::Assertion(_))));
    assert!(matches!(m.solve(), Err(MazeError::Assertion(_))));

    m.generator = Some(Box::new(Prims::new(3, 3).unwrap()));
    assert!(matches!(m.solve(), Err(MazeError::Assertion(_))));

    m.start = Some((1, 1));
    m.end = Some((3, 3));
    assert_eq!(m.to_string(), "");

    for difficulty in [-1.0, 10.0] {
        assert!(matches!(
            m.generate_monte_carlo(1, 3, difficulty),
            Err(MazeError::Assertion(_))
        ));
    }
}

#[test]
fn set_seed() {
    let mut m = prims_maze(9090, 7, 7);
    m.generate().unwrap();
    let grid0 = m.to_string();

    let mut m = prims_maze(9090, 7, 7);
    m.generate().unwrap();
    let grid1 = m.to_string();
    assert_eq!(grid0, grid1);

    m.generator = Some(Box::new(Prims::new(7, 7).unwrap()));
    m.generate().unwrap();
    assert_ne!(grid0, m.to_string());
}

#[test]
fn seed_out_of_numpy_range() {
    assert!(matches!(Maze::with_seed(-7), Err(MazeError::Value(_))));
    assert!(matches!(Maze::with_seed(1 << 32), Err(MazeError::Value(_))));
}
