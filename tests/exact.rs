//! Exact outputs recorded from the Python original (seed 2024, 4x6 mazes).

use mazelib::generate::*;
use mazelib::solve::ShortestPaths;
use mazelib::{Grid, Maze, Rng};

fn check(make: impl FnOnce(&mut Rng) -> Box<dyn MazeGenerator>, expected: &[&str]) {
    let mut rng = Rng::seeded(2024).unwrap();
    let mut g = make(&mut rng);
    assert_eq!(
        g.generate(&mut rng).unwrap(),
        Grid::from_rows(expected).unwrap()
    );
}

#[test]
fn aldous_broder() {
    check(
        |_rng| Box::new(AldousBroder::new(4, 6).unwrap()),
        &[
            "#############",
            "#.#.........#",
            "#.#.#######.#",
            "#.....#...#.#",
            "#######.#.#.#",
            "#.....#.#...#",
            "#.###.#.#####",
            "#...#.......#",
            "#############",
        ],
    );
}

#[test]
fn backtracking_generator() {
    check(
        |_rng| Box::new(BacktrackingGenerator::new(4, 6).unwrap()),
        &[
            "#############",
            "#.#.........#",
            "#.#.#######.#",
            "#.....#...#.#",
            "#######.#.#.#",
            "#.....#.#...#",
            "#.###.#.###.#",
            "#...#...#...#",
            "#############",
        ],
    );
}

#[test]
fn binary_tree() {
    check(
        |rng| Box::new(BinaryTree::new(4, 6, None, rng).unwrap()),
        &[
            "#############",
            "#...........#",
            "#.#######.#.#",
            "#.#.......#.#",
            "###.#.###.#.#",
            "#...#.#...#.#",
            "###.#######.#",
            "#...#.......#",
            "#############",
        ],
    );
}

#[test]
fn cellular_automaton() {
    check(
        |_rng| Box::new(CellularAutomaton::new(4, 6, 1.0, 1.0).unwrap()),
        &[
            "#############",
            "#.........#.#",
            "#.###.#####.#",
            "#.#...#.....#",
            "#.#####.###.#",
            "#.......#.#.#",
            "#.###.###.#.#",
            "#...#.#.....#",
            "#############",
        ],
    );
}

#[test]
fn division() {
    check(
        |_rng| Box::new(Division::new(4, 6).unwrap()),
        &[
            "#############",
            "#.#.#.#.#...#",
            "#.#.#.#.###.#",
            "#...........#",
            "###.#######.#",
            "#.#.#.#.#...#",
            "#.#.#.#.###.#",
            "#.......#...#",
            "#############",
        ],
    );
}

#[test]
fn dungeon_rooms() {
    check(
        |_rng| Box::new(DungeonRooms::new(4, 6, None, None, HuntOrder::Random).unwrap()),
        &[
            "#############",
            "#...#.......#",
            "#.#.#.#######",
            "#.#.#...#...#",
            "#.#.###.#.#.#",
            "#.#.#...#.#.#",
            "#.###.#.#.#.#",
            "#.....#...#.#",
            "#############",
        ],
    );
}

#[test]
fn ellers() {
    check(
        |_rng| Box::new(Ellers::new(4, 6, 0.5, 0.5).unwrap()),
        &[
            "#############",
            "#...#...#...#",
            "#.#.#.#.###.#",
            "#.#...#.#.#.#",
            "#######.#.#.#",
            "#.#.......#.#",
            "#.#.###.#.#.#",
            "#.....#.#...#",
            "#############",
        ],
    );
}

#[test]
fn growing_tree() {
    check(
        |_rng| Box::new(GrowingTree::new(4, 6, 1.0).unwrap()),
        &[
            "#############",
            "#.......#...#",
            "#.###.#.###.#",
            "#.#...#.#...#",
            "#.#.###.#.#.#",
            "#.#...#.#.#.#",
            "#.###.#.#.#.#",
            "#.#...#...#.#",
            "#############",
        ],
    );
}

#[test]
fn hunt_and_kill() {
    check(
        |_rng| Box::new(HuntAndKill::new(4, 6, HuntOrder::Random).unwrap()),
        &[
            "#############",
            "#.......#...#",
            "#.###.#.#.#.#",
            "#.#...#.#.#.#",
            "#.#.###.#.#.#",
            "#.#.#.#...#.#",
            "#.#.#.#####.#",
            "#.#.#.......#",
            "#############",
        ],
    );
}

#[test]
fn kruskal() {
    check(
        |_rng| Box::new(Kruskal::new(4, 6).unwrap()),
        &[
            "#############",
            "#...........#",
            "#######.#.###",
            "#.#.....#.#.#",
            "#.#####.###.#",
            "#.#.......#.#",
            "#.#.#####.#.#",
            "#.....#.....#",
            "#############",
        ],
    );
}

#[test]
fn prims() {
    check(
        |_rng| Box::new(Prims::new(4, 6).unwrap()),
        &[
            "#############",
            "#.......#...#",
            "###.#######.#",
            "#.....#.#.#.#",
            "###.###.#.#.#",
            "#.#.#...#...#",
            "#.#.###.###.#",
            "#...........#",
            "#############",
        ],
    );
}

#[test]
fn sidewinder() {
    check(
        |_rng| Box::new(Sidewinder::new(4, 6, 0.5).unwrap()),
        &[
            "#############",
            "#...........#",
            "#.#.#######.#",
            "#.#.#.......#",
            "#.#.#######.#",
            "#.#.#.......#",
            "#####.#######",
            "#...........#",
            "#############",
        ],
    );
}

#[test]
fn trivial_maze() {
    check(
        |_rng| Box::new(TrivialMaze::new(4, 6, MazeType::Spiral).unwrap()),
        &[
            "#############",
            "#.#.........#",
            "#.#.#######.#",
            "#.#.#.....#.#",
            "#.#.#####.#.#",
            "#.#.......#.#",
            "#.#########.#",
            "#...........#",
            "#############",
        ],
    );
}

#[test]
fn wilsons() {
    check(
        |_rng| Box::new(Wilsons::new(4, 6, HuntOrder::Random).unwrap()),
        &[
            "#############",
            "#.....#.#...#",
            "###.###.###.#",
            "#...#.......#",
            "#.#.###.#.###",
            "#.#...#.#.#.#",
            "#.#.###.###.#",
            "#.#.........#",
            "#############",
        ],
    );
}

#[test]
fn prims_entrances_and_shortest_paths() {
    let mut m = Maze::with_seed(77).unwrap();
    m.generator = Some(Box::new(Prims::new(5, 5).unwrap()));
    m.generate().unwrap();
    m.generate_entrances(true, true).unwrap();
    m.solver = Some(Box::new(ShortestPaths::new()));
    m.solve().unwrap();
    assert_eq!(
        m.grid.unwrap(),
        Grid::from_rows(&[
            "###########",
            "#...#.....#",
            "###.###.###",
            "#...#.#.#.#",
            "###.#.#.#.#",
            "#.......#.#",
            "###.###.#.#",
            "#.....#...#",
            "###.###.###",
            "#...#.....#",
            "###########"
        ])
        .unwrap()
    );
    assert_eq!((m.start, m.end), (Some((0, 1)), Some((10, 9))));
    assert_eq!(
        m.solutions.unwrap(),
        vec![vec![
            (1, 1),
            (1, 2),
            (1, 3),
            (2, 3),
            (3, 3),
            (4, 3),
            (5, 3),
            (5, 4),
            (5, 5),
            (5, 6),
            (5, 7),
            (6, 7),
            (7, 7),
            (8, 7),
            (9, 7),
            (9, 8),
            (9, 9)
        ]]
    );
}
