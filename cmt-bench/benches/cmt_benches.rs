use cmt_concurrent::CartesianMerkleTree as ConcurrentCMT;
use cmt_core::CartesianMerkleTree as SequentialCMT;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;

fn generate_key(i: usize) -> [u8; 32] {
    let mut key = [0u8; 32];
    key[0..8].copy_from_slice(&i.to_be_bytes());
    key
}

fn bench_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("CMT Insert");

    group.bench_function("Sequential", |b| {
        b.iter(|| {
            let mut cmt = SequentialCMT::new();
            for i in 0..100000 {
                cmt.insert(generate_key(i), vec![i as u8]);
            }
        });
    });

    group.bench_function("Concurrent", |b| {
        let keys: Vec<_> = (0..100000).map(generate_key).collect();
        b.iter(|| {
            let cmt = ConcurrentCMT::new();
            keys.par_iter().for_each(|key| {
                cmt.insert(*key, vec![0u8]);
            });
        });
    });

    group.finish();
}

fn bench_generate_proof(c: &mut Criterion) {
    let mut group = c.benchmark_group("CMT Generate Proof");

    let mut cmt_seq = SequentialCMT::new();
    for i in 0..100000 {
        cmt_seq.insert(generate_key(i), vec![i as u8]);
    }

    let cmt_conc = ConcurrentCMT::new();
    for i in 0..100000 {
        cmt_conc.insert(generate_key(i), vec![i as u8]);
    }

    let keys: Vec<_> = (0..100000).map(generate_key).collect();

    group.bench_function("Sequential", |b| {
        b.iter(|| {
            for key in &keys {
                cmt_seq.generate_proof(key);
            }
        });
    });

    group.bench_function("Concurrent", |b| {
        b.iter(|| {
            keys.par_iter().for_each(|key| {
                cmt_conc.generate_proof(key);
            });
        });
    });

    group.finish();
}

criterion_group!(benches, bench_insert, bench_generate_proof);
criterion_main!(benches);
