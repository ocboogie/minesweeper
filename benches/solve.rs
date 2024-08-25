use criterion::measurement::WallTime;
use minesweeper::minefield::Minefield;
use minesweeper::solver::{
    solve_bf, solve_bm, solve_bm_without_total_mines, solve_chucking, solve_pruning,
};

use criterion::{
    criterion_group, criterion_main, BatchSize, BenchmarkGroup, BenchmarkId, Criterion,
};
use rand::rngs::StdRng;
use rand::SeedableRng;

fn bench_solver(
    group: &mut BenchmarkGroup<WallTime>,
    (mf_label, minefields): &(&str, Vec<Minefield>),
    (solver_label, solver): (&str, &dyn Fn(&mut Minefield)),
) {
    group.bench_with_input(
        BenchmarkId::new(solver_label, mf_label),
        &minefields,
        |b, minefields| {
            b.iter_batched(
                || minefields.to_vec(),
                |mut minefields| {
                    for mf in minefields.iter_mut() {
                        solver(mf);
                    }
                },
                BatchSize::SmallInput,
            );
        },
    );
}

pub fn solver_benchmark(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(0);
    let mut group = c.benchmark_group("Solver");

    group.sample_size(10);
    group.warm_up_time(std::time::Duration::from_millis(500));
    group.measurement_time(std::time::Duration::from_secs(1));

    let minefields4x4m3 = (
        "10:4x4,3",
        (0..10)
            .map(|_| Minefield::random_start(&mut rng, 4, 4, 3))
            .collect::<Vec<_>>(),
    );
    let minefields6x6m5 = (
        "3:6x6,5",
        (0..3)
            .map(|_| Minefield::random_start(&mut rng, 6, 6, 5))
            .collect::<Vec<_>>(),
    );
    let minefields9x9m10 = (
        "10:9x9,10",
        (0..10)
            .map(|_| Minefield::random_start(&mut rng, 9, 9, 10))
            .collect::<Vec<_>>(),
    );
    let minefields16x16m40 = (
        "3:16x16,40",
        (0..10)
            .map(|_| Minefield::random_start(&mut rng, 16, 16, 40))
            .collect::<Vec<_>>(),
    );

    let bf = ("", &solve_bf as &dyn Fn(&mut Minefield));
    let pruning = ("pruning", &solve_pruning as &dyn Fn(&mut Minefield));
    let pruning_bm = ("pruning, bitmasks", &solve_bm as &dyn Fn(&mut Minefield));
    let pruning_bm_wtm = (
        "pruning, bitmasks, without total mines",
        &solve_bm_without_total_mines as &dyn Fn(&mut Minefield),
    );
    let chucking_4_3_wtm = (
        "pruning, chucking, without total mines",
        &(|mf: &mut Minefield| solve_chucking(mf, 4, 3)) as &dyn Fn(&mut Minefield),
    );

    bench_solver(&mut group, &minefields4x4m3, bf);
    bench_solver(&mut group, &minefields4x4m3, pruning);
    bench_solver(&mut group, &minefields4x4m3, pruning_bm);
    bench_solver(&mut group, &minefields4x4m3, pruning_bm_wtm);

    bench_solver(&mut group, &minefields6x6m5, pruning);
    bench_solver(&mut group, &minefields6x6m5, pruning_bm);
    bench_solver(&mut group, &minefields6x6m5, pruning_bm_wtm);
    bench_solver(&mut group, &minefields6x6m5, chucking_4_3_wtm);

    // bench_solver(&mut group, &minefields9x9m10, pruning_bm_wtm);
    // bench_solver(&mut group, &minefields9x9m10, chucking_4_3_wtm);

    bench_solver(&mut group, &minefields16x16m40, pruning_bm_wtm);
}

criterion_group!(benches, solver_benchmark);
criterion_main!(benches);
