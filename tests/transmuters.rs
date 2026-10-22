//! Port of mazelib's `test/test_transmuters.py`.

mod common;

use common::*;
use mazelib::generate::{MazeType, Prims, TrivialMaze};
use mazelib::transmute::{CuldeSacFiller, DeadEndFiller, Perturbation};
use mazelib::{Grid, Maze};

fn example_cul_de_sac_maze() -> Grid {
    grid_of(&[
        [1, 1, 1, 1, 1, 1, 1],
        [1, 0, 0, 0, 0, 0, 1],
        [1, 0, 1, 0, 1, 0, 1],
        [1, 0, 1, 0, 0, 0, 1],
        [1, 0, 1, 1, 1, 1, 1],
        [1, 0, 0, 0, 0, 0, 1],
        [1, 1, 1, 1, 1, 1, 1],
    ])
}

#[test]
fn cul_de_sac_filler() {
    let mut m = Maze::with_seed(8765).unwrap();
    m.generator = Some(Box::new(Prims::new(3, 3).unwrap()));
    m.generate().unwrap();
    m.grid = Some(example_cul_de_sac_maze());
    assert_eq!(m.grid.as_ref().unwrap().at((1, 5)), 0);

    m.transmuters = vec![Box::new(CuldeSacFiller::new())];
    m.transmute().unwrap();

    let g = m.grid.as_ref().unwrap();
    assert_eq!(g.at((1, 5)), 1);
    assert!(boundary_is_solid(g));
    assert!(all_corners_complete(g));
}

#[test]
fn dead_end_filler() {
    for i in 0..10 {
        let mut m = Maze::with_seed(678 + i).unwrap();
        m.generator = Some(Box::new(Prims::new(3, 3).unwrap()));
        m.generate().unwrap();
        m.start = Some((1, 0));
        m.end = Some((5, 4));
        m.grid = Some(example_cul_de_sac_maze());
        {
            let g = m.grid.as_ref().unwrap();
            assert_eq!((g.at((1, 5)), g.at((1, 2)), g.at((3, 3))), (0, 0, 0));
        }

        m.transmuters = vec![
            Box::new(CuldeSacFiller::new()),
            Box::new(DeadEndFiller::new(99)),
        ];
        m.transmute().unwrap();

        let g = m.grid.as_ref().unwrap();
        assert_eq!((g.at((1, 5)), g.at((1, 2)), g.at((3, 3))), (1, 1, 1));
        assert!(boundary_is_solid(g));
        assert!(all_corners_complete(g));
    }
}

#[test]
fn perturbation() {
    for i in 0..10 {
        let mut m = Maze::with_seed(9087 + i).unwrap();
        m.generator = Some(Box::new(TrivialMaze::new(4, 5, MazeType::Spiral).unwrap()));
        m.generate().unwrap();
        m.transmuters = vec![Box::new(Perturbation::default())];
        m.transmute().unwrap();

        let g = m.grid.as_ref().unwrap();
        assert!(boundary_is_solid(g));
        assert!(all_passages_open(g));
        assert!(all_corners_complete(g));
    }
}
