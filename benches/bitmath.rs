use criterion::{criterion_group, criterion_main, Criterion};
use siegfried::bitboard::Bitboard;
use siegfried::bitboard::BitboardMethods;
use siegfried::maps::*;
use siegfried::types::*;

pub fn criterion_benchmark(c: &mut Criterion) {

    //map generation
    c.bench_function("Knight attacks", |b| b.iter(|| get_knight_attack_map()));
    
    //some occupancy and square
    let square = Square::A1;
    let occupancy: Bitboard = 0.set_bit(Square::A2.into()).set_bit(Square::B1.into()).set_bit(Square::C1.into());

    //bench rook
    c.bench_function("Rook attacks", |b| b.iter(|| get_rook_attacks(square, occupancy)));
    
    //bench bishop
    c.bench_function("Bishop attacks", |b| b.iter(|| get_bishop_attacks(square, occupancy)));
    
    //bench queen
    c.bench_function("Queen attacks", |b| b.iter(|| get_queen_attacks(square, occupancy)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);