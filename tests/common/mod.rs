//! Helpers shared by the ported mazelib tests.
#![allow(dead_code)]

use mazelib::Grid;

/// Every square on the outer border is a wall.
pub fn boundary_is_solid(grid: &Grid) -> bool {
    let (h, w) = grid.shape();
    (0..w).all(|c| grid.row(0)[c] != 0 && grid.row(h - 1)[c] != 0)
        && (0..h).all(|r| grid.row(r)[0] != 0 && grid.row(r)[w - 1] != 0)
}

/// Every (odd, odd) square is open.
pub fn all_passages_open(grid: &Grid) -> bool {
    let (h, w) = grid.shape();
    (1..h)
        .step_by(2)
        .all(|r| (1..w).step_by(2).all(|c| grid.row(r)[c] != 1))
}

/// Every (even, even) square inside the border is a wall.
pub fn all_corners_complete(grid: &Grid) -> bool {
    let (h, w) = grid.shape();
    (2..h)
        .step_by(2)
        .all(|r| (2..w).step_by(2).all(|c| grid.row(r)[c] != 0))
}

/// Build a grid from rows of 0/1.
pub fn grid_of(rows: &[[u8; 7]]) -> Grid {
    let text: Vec<String> = rows
        .iter()
        .map(|r| r.iter().map(|&v| if v == 1 { '#' } else { '.' }).collect())
        .collect();
    Grid::from_rows(&text).unwrap()
}
