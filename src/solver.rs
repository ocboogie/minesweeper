use crate::minefield::{CellState, Minefield};
use log::warn;
use nalgebra::{DMatrix, DMatrixView, DVector, DVectorView};
use peroxide::{
    fuga::{LinearAlgebra, Shape},
    structure::matrix::matrix,
};
use std::{
    iter::{once, repeat},
    ops::Range,
};

pub fn solve_step_bf_aux(
    a: DMatrixView<f32>,
    x: DVectorView<f32>,
    b: DVector<f32>,
    i: usize,
    solutions: &mut Vec<DVector<f32>>,
) {
    if i == a.ncols() {
        if a * &b == x {
            solutions.push(b);
        }

        return;
    }

    let mut with_one = b.clone_owned();
    with_one[i] = 1.0;

    solve_step_bf_aux(a, x, b, i + 1, solutions);
    solve_step_bf_aux(a, x, with_one, i + 1, solutions);
}

pub fn solve_step_bf(minefield: &mut Minefield) -> bool {
    let mf_height = minefield.height;
    let mf_width = minefield.width;
    let undiscovered_mines = minefield.total_mines() - minefield.total_flags();

    let hidden_cells = (0..(mf_height * mf_width))
        .filter(|idx| minefield.cells[*idx].state == CellState::Hidden)
        .collect::<Vec<_>>();

    let mut matrix_height = 1;
    let mut x_inner = Vec::new();

    let a_inner = (0..mf_height)
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
                    1.0
                } else {
                    0.0
                }
            });

            let value = mines as f32 - minefield.count_flags(x, y) as f32;

            x_inner.push(value);

            neighbor_mask
        })
        .chain(repeat(1.0).take(hidden_cells.len()))
        .collect::<Vec<f32>>();

    x_inner.push(undiscovered_mines as f32);

    let matrix_width = a_inner.len() / matrix_height;

    let a = DMatrix::from_row_slice(matrix_height, matrix_width, &a_inner);
    let x = DVector::from_row_slice(&x_inner);
    let b = DVector::from_element(matrix_width, 0.0);

    let mut solutions = Vec::new();

    solve_step_bf_aux(a.as_view(), x.as_view(), b, 0, &mut solutions);

    if solutions.is_empty() {
        return false;
    }

    let mut changed = false;

    for i in 0..matrix_width {
        let first = solutions[0][i];

        if solutions.iter().any(|sol| sol[i] != first) {
            continue;
        }

        changed = true;

        if first == 0.0 {
            minefield.open(hidden_cells[i] % mf_width, hidden_cells[i] / mf_width);
        } else {
            minefield.cells[hidden_cells[i]].state = CellState::Flagged;
        }
    }

    changed
}

pub fn solve_step_pruning_aux(
    a: DMatrixView<f32>,
    x: DVectorView<f32>,
    b: DVectorView<f32>,
    i: usize,
    solutions: &mut Vec<DVector<f32>>,
) {
    if i == a.ncols() {
        if a * b == x {
            solutions.push(b.clone_owned());
        }

        return;
    }

    let mut with_one = b.clone_owned();
    with_one[i] = 1.0;

    let x_0 = a * &with_one;

    if x_0 == x {
        solve_step_pruning_aux(a, x, with_one.as_view(), i + 1, solutions);
        solutions.push(with_one);
    } else if x_0
        .row_iter()
        .map(|row| row.sum())
        .zip(x.iter())
        .all(|(a, b)| a <= *b)
    {
        solve_step_pruning_aux(a, x, with_one.as_view(), i + 1, solutions);
    }

    solve_step_pruning_aux(a, x, b, i + 1, solutions);
}

pub fn solve_step_pruning(minefield: &mut Minefield) -> bool {
    let mf_height = minefield.height;
    let mf_width = minefield.width;
    let undiscovered_mines = minefield.total_mines() - minefield.total_flags();

    let hidden_cells = (0..(mf_height * mf_width))
        .filter(|idx| minefield.cells[*idx].state == CellState::Hidden)
        .collect::<Vec<_>>();

    let mut matrix_height = 1;
    let mut x_inner = Vec::new();

    let a_inner = (0..mf_height)
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
                    1.0
                } else {
                    0.0
                }
            });

            let value = mines as f32 - minefield.count_flags(x, y) as f32;

            x_inner.push(value);

            neighbor_mask
        })
        .chain(repeat(1.0).take(hidden_cells.len()))
        .collect::<Vec<f32>>();

    x_inner.push(undiscovered_mines as f32);

    let matrix_width = a_inner.len() / matrix_height;

    let a = DMatrix::from_row_slice(matrix_height, matrix_width, &a_inner);
    let x = DVector::from_row_slice(&x_inner);
    let b = DVector::from_element(matrix_width, 0.0);

    let mut solutions = Vec::new();

    solve_step_pruning_aux(a.as_view(), x.as_view(), b.as_view(), 0, &mut solutions);

    if solutions.is_empty() {
        return false;
    }

    let mut changed = false;

    for i in 0..matrix_width {
        let first = solutions[0][i];

        if solutions.iter().any(|sol| sol[i] != first) {
            continue;
        }

        changed = true;

        if first == 0.0 {
            minefield.open(hidden_cells[i] % mf_width, hidden_cells[i] / mf_width);
        } else {
            minefield.cells[hidden_cells[i]].state = CellState::Flagged;
        }
    }

    changed
}

pub fn solve_step_chucking(
    minefield: &mut Minefield,
    chuck_size: usize,
    chuck_overlap: usize,
) -> bool {
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

    changed
}

pub fn solve_chuck(
    minefield: &mut Minefield,
    chuck_x: Range<usize>,
    chuck_y: Range<usize>,
) -> bool {
    let mf_height = minefield.height;
    let mf_width = minefield.width;

    let is_edge = |x: usize, y: usize| {
        x + 1 == chuck_x.start || x == chuck_x.end || y + 1 == chuck_y.start || y == chuck_y.end
    };

    let hidden_cells = (0..(mf_height * mf_width))
        .filter(|idx| {
            is_edge(idx % mf_width, idx / mf_width)
                || (chuck_x.contains(&(idx % mf_width))
                    && chuck_y.contains(&(idx / mf_width))
                    && (minefield.cells[*idx].state == CellState::Hidden))
        })
        .collect::<Vec<_>>();

    let mut matrix_height = 0;

    let inner_matrix = (0..mf_height)
        .flat_map(move |dy| (0..mf_width).map(move |dx| (dx, dy)))
        .filter(|(x, y)| chuck_x.contains(x) && chuck_y.contains(y))
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
                    1.0
                } else {
                    0.0
                }
            });

            let value = mines as f32 - minefield.count_flags(x, y) as f32;

            neighbor_mask.chain(once(value))
        })
        .collect::<Vec<f32>>();

    if matrix_height == 0 {
        warn!("No hidden cells to solve");
        return false;
    }

    let matrix_width = inner_matrix.len() / matrix_height;

    let matrix = matrix(inner_matrix, matrix_height, matrix_width, Shape::Row);

    let reduced = matrix.rref();

    let mut changed = false;

    for row in reduced.data.chunks(matrix_width) {
        let mut upper_bound: isize = 0;
        let mut lower_bound: isize = 0;

        for x in row[..row.len() - 1].iter() {
            if *x == 1.0 {
                upper_bound += 1;
            } else if *x == -1.0 {
                lower_bound -= 1;
            }
        }

        if upper_bound == 0 && lower_bound == 0 {
            continue;
        }

        let y = *row.last().unwrap() as isize;

        if upper_bound == y {
            for (val, idx) in row[..row.len() - 1]
                .iter()
                .zip(hidden_cells.iter())
                .filter(|(_, idx)| !is_edge(**idx % mf_width, **idx / mf_width))
            {
                if *val == 1.0 {
                    minefield.cells[*idx].state = CellState::Flagged;
                    changed = true;
                } else if *val == -1.0 {
                    minefield.open(*idx % mf_width, *idx / mf_width);
                    changed = true;
                }
            }
        } else if lower_bound == y {
            for (val, idx) in row[..row.len() - 1]
                .iter()
                .zip(hidden_cells.iter())
                .filter(|(_, idx)| !is_edge(**idx % mf_width, **idx / mf_width))
            {
                if *val == 1.0 {
                    minefield.open(*idx % mf_width, *idx / mf_width);
                    changed = true;
                } else if *val == -1.0 {
                    minefield.cells[*idx].state = CellState::Flagged;
                    changed = true;
                }
            }
        }
    }

    changed
}

pub fn solve_step_rref(minefield: &mut Minefield) -> bool {
    let mf_height = minefield.height;
    let mf_width = minefield.width;

    let mut changed = false;

    // dbg!(&minefield);
    for (x, y) in (0..mf_height)
        .flat_map(move |dy| (0..mf_width).map(move |dx| (dx, dy)))
        .filter(|(x, y)| minefield.cells[y * mf_width + x].state == CellState::Opened)
        .collect::<Vec<_>>()
    {
        let mines = minefield.count_mines(x, y);
        let flags = minefield.count_flags(x, y);
        let hidden = minefield.count_hidden(x, y);

        // dbg!(x, y, mines, flags, hidden);

        if mines == flags {
            for (x, y) in minefield.neighbors(x, y) {
                if minefield.cells[y * mf_width + x].state == CellState::Hidden {
                    minefield.open(x, y);
                    changed = true;
                }
            }
        }
        if mines - flags == hidden {
            for (x, y) in minefield.neighbors(x, y) {
                if minefield.cells[y * mf_width + x].state == CellState::Hidden {
                    minefield.cells[y * mf_width + x].state = CellState::Flagged;
                    changed = true;
                }
            }
        }
    }

    let undiscovered_mines = minefield.total_mines() - minefield.total_flags();

    // dbg!(&minefield);

    let hidden_cells = (0..(mf_height * mf_width))
        .filter(|idx| minefield.cells[*idx].state == CellState::Hidden)
        // .filter(|idx| minefield.neighboring_open(*idx % mf_width, *idx / mf_width))
        .collect::<Vec<_>>();

    let mut matrix_height = 1;

    let inner_matrix = (0..mf_height)
        .flat_map(move |dy| (0..mf_width).map(move |dx| (dx, dy)))
        .filter(|(x, y)| minefield.cells[y * mf_width + x].state == CellState::Opened)
        .map(|(x, y)| {
            let mines = minefield.count_mines(x, y);
            let flags = minefield.count_flags(x, y);

            // if mines == 0 {
            //     None
            // } else {
            //     Some((x, y, mines as f32 - flags as f32))
            // }
            (x, y, mines as f32 - flags as f32)
        })
        .flat_map(|(x, y, value)| {
            matrix_height += 1;

            let neighbor_mask = hidden_cells.iter().map(move |idx| {
                let (dx, dy) = (idx % mf_width, idx / mf_width);

                if dx.abs_diff(x) <= 1 && dy.abs_diff(y) <= 1 {
                    1.0
                } else {
                    0.0
                }
            });

            neighbor_mask.chain(once(value))
        })
        .chain(repeat(1.0).take(hidden_cells.len()))
        .chain(once(undiscovered_mines as f32))
        .collect::<Vec<f32>>();

    if matrix_height == 0 {
        warn!("No hidden cells to solve");
        return false;
    }

    let matrix_width = inner_matrix.len() / matrix_height;

    let matrix_b = matrix(
        inner_matrix.clone(),
        matrix_height,
        matrix_width,
        Shape::Row,
    );

    eprintln!("{}", matrix_b);

    let reduced = matrix_b.rref();

    eprintln!("{}", reduced);

    for row in reduced.data.chunks(matrix_width) {
        let y = row[row.len() - 1] as isize;

        let mut upper_bound: isize = 0;
        let mut lower_bound: isize = 0;

        for x in row[..row.len() - 1].iter() {
            if *x == 1.0 {
                upper_bound += 1;
            } else if *x == -1.0 {
                lower_bound -= 1;
            }
        }

        if upper_bound == 0 && lower_bound == 0 {
            continue;
        }

        if upper_bound == y {
            for (val, idx) in row[..row.len() - 1].iter().zip(hidden_cells.iter()) {
                if *val == 1.0 {
                    changed = true;
                    minefield.cells[*idx].state = CellState::Flagged;
                } else if *val == -1.0 {
                    changed = true;
                    minefield.open(*idx % mf_width, *idx / mf_width);
                }
            }
        } else if lower_bound == y {
            for (val, idx) in row[..row.len() - 1].iter().zip(hidden_cells.iter()) {
                if *val == 1.0 {
                    changed = true;
                    minefield.open(*idx % mf_width, *idx / mf_width);
                } else if *val == -1.0 {
                    changed = true;
                    minefield.cells[*idx].state = CellState::Flagged;
                }
            }
        }
    }

    changed
}

pub fn solve_bf(minefield: &mut Minefield) {
    while solve_step_bf(minefield) {}
}

pub fn solve_rref(minefield: &mut Minefield) {
    while solve_step_rref(minefield) {}
}

pub fn solve_chucking(minefield: &mut Minefield, chuck_size: usize, chuck_overlap: usize) {
    while solve_step_chucking(minefield, chuck_size, chuck_overlap) {}
}

pub fn solve_pruning(minefield: &mut Minefield) {
    while solve_step_pruning(minefield) {}
}

#[cfg(test)]
mod tests {
    use faer::Mat;
    use rand::{rngs::StdRng, SeedableRng};

    use super::*;

    fn solve_step_chucking_aux(mut minefield: Minefield) -> Minefield {
        solve_step_chucking(&mut minefield, 5, 1);
        minefield
    }

    fn solve_step_rref_aux(mut minefield: Minefield) -> Minefield {
        solve_step_rref(&mut minefield);
        minefield
    }

    fn solve_chuck_aux(
        mut minefield: Minefield,
        chuck_x: Range<usize>,
        chuck_y: Range<usize>,
    ) -> Minefield {
        solve_chuck(&mut minefield, chuck_x, chuck_y);
        minefield
    }

    fn solve_bf_aux(mut minefield: Minefield) -> Minefield {
        solve_bf(&mut minefield);
        minefield
    }

    fn solve_rref_aux(mut minefield: Minefield) -> Minefield {
        solve_rref(&mut minefield);
        minefield
    }

    fn solve_chucking_aux(mut minefield: Minefield) -> Minefield {
        solve_chucking(&mut minefield, 5, 1);
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

    #[test]
    fn test_solve_equality() {
        let mut rng = StdRng::seed_from_u64(0);

        for i in 0..1000 {
            eprintln!("{}", i);
            let mut minefield = Minefield::random_start(&mut rng, 4, 4, 3);
            let mut minefield2 = minefield.clone();

            eprintln!("{}", minefield);

            solve_bf(&mut minefield);
            solve_pruning(&mut minefield2);

            assert_eq!(minefield, minefield2);
        }
    }

    #[ignore]
    #[test]
    fn test_solve_rref() {
        let mut minefield = Minefield::parse(
            r#".m10
               ..10
               ..11
               ..m."#,
        );
        let expected = Minefield::parse(
            r#"1F10
               1110
               0111
               01F1"#,
        );
        solve_rref(&mut minefield);
        assert_eq!(expected, minefield);

        let mut minefield = Minefield::parse(
            r#"001m
               001.
               111.
               .m.."#,
        );
        let expected = Minefield::parse(
            r#"001F
               0011
               1110
               1F10"#,
        );
        solve_rref(&mut minefield);
        assert_eq!(expected, minefield);

        let mut minefield = Minefield::parse(
            r#".100
               m210
               .m10
               ..10"#,
        );
        let expected = Minefield::parse(
            r#"1100
               F210
               2F10
               1110"#,
        );
        solve_rref(&mut minefield);
        assert_eq!(expected, minefield);

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
        solve_rref(&mut minefield);
        assert_eq!(expected, minefield);

        let mut minefield = Minefield::parse(
            r#"001.
               111m
               m.2.
               ...m"#,
        );
        let expected = Minefield::parse(
            r#"0011
               111F
               F122
               111F"#,
        );
        solve_rref(&mut minefield);
        assert_eq!(expected, minefield);
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
