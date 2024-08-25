use crate::minefield::{CellState, Minefield};
use nalgebra::{DMatrix, DMatrixView, DVector, DVectorView};
use std::{
    iter::{once, repeat},
    ops::Range,
};

fn get_hidden_cells(minefield: &Minefield, all: bool) -> Vec<usize> {
    (0..(minefield.height * minefield.width))
        .filter(|idx| minefield.cells[*idx].state == CellState::Hidden)
        .filter(|idx| {
            all || minefield.neighboring_open(*idx % minefield.width, *idx / minefield.width)
        })
        .collect()
}

fn create_system(
    minefield: &Minefield,
    hidden_cells: &[usize],
    include_total_mines: bool,
) -> (DMatrix<u8>, DVector<u8>) {
    let mf_height = minefield.height;
    let mf_width = minefield.width;

    let mut matrix_height = 0;
    let mut x_inner = Vec::new();

    let a_inner_iter = (0..mf_height)
        .flat_map(move |dy| (0..mf_width).map(move |dx| (dx, dy)))
        .filter(|(x, y)| minefield.cells[y * mf_width + x].state == CellState::Opened)
        .filter_map(|(x, y)| {
            let mines = minefield.count_mines(x, y);

            if mines == 0 {
                None
            } else {
                Some((x, y, mines))
            }
        })
        .flat_map(|(x, y, mines)| {
            matrix_height += 1;

            let neighbor_mask = hidden_cells.iter().map(move |idx| {
                let (dx, dy) = (idx % mf_width, idx / mf_width);

                if dx.abs_diff(x) <= 1 && dy.abs_diff(y) <= 1 {
                    1
                } else {
                    0
                }
            });

            let value = mines as u8 - minefield.count_flags(x, y) as u8;

            x_inner.push(value);

            neighbor_mask
        });

    let a_inner: Vec<u8>;

    if include_total_mines {
        a_inner = a_inner_iter
            .chain(repeat(1).take(hidden_cells.len()))
            .collect::<Vec<u8>>();

        let undiscovered_mines = minefield.total_mines() - minefield.total_flags();

        x_inner.push(undiscovered_mines as u8);
        matrix_height += 1;
    } else {
        a_inner = a_inner_iter.collect::<Vec<u8>>();
    };

    let matrix_width = a_inner.len() / matrix_height;

    let a = DMatrix::from_row_slice(matrix_height, matrix_width, &a_inner);
    let x = DVector::from_row_slice(&x_inner);

    (a, x)
}

fn analyze_solutions(
    minefield: &mut Minefield,
    hidden_cells: &[usize],
    solutions: &[DVector<u8>],
) -> bool {
    let mf_width = minefield.width;

    let mut changed = false;

    if solutions.is_empty() {
        return false;
    }

    for i in 0..solutions[0].len() {
        let first = solutions[0][i];

        if solutions.iter().any(|sol| sol[i] != first) {
            continue;
        }

        changed = true;

        if first == 0 {
            minefield.open(hidden_cells[i] % mf_width, hidden_cells[i] / mf_width);
        } else {
            minefield.cells[hidden_cells[i]].state = CellState::Flagged;
        }
    }

    changed
}

pub fn find_solutions(
    a: DMatrixView<u8>,
    x: DVectorView<u8>,
    b: DVector<u8>,
    i: usize,
    solutions: &mut Vec<DVector<u8>>,
) {
    if i == a.ncols() {
        if a * &b == x {
            solutions.push(b);
        }

        return;
    }

    let mut with_one = b.clone_owned();
    with_one[i] = 1;

    find_solutions(a, x, b, i + 1, solutions);
    find_solutions(a, x, with_one, i + 1, solutions);
}

pub fn solve_step_bf(minefield: &mut Minefield) -> bool {
    let hidden_cells = get_hidden_cells(minefield, true);

    let (a, x) = create_system(minefield, &hidden_cells, true);

    let b = DVector::from_element(a.ncols(), 0);

    let mut solutions = Vec::new();

    find_solutions(a.as_view(), x.as_view(), b, 0, &mut solutions);

    analyze_solutions(minefield, &hidden_cells, &solutions)
}

pub fn find_solutions_pruning(
    a: DMatrixView<u8>,
    x: DVectorView<u8>,
    b: DVectorView<u8>,
    i: usize,
    solutions: &mut Vec<DVector<u8>>,
) {
    if i == a.ncols() {
        if a * b == x {
            solutions.push(b.clone_owned());
        }

        return;
    }

    let mut with_one = b.clone_owned();
    with_one[i] = 1;

    let x_0 = a * &with_one;

    if x_0 == x {
        find_solutions_pruning(a, x, with_one.as_view(), i + 1, solutions);
        solutions.push(with_one);
    } else if x_0
        .row_iter()
        .map(|row| row.sum())
        .zip(x.iter())
        .all(|(a, b)| a <= *b)
    {
        find_solutions_pruning(a, x, with_one.as_view(), i + 1, solutions);
    }

    find_solutions_pruning(a, x, b, i + 1, solutions);
}

pub fn solve_step_pruning(minefield: &mut Minefield) -> bool {
    let hidden_cells = get_hidden_cells(minefield, true);

    let (a, x) = create_system(minefield, &hidden_cells, true);

    let b = DVector::from_element(a.ncols(), 0);

    let mut solutions = Vec::new();

    find_solutions_pruning(a.as_view(), x.as_view(), b.as_view(), 0, &mut solutions);

    analyze_solutions(minefield, &hidden_cells, &solutions)
}

fn create_system_bm(
    minefield: &Minefield,
    hidden_cells: &[usize],
    include_total_mines: bool,
) -> (Vec<u64>, Vec<u32>) {
    let mf_height = minefield.height;
    let mf_width = minefield.width;

    let mut x_vector = Vec::new();

    let a_iter = (0..mf_height)
        .flat_map(move |dy| (0..mf_width).map(move |dx| (dx, dy)))
        .filter(|(x, y)| minefield.cells[y * mf_width + x].state == CellState::Opened)
        .filter_map(|(x, y)| {
            let mines = minefield.count_mines(x, y);

            if mines == 0 {
                None
            } else {
                Some((x, y, mines))
            }
        })
        .map(|(x, y, mines)| {
            let mut neighbor_mask: u64 = 0;
            for (idx, hidden_idx) in hidden_cells.iter().enumerate() {
                let (dx, dy) = (hidden_idx % mf_width, hidden_idx / mf_width);

                if dx.abs_diff(x) <= 1 && dy.abs_diff(y) <= 1 {
                    neighbor_mask |= 1 << idx;
                }
            }

            let value = mines as u32 - minefield.count_flags(x, y) as u32;

            x_vector.push(value);

            neighbor_mask
        });

    let a: Vec<u64>;

    if include_total_mines {
        let all_hidden = (0..hidden_cells.len()).fold(0, |acc, idx| acc | (1 << idx));
        a = a_iter.chain(once(all_hidden)).collect();

        let undiscovered_mines = minefield.total_mines() - minefield.total_flags();

        x_vector.push(undiscovered_mines as u32);
    } else {
        a = a_iter.collect();
    };

    (a, x_vector)
}

fn is_unrecoverable(a: &[u64], x: &[u32], b: u64) -> bool {
    for (a, x) in a.iter().zip(x.iter()) {
        if (a & b).count_ones() > *x {
            return true;
        }
    }

    false
}

fn is_solved(a: &[u64], x: &[u32], b: u64) -> bool {
    for (a, x) in a.iter().zip(x.iter()) {
        if (a & b).count_ones() != *x {
            return false;
        }
    }

    true
}

fn find_solutions_pruning_bm(
    a: &[u64],
    x: &[u32],
    size: usize,
    b: u64,
    i: usize,
    solutions: &mut Vec<u64>,
) {
    if i == size {
        if is_solved(a, x, b) {
            solutions.push(b);
        }

        return;
    }

    let mut with_one = b;
    with_one |= 1 << i;

    if is_solved(a, x, with_one) {
        find_solutions_pruning_bm(a, x, size, with_one, i + 1, solutions);
        solutions.push(with_one);
    } else if !is_unrecoverable(a, x, with_one) {
        find_solutions_pruning_bm(a, x, size, with_one, i + 1, solutions);
    }

    find_solutions_pruning_bm(a, x, size, b, i + 1, solutions);
}

fn analyze_solutions_bm(
    minefield: &mut Minefield,
    hidden_cells: &[usize],
    solutions: &[u64],
) -> bool {
    let mf_width = minefield.width;

    let mut changed = false;

    if solutions.is_empty() {
        return false;
    }

    for i in 0..hidden_cells.len() {
        let first = solutions[0] & (1 << i);

        if solutions.iter().any(|sol| sol & (1 << i) != first) {
            continue;
        }

        changed = true;

        if first == 0 {
            minefield.open(hidden_cells[i] % mf_width, hidden_cells[i] / mf_width);
        } else {
            minefield.cells[hidden_cells[i]].state = CellState::Flagged;
        }
    }

    changed
}
pub fn solve_step_pruning_bm(minefield: &mut Minefield, with_total_mines: bool) -> bool {
    let hidden_cells = get_hidden_cells(minefield, with_total_mines);

    let (a, x) = create_system_bm(minefield, &hidden_cells, with_total_mines);

    let b: u64 = 0;

    let mut solutions = Vec::new();

    find_solutions_pruning_bm(&a, &x, hidden_cells.len(), b, 0, &mut solutions);

    analyze_solutions_bm(minefield, &hidden_cells, &solutions)
}

fn get_hidden_cells_in_chuck(
    minefield: &Minefield,
    chuck_x: Range<usize>,
    chuck_y: Range<usize>,
) -> Vec<usize> {
    let mf_width = minefield.width;

    let is_edge = |x: usize, y: usize| {
        x + 1 == chuck_x.start || x == chuck_x.end || y + 1 == chuck_y.start || y == chuck_y.end
    };

    (0..(minefield.height * minefield.width))
        .filter(|idx| {
            is_edge(idx % mf_width, idx / mf_width)
                || (chuck_x.contains(&(idx % mf_width))
                    && chuck_y.contains(&(idx / mf_width))
                    && (minefield.cells[*idx].state == CellState::Hidden))
        })
        .filter(|idx| minefield.neighboring_open(*idx % minefield.width, *idx / minefield.width))
        .collect()
}

pub fn create_chuck_system(
    minefield: &Minefield,
    hidden_cells: &[usize],
    chuck_x: Range<usize>,
    chuck_y: Range<usize>,
) -> (DMatrix<u8>, DVector<u8>) {
    let hidden_cells: &[usize] = &hidden_cells;
    let mf_height = minefield.height;
    let mf_width = minefield.width;

    let mut matrix_height = 0;
    let mut x_inner = Vec::new();

    let a_inner = (0..mf_height)
        .flat_map(move |dy| (0..mf_width).map(move |dx| (dx, dy)))
        .filter(|(x, y)| minefield.cells[y * mf_width + x].state == CellState::Opened)
        .filter(|(x, y)| chuck_x.contains(&x) && chuck_y.contains(&y))
        .filter_map(|(x, y)| {
            let mines = minefield.count_mines(x, y);

            if mines == 0 {
                None
            } else {
                Some((x, y, mines))
            }
        })
        .flat_map(|(x, y, mines)| {
            matrix_height += 1;

            let neighbor_mask = hidden_cells.iter().map(move |idx| {
                let (dx, dy) = (idx % mf_width, idx / mf_width);

                if dx.abs_diff(x) <= 1 && dy.abs_diff(y) <= 1 {
                    1
                } else {
                    0
                }
            });

            let value = mines as u8 - minefield.count_flags(x, y) as u8;

            x_inner.push(value);

            neighbor_mask
        })
        .collect::<Vec<u8>>();

    if a_inner.is_empty() {
        return (DMatrix::from_element(0, 0, 0), DVector::from_element(0, 0));
    }

    let matrix_width = a_inner.len() / matrix_height;

    let a = DMatrix::from_row_slice(matrix_height, matrix_width, &a_inner);
    let x = DVector::from_row_slice(&x_inner);

    (a, x)
}

pub fn solve_chuck(
    minefield: &mut Minefield,
    chuck_x: Range<usize>,
    chuck_y: Range<usize>,
) -> bool {
    // let hidden_cells = get_hidden_cells(minefield, false);
    let hidden_cells = get_hidden_cells_in_chuck(minefield, chuck_x.clone(), chuck_y.clone());

    let (a, x) = create_chuck_system(minefield, &hidden_cells, chuck_x, chuck_y);

    if a.is_empty() {
        return false;
    }

    let b = DVector::from_element(a.ncols(), 0);

    let mut solutions = Vec::new();

    find_solutions(a.as_view(), x.as_view(), b, 0, &mut solutions);

    analyze_solutions(minefield, &hidden_cells, &solutions)
}

pub fn solve_step_chucking(
    minefield: &mut Minefield,
    chuck_size: usize,
    chuck_overlap: usize,
) -> bool {
    dbg!(&minefield);
    let mut chuck_y = 0..chuck_size;

    let mut changed = false;

    while chuck_y.start <= minefield.height {
        let mut chuck_x = 0..chuck_size;

        while chuck_x.start <= minefield.width {
            if solve_chuck(minefield, chuck_x.clone(), chuck_y.clone()) {
                changed = true;
            }

            chuck_x.start += chuck_size - chuck_overlap;
            chuck_x.end += chuck_size - chuck_overlap;
        }

        chuck_y.start += chuck_size - chuck_overlap;
        chuck_y.end += chuck_size - chuck_overlap;
    }
    dbg!(minefield);

    changed
}

pub fn solve_bf(minefield: &mut Minefield) {
    while solve_step_bf(minefield) {}
}

pub fn solve_pruning(minefield: &mut Minefield) {
    while solve_step_pruning(minefield) {}
}

pub fn solve_bm(minefield: &mut Minefield) {
    while solve_step_pruning_bm(minefield, true) {}
}

pub fn solve_bm_without_total_mines(minefield: &mut Minefield) {
    while solve_step_pruning_bm(minefield, false) {}
}

pub fn solve_chucking(minefield: &mut Minefield, chuck_size: usize, chuck_overlap: usize) {
    while solve_step_chucking(minefield, chuck_size, chuck_overlap) {}
}

pub fn solve_step(minefield: &mut Minefield) -> bool {
    solve_step_pruning_bm(minefield, true)
}

pub fn solve(minefield: &mut Minefield) {
    while solve_step(minefield) {}
}

#[cfg(test)]
mod tests {
    use rand::{rngs::StdRng, SeedableRng};

    use super::*;

    fn solve_bf_aux(mut minefield: Minefield) -> Minefield {
        solve_bf(&mut minefield);
        minefield
    }

    #[test]
    fn test_solve_step() {
        for (a, b) in &[
            (
                r#".1.
                   .F."#,
                r#"111
                   1F1"#,
            ),
            (
                r#"000m
             000.
             000.
             00m."#,
                r#"000F
             0000
             0000
             00F0"#,
            ),
            (
                r#"0000
                   0000
                   00m.
                   m..."#,
                r#"0000
                   0000
                   00F0
                   m.00"#,
            ),
            (
                r#"0000
                   0000
                   00F0
                   m.0."#,
                r#"0000
                   0000
                   00F0
                   F000
                   "#,
            ),
            (
                r#"00000m
                       000000
                       000000
                       000000"#,
                r#"00000F
                       000000
                       000000
                       000000"#,
            ),
            (
                r#"00001m
                    000011
                    000000
                    000000"#,
                r#"00001F
                    000011
                    000000
                    000000"#,
            ),
        ] {
            eprintln!("{}", Minefield::parse(a));
            let mut left = Minefield::parse(a);
            solve_step_bf(&mut left);

            let right = Minefield::parse(b);
            assert_eq!(left, right);
        }
    }

    #[ignore]
    #[test]
    fn test_solve_equality() {
        let mut rng = StdRng::seed_from_u64(0);

        for i in 0..100 {
            eprintln!("{}", i);
            let mut minefield = Minefield::random_start(&mut rng, 4, 4, 3);
            let mut minefield2 = minefield.clone();
            let mut minefield3 = minefield.clone();

            eprintln!("{}", minefield);

            solve_bf(&mut minefield);
            solve_pruning(&mut minefield2);
            solve_bm(&mut minefield3);

            assert_eq!(minefield, minefield2);
            assert_eq!(minefield, minefield3);
        }
    }

    #[ignore]
    #[test]
    fn test_solve_equality_without_end_game() {
        let mut rng = StdRng::seed_from_u64(0);

        for i in 0..10 {
            eprintln!("{}", i);
            let mut minefield1 = Minefield::random_start(&mut rng, 4, 4, 3);
            let mut minefield2 = minefield1.clone();

            eprintln!("{}", minefield1);

            solve_bm_without_total_mines(&mut minefield1);
            solve_chucking(&mut minefield2, 3, 0);

            assert_eq!(minefield1, minefield2);
        }
    }

    #[test]
    fn test_solve_bf() {
        let mut minefield = Minefield::parse(
            r#"0000
               0011
               112m
               .m.."#,
        );
        let expected = Minefield::parse(
            r#"0000
               0011
               112F
               1F21"#,
        );
        solve_bf(&mut minefield);
        assert_eq!(expected, minefield);
    }

    #[test]
    fn test_solve_pruning() {
        let expected = Minefield::parse(
            r#"FFF1
2321
0000
0000
"#,
        );
        let mut minefield = Minefield::parse(
            r#"FFF.
2321
0000
0000"#,
        );
        solve_pruning(&mut minefield);
        assert_eq!(expected, minefield);
    }
}
