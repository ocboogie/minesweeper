use minesweeper::minefield::Minefield;
use minesweeper::solver::{solve_bf, solve_bm, solve_endgame, solve_pruning};

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use rand::rngs::StdRng;
use rand::SeedableRng;

pub fn solver_benchmark(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(0);

    let minefields4x4 = (0..10)
        .map(|_| Minefield::random_start(&mut rng, 4, 4, 3))
        .collect::<Vec<_>>();
    let minefields6x6 = (0..3)
        .map(|_| Minefield::random_start(&mut rng, 6, 6, 5))
        .collect::<Vec<_>>();

    c.bench_function(
        "solver brute force with pruning and bitmasks 10x4x4x3 ",
        |b| {
            b.iter_batched(
                || minefields4x4.clone(),
                |mut minefields| {
                    for mf in minefields.iter_mut() {
                        solve_bm(mf);
                    }
                },
                BatchSize::SmallInput,
            );
        },
    );

    c.bench_function(
        "solver brute force with pruning and bitmasks 3x6x6x5 ",
        |b| {
            b.iter_batched(
                || minefields6x6.clone(),
                |mut minefields| {
                    for mf in minefields.iter_mut() {
                        solve_bm(mf);
                    }
                },
                BatchSize::SmallInput,
            );
        },
    );

    c.bench_function("solver brute force with pruning 10x4x4x3 ", |b| {
        b.iter_batched(
            || minefields4x4.clone(),
            |mut minefields| {
                for mf in minefields.iter_mut() {
                    solve_pruning(mf);
                }
            },
            BatchSize::SmallInput,
        );
    });

    c.bench_function("solver brute force 10x4x4x3 ", |b| {
        b.iter_batched(
            || minefields4x4.clone(),
            |mut minefields| {
                for mf in minefields.iter_mut() {
                    solve_bf(mf);
                }
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, solver_benchmark);
criterion_main!(benches);
