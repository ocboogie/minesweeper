use crate::minefield::{CellKind, CellState, Minefield};
use log::warn;
use peroxide::{
    fuga::{LinearAlgebra, Shape},
    structure::matrix::matrix,
};
use std::{
    iter::once,
    sync::{
        atomic::{AtomicU32, Ordering},
        mpsc::{channel, sync_channel, Receiver, Sender},
        Arc,
    },
    thread,
};

pub fn solve_step(mut minefield: Minefield) -> Minefield {
    let mf_height = minefield.height;
    let mf_width = minefield.width;

    let hidden_cells = (0..(mf_height * mf_width))
        .filter(|idx| minefield.cells[*idx].state == CellState::Hidden)
        .collect::<Vec<_>>();

    let mut matrix_height = 0;

    let inner_matrix = (0..mf_height)
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

            neighbor_mask.chain(once(value))
        })
        .collect::<Vec<f32>>();

    if matrix_height == 0 {
        warn!("No hidden cells to solve");
        return minefield;
    }

    let matrix_width = inner_matrix.len() / matrix_height;

    let matrix = matrix(inner_matrix, matrix_height, matrix_width, Shape::Row);

    let reduced = matrix.rref();

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
            for (val, idx) in row[..row.len() - 1].iter().zip(hidden_cells.iter()) {
                if *val == 1.0 {
                    minefield.cells[*idx].state = CellState::Flagged;
                } else if *val == -1.0 {
                    minefield.open(*idx % mf_width, *idx / mf_width);
                }
            }
        } else if lower_bound == y {
            for (val, idx) in row[..row.len() - 1].iter().zip(hidden_cells.iter()) {
                if *val == 1.0 {
                    minefield.open(*idx % mf_width, *idx / mf_width);
                } else if *val == -1.0 {
                    minefield.cells[*idx].state = CellState::Flagged;
                }
            }
        }
    }

    minefield
}

pub fn solve(minefield: &Minefield) -> Minefield {
    let mut minefield = minefield.clone();
    loop {
        let new_minefield = solve_step(minefield.clone());

        if new_minefield == minefield {
            return new_minefield;
        }

        minefield = new_minefield;
    }
}

pub struct GuessfreeGenerator {
    pub attempts: Arc<AtomicU32>,
    pub board: Receiver<Minefield>,
    pub cancel: Sender<()>,
}

pub fn generate_guessfree(
    start: usize,
    width: usize,
    height: usize,
    mines: usize,
) -> GuessfreeGenerator {
    let (tx, rx) = sync_channel(1);
    let (cancel_tx, cancel_rx) = channel();

    let attempts = Arc::new(AtomicU32::new(0));

    let generator = GuessfreeGenerator {
        attempts: attempts.clone(),
        board: rx,
        cancel: cancel_tx,
    };

    thread::spawn(move || loop {
        if cancel_rx.try_recv().is_ok() {
            return;
        }

        let mut minefield = Minefield::generate(width, height, mines);
        attempts.fetch_add(1, Ordering::Relaxed);

        if minefield.cells[start].kind == CellKind::Mine {
            continue;
        }

        minefield.open(start % width, start / width);

        if !solve(&minefield).is_solved() {
            continue;
        }

        let _ = tx.send(minefield);
        return;
    });

    generator
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solve_step() {
        assert_eq!(
            solve_step(Minefield::parse(
                r#"000m
000.
000.
00m."#,
            ))
            .format(),
            r#"000F
0000
0000
00F0
"#
        );
        assert_eq!(
            solve_step(Minefield::parse(
                r#"0000
0000
00m.
m..."#,
            ))
            .format(),
            r#"0000
0000
00F0
m.0.
"#
        );
        assert_eq!(
            solve_step(Minefield::parse(
                r#"0000
0000
00F0
m.0."#,
            ))
            .format(),
            r#"0000
0000
00F0
F000
"#
        );
    }

    #[test]
    fn test_solve() {
        assert_eq!(
            solve(&Minefield::parse(
                r#"00000000
00000000
m..m0000
00000000
00000000
00000000
mm000000
..000000"#,
            ))
            .format(),
            r#"00000000
00000000
F00F0000
00000000
00000000
00000000
FF000000
00000000
"#
        );
        assert_eq!(
            solve(&Minefield::parse(
                r#".0000000
m0000000
00000000
00000m00
00000000
00000000
m.m00000
...00000"#,
            ))
            .format(),
            r#"00000000
F0000000
00000000
00000F00
00000000
00000000
F0F00000
00000000
"#
        );
        assert_eq!(
            solve(&Minefield::parse(
                r#"........
...m.m..
.m00000m
.000000.
m000000.
.0000m..
m0000m..
.0000.mm"#,
            ))
            .format(),
            r#"00000000
000F0F00
0F00000F
00000000
F0000000
00000F00
F0000F00
000000FF
"#
        );
        assert_eq!(
            solve(&Minefield::parse(
                r#".......m
....m.mm
...0000.
...0000m
m.m0000.
..00000.
.m0000m.
..0000m."#,
            ))
            .format(),
            r#"0000000F
0000F0FF
00000000
0000000F
F0F00000
00000000
0F0000F0
000000F0
"#
        );
    }
}
