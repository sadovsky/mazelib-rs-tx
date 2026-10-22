//! Port of mazelib's `test/test_generators.py`.

mod common;

use common::*;
use mazelib::generate::*;
use mazelib::{Grid, Maze, MazeError, Rng};

fn generated(seed: i64, make: impl FnOnce(&mut Rng) -> Box<dyn MazeGenerator>) -> Grid {
    let mut m = Maze::with_seed(seed).unwrap();
    m.generator = Some(make(&mut m.rng));
    m.generate().unwrap();
    m.grid.unwrap()
}

fn assert_sane(grid: &Grid, corners: bool) {
    assert!(boundary_is_solid(grid));
    assert!(all_passages_open(grid));
    if corners {
        assert!(all_corners_complete(grid));
    }
}

#[test]
fn abstract_constructor() {
    let m = Maze::with_seed(723).unwrap();
    let mut g = Prims::new(3, 3).unwrap();
    let mut rng = m.rng.clone();
    assert_eq!(g.generate(&mut rng).unwrap().shape(), (7, 7));
    let mut g = Prims::new(24, 12).unwrap();
    assert_eq!(g.generate(&mut rng).unwrap().shape(), (49, 25));

    for (h, w) in [(2, 2), (0, 2), (-2, 3), (224, -2)] {
        assert!(matches!(Prims::new(h, w), Err(MazeError::Assertion(_))));
    }
}

#[test]
fn aldous_broder() {
    let g = generated(143, |_| Box::new(AldousBroder::new(4, 5).unwrap()));
    assert_sane(&g, true);
}

#[test]
fn backtracking_generator() {
    let g = generated(23, |_| Box::new(BacktrackingGenerator::new(4, 5).unwrap()));
    assert_sane(&g, true);
}

#[test]
fn binary_tree() {
    let g = generated(163, |rng| {
        Box::new(BinaryTree::new(4, 5, None, rng).unwrap())
    });
    assert_sane(&g, true);
    let g = generated(1298, |rng| {
        Box::new(BinaryTree::new(4, 5, Skew::parse("NW"), rng).unwrap())
    });
    assert_sane(&g, true);
}

#[test]
fn cellular_automaton() {
    let g = generated(183, |_| {
        Box::new(CellularAutomaton::new(4, 5, 1.0, 1.0).unwrap())
    });
    assert_sane(&g, false);
}

#[test]
fn division() {
    let g = generated(653, |_| Box::new(Division::new(4, 5).unwrap()));
    assert_sane(&g, true);
}

#[test]
fn dungeon_rooms_grid() {
    let g = grid_of(&[
        [1, 1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 1],
        [1, 1, 0, 0, 0, 1, 1],
        [1, 1, 0, 0, 0, 1, 1],
        [1, 1, 0, 0, 0, 1, 1],
        [1, 1, 1, 1, 1, 1, 1],
    ]);
    let out = generated(1293, |_| {
        Box::new(DungeonRooms::new(4, 4, None, Some(g), HuntOrder::Random).unwrap())
    });
    assert_sane(&out, false);
}

#[test]
fn dungeon_reconnect_maze() {
    let g = grid_of(&[
        [1, 1, 1, 1, 1, 1, 1],
        [1, 0, 0, 0, 1, 0, 1],
        [1, 0, 1, 1, 1, 0, 1],
        [1, 0, 0, 0, 1, 0, 1],
        [1, 0, 0, 0, 1, 0, 1],
        [1, 0, 0, 0, 1, 0, 1],
        [1, 1, 1, 1, 1, 1, 1],
    ]);
    let mut m = Maze::with_seed(865).unwrap();
    let mut gen = DungeonRooms::new(4, 4, None, Some(g), HuntOrder::Random).unwrap();
    gen.reconnect_maze(&mut m.rng).unwrap();
    assert_sane(gen.grid(), false);
}

#[test]
fn dungeon_rooms_random_rooms() {
    for i in 0..20 {
        let g = generated(900 + i, |_| {
            Box::new(
                DungeonRooms::new(
                    4,
                    4,
                    Some(vec![((1, 1), (3, 3))]),
                    None,
                    HuntOrder::parse("random"),
                )
                .unwrap(),
            )
        });
        assert_sane(&g, false);
    }
}

#[test]
fn dungeon_rooms_serpentine_rooms() {
    let g = generated(443, |_| {
        Box::new(
            DungeonRooms::new(
                4,
                4,
                Some(vec![((1, 1), (3, 3))]),
                None,
                HuntOrder::parse("serpentine"),
            )
            .unwrap(),
        )
    });
    assert_sane(&g, false);
}

#[test]
fn ellers() {
    let g = generated(563, |_| Box::new(Ellers::new(4, 5, 0.5, 0.5).unwrap()));
    assert_sane(&g, true);
}

#[test]
fn ellers_more_than_127_sets() {
    // the original overflows its int8 set ids here (KNOWN_BUGS.json)
    for seed in 0..8 {
        let g = generated(seed, |_| Box::new(Ellers::new(12, 17, 0.9, 0.1).unwrap()));
        assert_sane(&g, true);
    }
}

#[test]
fn growing_tree() {
    let g = generated(743, |_| Box::new(GrowingTree::new(4, 5, 1.0).unwrap()));
    assert_sane(&g, true);
}

#[test]
fn hunt_and_kill_random_order() {
    let g = generated(8888, |_| {
        Box::new(HuntAndKill::new(4, 5, HuntOrder::parse("random")).unwrap())
    });
    assert_sane(&g, true);
}

#[test]
fn hunt_and_kill_serpentine_order() {
    let g = generated(808, |_| {
        Box::new(HuntAndKill::new(4, 5, HuntOrder::parse("serpentine")).unwrap())
    });
    assert_sane(&g, true);
}

#[test]
fn kruskal() {
    let g = generated(314, |_| Box::new(Kruskal::new(4, 5).unwrap()));
    assert_sane(&g, true);
}

#[test]
fn prims() {
    let g = generated(159, |_| Box::new(Prims::new(4, 5).unwrap()));
    assert_sane(&g, true);
}

#[test]
fn sidewinder() {
    let g = generated(26, |_| Box::new(Sidewinder::new(4, 5, 0.5).unwrap()));
    assert_sane(&g, true);
}

#[test]
fn trivial_maze_spiral() {
    let g = generated(535, |_| {
        Box::new(TrivialMaze::new(4, 5, MazeType::Spiral).unwrap())
    });
    assert_sane(&g, true);
}

#[test]
fn trivial_maze_serpentine() {
    for i in 0..20 {
        let g = generated(1967 + i, |_| {
            Box::new(TrivialMaze::new(4, 5, MazeType::parse("serpentine")).unwrap())
        });
        assert_sane(&g, true);
    }
}

#[test]
fn wilsons_random_order() {
    let g = generated(432, |_| {
        Box::new(Wilsons::new(4, 5, HuntOrder::Random).unwrap())
    });
    assert_sane(&g, true);
}

#[test]
fn wilsons_serpentine_order() {
    let g = generated(12345, |_| {
        Box::new(Wilsons::new(4, 5, HuntOrder::Serpentine).unwrap())
    });
    assert_sane(&g, true);
}
