use criterion::{criterion_group, criterion_main, Criterion};
use siegfried::maps::*;
use siegfried::position::Position;
use siegfried::types::*;

pub fn criterion_benchmark(c: &mut Criterion) {

    let position = Position::from_fen("1k1r3r/pppqb1pp/1nn1p3/3bPp2/1P1PN3/P2BBN2/5PPP/2RQ1RK1 w - f6 0 15");
    
    c.bench_function("get_ray_between_squares", |b| b.iter(|| get_ray_between_squares(Square::E4, Square::E8)));

    c.bench_function("position_eval", |b| b.iter(|| position.evaluate()));

    //load maps first
    //load_maps();

    //run first eval


}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);