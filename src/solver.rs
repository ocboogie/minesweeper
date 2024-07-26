use crate::minefield::{CellKind, CellState, Minefield};
use faer::{
    linalg::zip::{MatShape, ViewMut},
    mat::from_row_major_slice,
    reborrow::{Reborrow, ReborrowMut},
    scale,
    sparse::SparseRowMatMut,
    unzipped, zipped, MatMut,
};
use log::warn;
use peroxide::{
    fuga::{LinearAlgebra, Shape},
    structure::matrix::matrix,
};
use rand::thread_rng;
use std::{
    iter::once,
    ops::{MulAssign, Range},
    sync::{
        atomic::{AtomicU32, Ordering},
        mpsc::{channel, sync_channel, Receiver, Sender, TryRecvError},
        Arc, Mutex,
    },
    thread,
};
use web_time::{Duration, Instant};

const MIN_RENDER_INTERVAL: Duration = Duration::from_millis(100);

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

pub fn convert_ref_to_rref(mut matrix: MatMut<f32>) {
    for y in (0..matrix.nrows()).rev() {
        let mut row = matrix.rb_mut().row_mut(y);
        let Some((leading, pivot)) = row
            .rb_mut()
            .iter()
            .cloned()
            .enumerate()
            .find(|(_, a)| *a != 0.0)
        else {
            continue;
        };

        row.mul_assign(scale(1.0 / pivot));

        for y2 in (0..y).rev() {
            let above = matrix.rb().row(y2)[leading];

            for x in leading..matrix.ncols() {
                matrix.write(y2, x, matrix.read(y2, x) - above * matrix.read(y, x));
            }
        }
    }
}

// pub fn sparse_convert_ref_to_rref(mut matrix: SparseRowMatMut<f32>) {
//     for y in (0..matrix.nrows()).rev() {
//         let mut row = matrix.rb_mut().row_mut(y);
//         let Some((leading, pivot)) = row
//             .rb_mut()
//             .iter()
//             .cloned()
//             .enumerate()
//             .find(|(_, a)| *a != 0.0)
//         else {
//             continue;
//         };
//
//         row.mul_assign(scale(1.0 / pivot));
//
//         for y2 in (0..y).rev() {
//             let above = matrix.rb().row(y2)[leading];
//
//             for x in leading..matrix.ncols() {
//                 matrix.write(y2, x, matrix.read(y2, x) - above * matrix.read(y, x));
//             }
//         }
//     }
// }

pub fn solve_step_rref(minefield: &mut Minefield) -> bool {
    eprintln!("before: {}", minefield);
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

            // TODO: Add optimization for when mines == 0
            if mines == 0 {
                None
            } else {
                Some((x, y, mines))
            }
            // (x, y, mines)
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

    dbg!(
        inner_matrix.len() / matrix_height,
        matrix_height,
        inner_matrix.len()
    );

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

    eprintln!("{}", matrix_b.submat((0, 0), (7, 24)));
    eprintln!(
        "{}",
        matrix_b.submat((0, matrix_width - 1), (7, matrix_width - 1))
    );
    // eprintln!("{}",

    let reduced = matrix_b.rref();

    eprintln!("{}", reduced.submat((0, 0), (7, 24)));
    eprintln!(
        "{}",
        reduced.submat((0, matrix_width - 1), (7, matrix_width - 1))
    );

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

        let y = row[row.len() - 1] as isize;

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

pub fn solve_rref(minefield: &mut Minefield) {
    while solve_step_rref(minefield) {}
}

pub fn solve_chucking(minefield: &mut Minefield, chuck_size: usize, chuck_overlap: usize) {
    while solve_step_chucking(minefield, chuck_size, chuck_overlap) {}
}

pub enum GeneratorStatus {
    Found(Minefield),
    StillSolving(Option<Minefield>),
}

pub struct ParallelGuessfreeGenerator {
    pub attempts: Arc<AtomicU32>,
    pub found: Receiver<Minefield>,
    pub stuck: Arc<Mutex<Option<Minefield>>>,
    pub cancel: Sender<()>,
}

impl ParallelGuessfreeGenerator {
    pub fn new(
        start: usize,
        width: usize,
        height: usize,
        mines: usize,
    ) -> ParallelGuessfreeGenerator {
        let (tx, rx) = sync_channel(1);
        let (cancel_tx, cancel_rx) = channel();

        let attempts = Arc::new(AtomicU32::new(0));

        let stuck = Arc::new(Mutex::new(None));

        let generator = ParallelGuessfreeGenerator {
            attempts: attempts.clone(),
            found: rx,
            stuck: stuck.clone(),
            cancel: cancel_tx,
        };

        thread::spawn(move || loop {
            let mut minefield = Minefield::generate(&mut thread_rng(), width, height, mines);
            attempts.fetch_add(1, Ordering::Relaxed);

            if minefield.cells[start].kind == CellKind::Mine {
                continue;
            }

            minefield.open(start % width, start / width);

            solve_rref(&mut minefield);

            if cancel_rx.try_recv().is_ok() {
                return;
            }

            if minefield.is_solved() {
                let _ = tx.send(minefield);
                return;
            }

            {
                let mut stuck = stuck.lock().unwrap();
                *stuck = Some(minefield);
            }
        });

        generator
    }

    pub fn attempts(&self) -> usize {
        self.attempts.load(Ordering::Relaxed) as usize
    }

    pub fn run(&mut self) -> GeneratorStatus {
        match self.found.try_recv() {
            Ok(minefield) => GeneratorStatus::Found(minefield),
            Err(TryRecvError::Empty) => {
                GeneratorStatus::StillSolving(self.stuck.lock().unwrap().clone())
            }
            Err(TryRecvError::Disconnected) => {
                panic!("Generator thread disconnected")
            }
        }
    }
}

impl Drop for ParallelGuessfreeGenerator {
    fn drop(&mut self) {
        let _ = self.cancel.send(());
    }
}

pub struct AsyncGuessfreeGenerator {
    start: usize,
    mines: usize,
    width: usize,
    height: usize,
    attempts: usize,
    solving: Option<Minefield>,
}

impl AsyncGuessfreeGenerator {
    pub fn new(start: usize, width: usize, height: usize, mines: usize) -> Self {
        AsyncGuessfreeGenerator {
            start,
            mines,
            width,
            height,
            attempts: 0,
            solving: Some(Minefield::new(width, height)),
        }
    }

    pub fn attempts(&self) -> usize {
        self.attempts
    }

    fn find_initial_minefield(&mut self) -> &mut Minefield {
        loop {
            self.attempts += 1;

            let mut minefield =
                Minefield::generate(&mut thread_rng(), self.width, self.height, self.mines);

            if minefield.cells[self.start].kind != CellKind::Mine {
                minefield.open(self.start % self.width, self.start / self.width);
                self.solving = Some(minefield);
                return self.solving.as_mut().unwrap();
            }
        }
    }

    pub fn run(&mut self) -> GeneratorStatus {
        let start_instant = Instant::now();

        let mut minefield = match self.solving {
            Some(ref mut minefield) => minefield,
            None => self.find_initial_minefield(),
        };

        while start_instant.elapsed() < MIN_RENDER_INTERVAL {
            let stuck = solve_step_rref(minefield);

            if minefield.is_solved() {
                let mut solved_minefield = minefield.clone();
                solved_minefield.hide();
                solved_minefield.open(self.start % self.width, self.start / self.width);
                return GeneratorStatus::Found(solved_minefield);
            }

            if stuck {
                minefield = self.find_initial_minefield();
            }
        }

        GeneratorStatus::StillSolving(Some(minefield.clone()))
    }
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

    fn solve_step_aux(mut minefield: Minefield) -> Minefield {
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

    fn solve_aux(mut minefield: Minefield) -> Minefield {
        solve_rref(&mut minefield);
        minefield
    }

    fn solve_chucking_aux(mut minefield: Minefield) -> Minefield {
        solve_chucking(&mut minefield, 5, 1);
        minefield
    }

    #[ignore]
    #[test]
    fn test_solve_chuck() {
        assert_eq!(
            solve_chuck_aux(
                Minefield::parse(
                    r#"000m
                       000.
                       000.
                       00m."#,
                ),
                0..3,
                0..3
            ),
            Minefield::parse(
                r#"000m
                   000.
                   000.
                   00m."#
            )
        );

        assert_eq!(
            solve_chuck_aux(
                Minefield::parse(
                    r#"m11m
                       111.
                       011.
                       01m."#,
                ),
                0..3,
                0..2
            ),
            Minefield::parse(
                r#"F00m
                   000.
                   000.
                   00m."#
            )
        );

        assert_eq!(
            solve_chuck_aux(
                Minefield::parse(
                    r#"m11m
                       111.
                       011.
                       01m."#,
                ),
                0..3,
                0..2
            ),
            Minefield::parse(
                r#"F00m
                   000.
                   000.
                   00m."#
            )
        );
    }

    #[ignore]
    #[test]
    fn test_solve_step_chucking() {
        for (a, b) in &[(
            r#"00011.
               0001F.
               00011."#,
            r#"000111
               0001F1
               000111"#,
        )] {
            eprintln!("{}", Minefield::parse(a));
            assert_eq!(
                solve_step_chucking_aux(Minefield::parse(a)),
                Minefield::parse(b)
            );
        }
    }

    #[ignore]
    #[test]
    fn test_solve_step() {
        for (a, b) in &[
            (
                r#"0011..
                   001m..
                   0012..
                   1122m.
                   1F2m..
                   1133mm
                   012m.."#,
                r#"000F
             0000
             0000
             00F0"#,
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
                   m.0."#,
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
            // (
            //     r#"00011.
            //        0001F.
            //        00011."#,
            //     r#"000111
            //        0001F1
            //        000111"#,
            // ),
        ] {
            eprintln!("{}", Minefield::parse(a));
            assert_eq!(solve_step_aux(Minefield::parse(a)), Minefield::parse(b));
        }
    }

    #[ignore]
    #[test]
    fn test_solve_step_equality() {
        let mut rng = StdRng::seed_from_u64(0);

        for _ in 0..100 {
            let mut minefield = Minefield::random_start(&mut rng, 6, 3, 1);
            let mut minefield2 = minefield.clone();

            eprintln!("{}", minefield);

            solve_step_chucking(&mut minefield, 5, 1);
            solve_step_rref(&mut minefield2);

            assert_eq!(minefield, minefield2);
        }
    }

    #[ignore]
    #[test]
    fn test_solve_equality() {
        let mut rng = StdRng::seed_from_u64(0);

        for _ in 0..100 {
            let mut minefield = Minefield::random_start(&mut rng, 30, 16, 99);
            let mut minefield2 = minefield.clone();

            eprintln!("{}", minefield);

            solve_chucking(&mut minefield, 5, 4);
            solve_rref(&mut minefield2);

            assert_eq!(minefield, minefield2);
        }
    }

    #[ignore]
    #[test]
    fn test_solve_chucking() {
        for (a, b) in &[(
            r#"...110
               ..1F10
               ..1110"#,
            r#"001110
               001F10
               001110"#,
        )] {
            eprintln!("{}", Minefield::parse(a));
            assert_eq!(solve_chucking_aux(Minefield::parse(a)), Minefield::parse(b));
        }
    }

    #[test]
    fn test_solve() {
        let solved = solve_aux(Minefield::parse(
            r#"m.1m.1......
               22112F22m..m
               F100223m....
               11001F3.m...
               001134F42...
               124F4FF3F2..
               1FFFF43223m.
               12333F101F..
               1100111022.m
               F21000001F3.
               3F100011213F
               F310002F202m
               F200002F201."#,
        ));
        // let solved = solve_aux(Minefield::parse(
        //     r#"m.1m.1......
        //        22112F..m..m
        //        F10022.m....
        //        11001F..m...
        //        001134m.....
        //        124F4FF3F2..
        //        1FFFF32212m.
        //        1233321012..
        //        00001F101F..
        //        1100111022.m
        //        F21000001F3.
        //        3F100011213m
        //        F310002F202m
        //        F322224F323."#,
        // ));

        panic!("{}", solved);
        return;

        for (a, b) in &[
            (
                r#"m.1
               221"#,
                r#"F11
               221"#,
            ),
            (
                r#"m.1m.1.
                   22112F."#,
                r#"F11F211
                   22112F1"#,
            ),
        ] {
            eprintln!("{}", Minefield::parse(a));
            assert_eq!(solve_aux(Minefield::parse(a)), Minefield::parse(b));
        }
    }

    #[ignore]
    #[test]
    fn test_solve_is_solved() {
        assert!(solve_aux(Minefield::parse(
            r#".m.......
            112mm....
            0012211..
            0000001m.
            11111012m
            m..m10011
            ....10000
            ....10111
            m..m101m."#
        ))
        .is_solved());

        assert!(!solve_aux(Minefield::parse(
            r#"00001m...
    00112....
    001m.....
    001..m...
    001.m...m
    001m....m
    0012.....
    0001m.m.m
    0001....."#
        ))
        .is_solved());

        assert!(solve_aux(Minefield::parse(
            r#".1001....
    m1001m...
    11001....
    00112.m..
    001m.m...
    0011..mm.
    0001.m...
    1101m....
    m101....."#
        ))
        .is_solved());
    }

    // #[test]
    // fn test_solve() {
    //     assert_eq!(
    //         solve_aux(Minefield::parse(
    //             r#"00000000
    // 00000000
    // m..m0000
    // 00000000
    // 00000000
    // 00000000
    // mm000000
    // ..000000"#,
    //         ))
    //         .format(),
    //         r#"00000000
    // 00000000
    // F00F0000
    // 00000000
    // 00000000
    // 00000000
    // FF000000
    // 00000000
    // "#
    //     );
    //     assert_eq!(
    //         solve_aux(Minefield::parse(
    //             r#".0000000
    // m0000000
    // 00000000
    // 00000m00
    // 00000000
    // 00000000
    // m.m00000
    // ...00000"#,
    //         ))
    //         .format(),
    //         r#"00000000
    // F0000000
    // 00000000
    // 00000F00
    // 00000000
    // 00000000
    // F0F00000
    // 00000000
    // "#
    //     );
    //     assert_eq!(
    //         solve_aux(Minefield::parse(
    //             r#"........
    // ...m.m..
    // .m00000m
    // .000000.
    // m000000.
    // .0000m..
    // m0000m..
    // .0000.mm"#,
    //         ))
    //         .format(),
    //         r#"00000000
    // 000F0F00
    // 0F00000F
    // 00000000
    // F0000000
    // 00000F00
    // F0000F00
    // 000000FF
    // "#
    //     );
    //     assert_eq!(
    //         solve_aux(Minefield::parse(
    //             r#".......m
    // ....m.mm
    // ...0000.
    // ...0000m
    // m.m0000.
    // ..00000.
    // .m0000m.
    // ..0000m."#,
    //         ))
    //         .format(),
    //         r#"0000000F
    // 0000F0FF
    // 00000000
    // 0000000F
    // F0F00000
    // 00000000
    // 0F0000F0
    // 000000F0
    // "#
    //     );
    // }

    #[test]
    fn test_convert_ref_to_rref() {
        use faer::mat;

        let mut matrix: Mat<f32> = mat![
            [1.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 2.0, 1.0],
            [0.0, 0.0, 0.0]
        ];

        convert_ref_to_rref(matrix.as_mut());

        assert_eq!(
            matrix,
            mat![
                [1.0, 0.0, -0.5 as f32],
                [0.0, 0.0, 0.0],
                [0.0, 1.0, 0.5],
                [0.0, 0.0, 0.0]
            ]
        );
    }
}
