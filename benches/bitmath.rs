use criterion::{criterion_group, criterion_main, Criterion};
use siegfried::bitboard::Bitboard;
use siegfried::bitboard::BitboardMethods;
use siegfried::maps::*;
use siegfried::position::Position;
use siegfried::types::*;

pub fn criterion_benchmark(c: &mut Criterion) {

    let position = Position::new();

    //load maps first
    //load_maps();

    //run first eval


}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);