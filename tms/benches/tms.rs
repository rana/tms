use criterion::{criterion_group, criterion_main, Criterion};
use tms;
use tms::dat::goog;
use tms::vrn;
use usize;

// To install criterion for cargo:
//  cargo install cargo-criterion

// To benchmark with cargo:
//  cargo criterion

pub fn bench_vrn(c: &mut Criterion) {
  let mut g = c.benchmark_group("vrn");

  g.bench_function("usize_byt_len", |b| {
    let v = usize::pow(2, 8*7)-1;
    b.iter(|| {
      // [4.3354 ns 4.3633 ns 4.3918 ns]
      vrn::usize_byt_len(v)
    });
  });
  g.bench_function("usize_pck", |b| {
    let v = usize::pow(2, 8*7)-1;
    let mut pck = vec![0u8; 8];
    b.iter(|| {
      // [6.7060 ns 6.7354 ns 6.7666 ns]
      vrn::usize_pck(v, &mut pck);
    });
  });
  // g.bench_function("usize_unp", |b| {
  //   let v = usize::pow(2, 8*7)-1;
  //   let mut pck = vec![0u8; 8];
  //   b.iter(|| {
  //     // [6.7060 ns 6.7354 ns 6.7666 ns]
  //     vrn::usize_pck(v, &mut pck);
  //   });
  // });

  // g.bench_function("u32s_byt_len", |b| {
  //   let blk = vec![0, 1, 128, 16384, 2097152];
  //   b.iter(|| {
  //     vrn::u32s_byt_len(&blk)
  //   });
  // });


}

pub fn bench_smd(c: &mut Criterion) {
  let mut g = c.benchmark_group("tms");

  g.bench_function("u32x256_bit_len", |b| {
    let blk = goog::blk256();
    b.iter(|| {
      unsafe {
        // [41.726 ns 41.769 ns 41.820 ns]
        tms::u32x256_bit_len(&blk)
      }
    });
  });

  g.bench_function("day_u32x256_pck", |b| {
    let unp_exp = goog::day();
    let day = tms::DayLen::u32x256(&unp_exp);
    let mut pck = vec![0u8; day.byt_len];
    b.iter(|| {
      // [78.112 us 78.408 us 78.730 us]
      tms::day_u32x256_pck(&day, &unp_exp, &mut pck);
    });
  });
  g.bench_function("day_u32x256_unp", |b| {
    let unp_exp = goog::day();
    let mut unp_act = vec![0u32; unp_exp.len()];
    let day = tms::DayLen::u32x256(&unp_exp);
    let mut pck = vec![0u8; day.byt_len];
      tms::day_u32x256_pck(&day, &unp_exp, &mut pck);
    b.iter(|| {
      // [101.12 us 101.39 us 101.71 us]
      tms::day_u32x256_unp(&pck, &mut unp_act);
    });
  });

}



criterion_group!(benches, bench_vrn, bench_smd);
criterion_main!(benches);
