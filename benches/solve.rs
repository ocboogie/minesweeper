use minesweeper::minefield::Minefield;
use minesweeper::solver::solve;

use criterion::{criterion_group, criterion_main, Criterion};

const _25X25_100: &str = r#".mm....m............m.m..
m..m...m..........m...m..
..........m.......m......
..............m....m.....
..m...m................m.
.m.m.............m....m.m
...........mmm.m.m.......
...m.......00000m........
mm.....mmm.0000000m..m..m
...m...m..m0000000.mmm...
....m...mm00000000...m...
.m.......m0000000m.......
....m...m.0000000...m....
...........mm0000....m..m
.........m.....m...m....m
..m..............m.......
....m.m........m.........
m..............m.........
.....m.......m...........
m....m...m.....m.m.m...m.
m..............m...m...m.
.........m....m.........m
..........m.m.......m....
..m......m.m..m..........
..m......mm...mm.mmm.m..."#;

const _50X50_390: &str = r#"............m...m..............m...........m......
..m.............mm.....................m.m........
............m.................m...........m.....mm
m..m...............mm....mm.......m..m.....m.m....
....mm.m................................m.m.......
..........m.....m......m......m.m..m...m..........
....m.mm..........m.....m.m........m........m.....
m.m...m..m.........m...........m..m...m...m.m....m
...............m......................m...........
...........m......m...mm.m..m...m.................
...m....m..........m.........m..m..........m.mm.mm
..mm...........m...mm.................m..........m
...m..m.m....m.....mm..............m.....m..m.....
.m....m..m......m....m............mm............m.
......m........m...mm..m.m...............mm...m...
.................m.........m.......m.....m........
mm......m.........................m..............m
...mm.m....m.........m....mm.m...mm.......m.......
...m...m.......mm......m......mm.......m...m.m.m..
.......mm..m.....m.................m..............
..............m....m......m.m.......mmm....m......
......m...................m.......mm..............
....m.....m.......m..............m................
....m...m.m.......mm......m..............mm.......
.m.m....m...m..m...........m.m....m.mm...m..m.....
mmm...mm...............m..000m...mm.........m....m
.....m......m.m....mm....m00000m..m.m..m.......m..
....m..................m.000000m.m.........m......
...................mm.m.m00000000.....m.m...m.....
m.........m..m.....m.....00000000m........m.......
....mm........m.m.m.m....m..m0000m....m.....m..mmm
.m.................m.........0000..........m.m....
....m.m..m..m....m.m.........0000...m..m...m.m....
........m......m..m....m..m..mm.m......m..........
...m..mm...m.m.m....m..mm..........m.m............
....m......mmmm.......m..............m...m.m..m...
.....................m...........m....m..m.m.m....
m....m.........m........mmm........m.mm.m.........
...m....m......m............m..m.....m.m.....m....
..m.................m..........m..............m...
.....m...m......m....m..m....m.......m........m..m
.......m.m..m..........m...m.........m.m..m...m...
m.......mm.....m.....m.mm.....m..m..m.......m.....
...m.mm....m.m.m..m.mm.............m..mm..........
.......m.mm.............m...........m.mm....m..m.m
.........m..m............................m...mmm..
.m........m..m..m.m......m........m...............
........m.....m..m.m...............m..............
.m.....m..m..m......m..........mm.....m...m...m.mm
........m........m....m........................m.."#;

pub fn solver_benchmark(c: &mut Criterion) {
    let mf_50x50_390 = Minefield::parse(_50X50_390);
    let _25x25_100 = Minefield::parse(_25X25_100);

    // c.bench_function("solver 50x50, 390", |b| b.iter(|| solve(&mf_50x50_390)));
    c.bench_function("solver 25x25, 100", |b| b.iter(|| solve(&_25x25_100)));
}

criterion_group!(benches, solver_benchmark);
criterion_main!(benches);