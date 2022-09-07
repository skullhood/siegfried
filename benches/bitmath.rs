use criterion::{criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {

    //Map generation
    //c.bench_function("Knight attacks", |b| b.iter(|| get_knight_attack_map()));
    //c.bench_function("Bishop attacks", |b| b.iter(|| get_bishop_attack_map()));
    //c.bench_function("Rook attacks", |b| b.iter(|| get_rook_attack_map()));
    //c.bench_function("Queen attacks", |b| b.iter(|| get_queen_attack_map()));


}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);