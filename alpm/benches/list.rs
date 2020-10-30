use alpm::{Alpm, SigLevel};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_list(c: &mut Criterion) {
    c.bench_function("list", |b| {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb("core", SigLevel::NONE).unwrap();
        let pkg = db.pkg("linux").unwrap();

        b.iter(|| {
            black_box(&pkg.depends().into_iter().collect::<Vec<_>>());
        });
    });
}

criterion_group!(benches, benchmark_list);
criterion_main!(benches);
