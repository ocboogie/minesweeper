use crate::minefield::{CellState, Minefield};
use nalgebra::{DMatrix, DMatrixView, DVector, DVectorView};
use std::iter::repeat;

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
) -> (DMatrix<f32>, DVector<f32>) {
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
                    1.0
                } else {
                    0.0
                }
            });

            let value = mines as f32 - minefield.count_flags(x, y) as f32;

            x_inner.push(value);

            neighbor_mask
        });

    let a_inner: Vec<f32>;

    if include_total_mines {
        a_inner = a_inner_iter
            .chain(repeat(1.0).take(hidden_cells.len()))
            .collect::<Vec<f32>>();

        let undiscovered_mines = minefield.total_mines() - minefield.total_flags();

        x_inner.push(undiscovered_mines as f32);
        matrix_height += 1;
    } else {
        a_inner = a_inner_iter.collect::<Vec<f32>>();
    };

    let matrix_width = a_inner.len() / matrix_height;

    let a = DMatrix::from_row_slice(matrix_height, matrix_width, &a_inner);
    let x = DVector::from_row_slice(&x_inner);

    (a, x)
}

fn analyze_solutions(
    minefield: &mut Minefield,
    hidden_cells: &[usize],
    solutions: &[DVector<f32>],
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

        if first == 0.0 {
            minefield.open(hidden_cells[i] % mf_width, hidden_cells[i] / mf_width);
        } else {
            minefield.cells[hidden_cells[i]].state = CellState::Flagged;
        }
    }

    changed
}

pub fn find_solutions(
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

    find_solutions(a, x, b, i + 1, solutions);
    find_solutions(a, x, with_one, i + 1, solutions);
}

pub fn solve_step_bf(minefield: &mut Minefield) -> bool {
    let hidden_cells = get_hidden_cells(minefield, true);

    let (a, x) = create_system(minefield, &hidden_cells, true);

    let b = DVector::from_element(a.ncols(), 0.0);

    let mut solutions = Vec::new();

    find_solutions(a.as_view(), x.as_view(), b, 0, &mut solutions);

    analyze_solutions(minefield, &hidden_cells, &solutions)
}

pub fn find_solutions_pruning(
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

    let b = DVector::from_element(a.ncols(), 0.0);

    let mut solutions = Vec::new();

    find_solutions_pruning(a.as_view(), x.as_view(), b.as_view(), 0, &mut solutions);

    analyze_solutions(minefield, &hidden_cells, &solutions)
}

pub fn solve_step_endgame(minefield: &mut Minefield) -> bool {
    let hidden_cells = get_hidden_cells(minefield, false);

    let (a, x) = create_system(minefield, &hidden_cells, false);

    let b = DVector::from_element(a.ncols(), 0.0);

    let mut solutions = Vec::new();

    find_solutions_pruning(a.as_view(), x.as_view(), b.as_view(), 0, &mut solutions);

    let changed = analyze_solutions(minefield, &hidden_cells, &solutions);

    if !changed {
        return solve_step_pruning(minefield);
    }

    changed
}

pub fn solve_bf(minefield: &mut Minefield) {
    while solve_step_bf(minefield) {}
}

pub fn solve_pruning(minefield: &mut Minefield) {
    while solve_step_pruning(minefield) {}
}

pub fn solve_endgame(minefield: &mut Minefield) {
    while solve_step_endgame(minefield) {}
}

pub fn solve_step(minefield: &mut Minefield) -> bool {
    solve_step_pruning(minefield)
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
            solve_endgame(&mut minefield3);

            assert_eq!(minefield, minefield2);
            assert_eq!(minefield, minefield3);
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
