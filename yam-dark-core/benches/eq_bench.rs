use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bool_to_u8(b: bool) -> u8 {
    if b {
        0xFF
    } else {
        0x00
    }
}

fn u8x16_eq2(a: [u8; 16], b: [u8; 16]) -> [u8; 16] {
    [
        bool_to_u8(a[0] == b[0]),
        bool_to_u8(a[1] == b[1]),
        bool_to_u8(a[2] == b[2]),
        bool_to_u8(a[3] == b[3]),
        bool_to_u8(a[4] == b[4]),
        bool_to_u8(a[5] == b[5]),
        bool_to_u8(a[6] == b[6]),
        bool_to_u8(a[7] == b[7]),
        bool_to_u8(a[8] == b[8]),
        bool_to_u8(a[9] == b[9]),
        bool_to_u8(a[10] == b[10]),
        bool_to_u8(a[11] == b[11]),
        bool_to_u8(a[12] == b[12]),
        bool_to_u8(a[13] == b[13]),
        bool_to_u8(a[14] == b[14]),
        bool_to_u8(a[15] == b[15]),
    ]
}

fn u8x16_eq(a: [u8; 16], cmp: u8) -> [u8; 16] {
    let x = [cmp; 16];
    [
        if a[0] == x[0] { 1 } else { 0 },
        if a[1] == x[1] { 1 } else { 0 },
        if a[2] == x[2] { 1 } else { 0 },
        if a[3] == x[3] { 1 } else { 0 },
        if a[4] == x[4] { 1 } else { 0 },
        if a[5] == x[5] { 1 } else { 0 },
        if a[6] == x[6] { 1 } else { 0 },
        if a[7] == x[7] { 1 } else { 0 },
        if a[8] == x[8] { 1 } else { 0 },
        if a[9] == x[9] { 1 } else { 0 },
        if a[10] == x[10] { 1 } else { 0 },
        if a[11] == x[11] { 1 } else { 0 },
        if a[12] == x[12] { 1 } else { 0 },
        if a[13] == x[13] { 1 } else { 0 },
        if a[14] == x[14] { 1 } else { 0 },
        if a[15] == x[15] { 1 } else { 0 },
    ]
}

fn u8x16_bit(a: [u8; 16]) -> u16 {
    (a[0] & 0b1000_0000 != 0) as u16
        | (((a[1] & 0b1000_0000 != 0) as u16) << 1)
        | (((a[2] & 0b1000_0000 != 0) as u16) << 2)
        | (((a[3] & 0b1000_0000 != 0) as u16) << 3)
        | (((a[4] & 0b1000_0000 != 0) as u16) << 4)
        | (((a[5] & 0b1000_0000 != 0) as u16) << 5)
        | (((a[6] & 0b1000_0000 != 0) as u16) << 6)
        | (((a[7] & 0b1000_0000 != 0) as u16) << 7)
        | (((a[8] & 0b1000_0000 != 0) as u16) << 8)
        | (((a[9] & 0b1000_0000 != 0) as u16) << 9)
        | (((a[10] & 0b1000_0000 != 0) as u16) << 10)
        | (((a[11] & 0b1000_0000 != 0) as u16) << 11)
        | (((a[12] & 0b1000_0000 != 0) as u16) << 12)
        | (((a[13] & 0b1000_0000 != 0) as u16) << 13)
        | (((a[14] & 0b1000_0000 != 0) as u16) << 14)
        | (((a[15] & 0b1000_0000 != 0) as u16) << 15)
}

fn u8x32_bit(a: [u8; 32]) -> u32 {
    (a[0] & 0b1000_0000 != 0) as u32
        | (((a[1] & 0b1000_0000 != 0) as u32) << 1)
        | (((a[2] & 0b1000_0000 != 0) as u32) << 2)
        | (((a[3] & 0b1000_0000 != 0) as u32) << 3)
        | (((a[4] & 0b1000_0000 != 0) as u32) << 4)
        | (((a[5] & 0b1000_0000 != 0) as u32) << 5)
        | (((a[6] & 0b1000_0000 != 0) as u32) << 6)
        | (((a[7] & 0b1000_0000 != 0) as u32) << 7)
        | (((a[8] & 0b1000_0000 != 0) as u32) << 8)
        | (((a[9] & 0b1000_0000 != 0) as u32) << 9)
        | (((a[10] & 0b1000_0000 != 0) as u32) << 10)
        | (((a[11] & 0b1000_0000 != 0) as u32) << 11)
        | (((a[12] & 0b1000_0000 != 0) as u32) << 12)
        | (((a[13] & 0b1000_0000 != 0) as u32) << 13)
        | (((a[14] & 0b1000_0000 != 0) as u32) << 14)
        | (((a[15] & 0b1000_0000 != 0) as u32) << 15)
        | (((a[16] & 0b1000_0000 != 0) as u32) << 16)
        | (((a[17] & 0b1000_0000 != 0) as u32) << 17)
        | (((a[18] & 0b1000_0000 != 0) as u32) << 18)
        | (((a[19] & 0b1000_0000 != 0) as u32) << 19)
        | (((a[20] & 0b1000_0000 != 0) as u32) << 20)
        | (((a[21] & 0b1000_0000 != 0) as u32) << 21)
        | (((a[22] & 0b1000_0000 != 0) as u32) << 22)
        | (((a[23] & 0b1000_0000 != 0) as u32) << 23)
        | (((a[24] & 0b1000_0000 != 0) as u32) << 24)
        | (((a[25] & 0b1000_0000 != 0) as u32) << 25)
        | (((a[26] & 0b1000_0000 != 0) as u32) << 26)
        | (((a[27] & 0b1000_0000 != 0) as u32) << 27)
        | (((a[28] & 0b1000_0000 != 0) as u32) << 28)
        | (((a[30] & 0b1000_0000 != 0) as u32) << 30)
        | (((a[31] & 0b1000_0000 != 0) as u32) << 31)
}

fn u8x32_eq(a: [u8; 32], cmp: u8) -> [u8; 32] {
    [
        if a[0] == cmp { 1 } else { 0 },
        if a[1] == cmp { 1 } else { 0 },
        if a[2] == cmp { 1 } else { 0 },
        if a[3] == cmp { 1 } else { 0 },
        if a[4] == cmp { 1 } else { 0 },
        if a[5] == cmp { 1 } else { 0 },
        if a[6] == cmp { 1 } else { 0 },
        if a[7] == cmp { 1 } else { 0 },
        if a[8] == cmp { 1 } else { 0 },
        if a[9] == cmp { 1 } else { 0 },
        if a[10] == cmp { 1 } else { 0 },
        if a[11] == cmp { 1 } else { 0 },
        if a[12] == cmp { 1 } else { 0 },
        if a[13] == cmp { 1 } else { 0 },
        if a[14] == cmp { 1 } else { 0 },
        if a[15] == cmp { 1 } else { 0 },
        if a[16] == cmp { 1 } else { 0 },
        if a[17] == cmp { 1 } else { 0 },
        if a[18] == cmp { 1 } else { 0 },
        if a[19] == cmp { 1 } else { 0 },
        if a[20] == cmp { 1 } else { 0 },
        if a[21] == cmp { 1 } else { 0 },
        if a[22] == cmp { 1 } else { 0 },
        if a[23] == cmp { 1 } else { 0 },
        if a[24] == cmp { 1 } else { 0 },
        if a[25] == cmp { 1 } else { 0 },
        if a[26] == cmp { 1 } else { 0 },
        if a[27] == cmp { 1 } else { 0 },
        if a[28] == cmp { 1 } else { 0 },
        if a[29] == cmp { 1 } else { 0 },
        if a[30] == cmp { 1 } else { 0 },
        if a[31] == cmp { 1 } else { 0 },
    ]
}

fn u8x64_eq(a: [u8; 64], cmp: u8) -> u64 {
    (if a[0] == cmp { 1 } else { 0 })
        | (if a[1] == cmp { 1 << 1 } else { 0 })
        | (if a[2] == cmp { 1 << 2 } else { 0 })
        | (if a[3] == cmp { 1 << 3 } else { 0 })
        | (if a[4] == cmp { 1 << 4 } else { 0 })
        | (if a[5] == cmp { 1 << 5 } else { 0 })
        | (if a[6] == cmp { 1 << 6 } else { 0 })
        | (if a[7] == cmp { 1 << 7 } else { 0 })
        | (if a[8] == cmp { 1 << 8 } else { 0 })
        | (if a[9] == cmp { 1 << 9 } else { 0 })
        | (if a[10] == cmp { 1 << 10 } else { 0 })
        | (if a[11] == cmp { 1 << 11 } else { 0 })
        | (if a[12] == cmp { 1 << 12 } else { 0 })
        | (if a[13] == cmp { 1 << 13 } else { 0 })
        | (if a[14] == cmp { 1 << 14 } else { 0 })
        | (if a[15] == cmp { 1 << 15 } else { 0 })
        | (if a[16] == cmp { 1 << 16 } else { 0 })
        | (if a[17] == cmp { 1 << 17 } else { 0 })
        | (if a[18] == cmp { 1 << 18 } else { 0 })
        | (if a[19] == cmp { 1 << 19 } else { 0 })
        | (if a[20] == cmp { 1 << 20 } else { 0 })
        | (if a[21] == cmp { 1 << 21 } else { 0 })
        | (if a[22] == cmp { 1 << 22 } else { 0 })
        | (if a[23] == cmp { 1 << 23 } else { 0 })
        | (if a[24] == cmp { 1 << 24 } else { 0 })
        | (if a[25] == cmp { 1 << 25 } else { 0 })
        | (if a[26] == cmp { 1 << 26 } else { 0 })
        | (if a[27] == cmp { 1 << 27 } else { 0 })
        | (if a[28] == cmp { 1 << 28 } else { 0 })
        | (if a[29] == cmp { 1 << 29 } else { 0 })
        | (if a[30] == cmp { 1 << 30 } else { 0 })
        | (if a[31] == cmp { 1 << 31 } else { 0 })
        | (if a[32] == cmp { 1 << 32 } else { 0 })
        | (if a[33] == cmp { 1 << 33 } else { 0 })
        | (if a[34] == cmp { 1 << 34 } else { 0 })
        | (if a[35] == cmp { 1 << 35 } else { 0 })
        | (if a[36] == cmp { 1 << 36 } else { 0 })
        | (if a[37] == cmp { 1 << 37 } else { 0 })
        | (if a[38] == cmp { 1 << 38 } else { 0 })
        | (if a[39] == cmp { 1 << 39 } else { 0 })
        | (if a[40] == cmp { 1 << 40 } else { 0 })
        | (if a[41] == cmp { 1 << 41 } else { 0 })
        | (if a[42] == cmp { 1 << 42 } else { 0 })
        | (if a[43] == cmp { 1 << 43 } else { 0 })
        | (if a[44] == cmp { 1 << 44 } else { 0 })
        | (if a[45] == cmp { 1 << 45 } else { 0 })
        | (if a[46] == cmp { 1 << 46 } else { 0 })
        | (if a[47] == cmp { 1 << 47 } else { 0 })
        | (if a[48] == cmp { 1 << 48 } else { 0 })
        | (if a[49] == cmp { 1 << 49 } else { 0 })
        | (if a[50] == cmp { 1 << 50 } else { 0 })
        | (if a[51] == cmp { 1 << 51 } else { 0 })
        | (if a[52] == cmp { 1 << 52 } else { 0 })
        | (if a[53] == cmp { 1 << 53 } else { 0 })
        | (if a[54] == cmp { 1 << 54 } else { 0 })
        | (if a[55] == cmp { 1 << 55 } else { 0 })
        | (if a[56] == cmp { 1 << 56 } else { 0 })
        | (if a[57] == cmp { 1 << 57 } else { 0 })
        | (if a[58] == cmp { 1 << 58 } else { 0 })
        | (if a[59] == cmp { 1 << 59 } else { 0 })
        | (if a[60] == cmp { 1 << 60 } else { 0 })
        | (if a[61] == cmp { 1 << 61 } else { 0 })
        | (if a[62] == cmp { 1 << 62 } else { 0 })
        | (if a[63] == cmp { 1 << 63 } else { 0 })
}

fn u8x16_bit_iter(a: [u8; 16], c: u8) -> u16 {
    a.iter().fold(0u16, move |b, x| {
        let m = b << 1;
        let z = if *x == c { 1 } else { 0 };
        m | z
    })
}

fn array_iter(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench-array");
    let rand_bytes: [u8; 16] = rand::random();
    group.significance_level(0.05).sample_size(100);
    group.bench_function("array_iter", |b| {
        b.iter(|| {
            let x1 = u8x16_bit_iter(rand_bytes, 3);
            let x2 = u8x16_bit_iter(rand_bytes, 5);
            black_box(x1 | x2);
        });
    });
    group.finish();
}

fn array_bit(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench-array");
    let rand_bytes: [u8; 16] = rand::random();
    group.significance_level(0.05).sample_size(100);
    group.bench_function("array_bit", |b| {
        b.iter(|| {
            let x1 = u8x16_bit(u8x16_eq(rand_bytes, 3));
            let x2 = u8x16_bit(u8x16_eq(rand_bytes, 5));
            black_box(x1 | x2);
        });
    });
    group.finish();
}

fn array_bit16(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench-array");
    let rand_bytes: [u8; 16] = rand::random();
    group.significance_level(0.05).sample_size(100);
    group.bench_function("array_bit16", |b| {
        b.iter(|| {
            let x1 = u8x16_bit(u8x16_eq2(rand_bytes, [3; 16]));
            let x2 = u8x16_bit(u8x16_eq2(rand_bytes, [5; 16]));
            black_box(x1 | x2);
        });
    });
    group.finish();
}

fn array_bit32(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench-array");
    let rand_bytes: [u8; 32] = rand::random();

    group.significance_level(0.05).sample_size(100);
    group.bench_function("array_bit32", |b| {
        b.iter(|| {
            let x1 = u8x32_bit(u8x32_eq(rand_bytes, 3));
            let x2 = u8x32_bit(u8x32_eq(rand_bytes, 5));
            black_box(x1 | x2);
        });
    });
    group.finish();
}

fn array_bit64(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench-array");
    let rand_bytes: [u8; 64] = rand::random();

    group.significance_level(0.05).sample_size(100);
    group.bench_function("array_bit64", |b| {
        b.iter(|| {
            let x1 = u8x64_eq(rand_bytes, 3);
            let x2 = u8x64_eq(rand_bytes, 3);
            black_box(x1 | x2);
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    array_iter,
    array_bit,
    array_bit16,
    array_bit32,
    array_bit64
);
criterion_main!(benches);
