use std::ops::Shr;

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use rand::prelude::*;

use yam_dark_core::util::U8X16;
use yam_dark_core::YamlChunkState;
use yam_dark_core::{util, NativeScanner, Stage1Scanner, HIGH_NIBBLE_MASK, LOW_NIBBLE_MASK};

unsafe fn find_whitespace_and_structurals(
    input: [u8; 64],
    whitespace: &mut u64,
    structurals: &mut u64,
) {
    let v0 = unsafe { from_slice(&input[0..16]) };
    let v1 = unsafe { from_slice(&input[16..32]) };
    let v2 = unsafe { from_slice(&input[32..48]) };
    let v3 = unsafe { from_slice(&input[48..64]) };

    let structural_shufti_mask: [u8; 16] = [0x7; 16];
    let whitespace_shufti_mask: [u8; 16] = [0x18; 16];
    let low_nib_and_mask: [u8; 16] = [0xF; 16]; //8x16_splat(0xf);
    let high_nib_and_mask: [u8; 16] = [0x7F; 16]; //u8x16_splat(0x7f);
    let zero_mask: [u8; 16] = [0x0; 16];

    let v_v0 = v128_and(
        u8x16_swizzle(LOW_NIBBLE_MASK, v128_and(v0, low_nib_and_mask)),
        u8x16_swizzle(
            HIGH_NIBBLE_MASK,
            v128_and(u8x16_shr(v0, 4), high_nib_and_mask),
        ),
    );
    let v_v1 = v128_and(
        u8x16_swizzle(LOW_NIBBLE_MASK, v128_and(v1, low_nib_and_mask)),
        u8x16_swizzle(
            HIGH_NIBBLE_MASK,
            v128_and(u8x16_shr(v1, 4), high_nib_and_mask),
        ),
    );
    let v_v2 = v128_and(
        u8x16_swizzle(LOW_NIBBLE_MASK, v128_and(v2, low_nib_and_mask)),
        u8x16_swizzle(
            HIGH_NIBBLE_MASK,
            v128_and(u8x16_shr(v2, 4), high_nib_and_mask),
        ),
    );
    let v_v3 = v128_and(
        u8x16_swizzle(LOW_NIBBLE_MASK, v128_and(v3, low_nib_and_mask)),
        u8x16_swizzle(
            HIGH_NIBBLE_MASK,
            v128_and(u8x16_shr(v3, 4), high_nib_and_mask),
        ),
    );
    let tmp_v0 = u8x16_eq(v128_and(v_v0, structural_shufti_mask), zero_mask);
    let tmp_v1 = u8x16_eq(v128_and(v_v1, structural_shufti_mask), zero_mask);
    let tmp_v2 = u8x16_eq(v128_and(v_v2, structural_shufti_mask), zero_mask);
    let tmp_v3 = u8x16_eq(v128_and(v_v3, structural_shufti_mask), zero_mask);

    let structural_res_0 = u8x16_bitmask(tmp_v0) as u64;
    let structural_res_1 = u8x16_bitmask(tmp_v1) as u64;
    let structural_res_2 = u8x16_bitmask(tmp_v2) as u64;
    let structural_res_3 = u8x16_bitmask(tmp_v3) as u64;

    *structurals = !(structural_res_0
        | (structural_res_1 << 16)
        | (structural_res_2 << 32)
        | (structural_res_3 << 48));

    let tmp_ws_v0 = u8x16_eq(v128_and(v_v0, whitespace_shufti_mask), zero_mask);
    let tmp_ws_v1 = u8x16_eq(v128_and(v_v1, whitespace_shufti_mask), zero_mask);
    let tmp_ws_v2 = u8x16_eq(v128_and(v_v2, whitespace_shufti_mask), zero_mask);
    let tmp_ws_v3 = u8x16_eq(v128_and(v_v3, whitespace_shufti_mask), zero_mask);

    let ws_res_0 = u8x16_bitmask(tmp_ws_v0) as u64;
    let ws_res_1 = u8x16_bitmask(tmp_ws_v1) as u64;
    let ws_res_2 = u8x16_bitmask(tmp_ws_v2) as u64;
    let ws_res_3 = u8x16_bitmask(tmp_ws_v3) as u64;

    *whitespace = !(ws_res_0 | (ws_res_1 << 16) | (ws_res_2 << 32) | (ws_res_3 << 48));
}

unsafe fn find_whitespace_and_structurals_u8x16(
    input: [u8; 64],
    whitespace: &mut u64,
    structurals: &mut u64,
) {
    let v0 = unsafe { U8X16::from_slice(&input[0..16]) };
    let v1 = unsafe { U8X16::from_slice(&input[16..32]) };
    let v2 = unsafe { U8X16::from_slice(&input[32..48]) };
    let v3 = unsafe { U8X16::from_slice(&input[48..64]) };

    let structural_shufti_mask: [u8; 16] = [0x7; 16];
    let whitespace_shufti_mask: [u8; 16] = [0x18; 16];
    let low_nib_and_mask: [u8; 16] = [0xF; 16]; //8x16_splat(0xf);
    let high_nib_and_mask: [u8; 16] = [0x7F; 16]; //u8x16_splat(0x7f);

    let v_v0 = (util::u8x16_swizzle(LOW_NIBBLE_MASK, v0 & low_nib_and_mask))
        & (util::u8x16_swizzle(HIGH_NIBBLE_MASK, v0.shr(4) & high_nib_and_mask));

    let v_v1 = (util::u8x16_swizzle(LOW_NIBBLE_MASK, v1 & low_nib_and_mask))
        & (util::u8x16_swizzle(HIGH_NIBBLE_MASK, v1.shr(4) & high_nib_and_mask));

    let v_v2 = (util::u8x16_swizzle(LOW_NIBBLE_MASK, v2 & low_nib_and_mask))
        & (util::u8x16_swizzle(HIGH_NIBBLE_MASK, v2.shr(4) & high_nib_and_mask));

    let v_v3 = (util::u8x16_swizzle(LOW_NIBBLE_MASK, v3 & low_nib_and_mask))
        & (util::u8x16_swizzle(HIGH_NIBBLE_MASK, v3.shr(4) & high_nib_and_mask));

    let tmp_v0 = (v_v0 & 0x7).comp_all(0);
    let tmp_v1 = (v_v1 & 0x7).comp_all(0);
    let tmp_v2 = (v_v2 & 0x7).comp_all(0);
    let tmp_v3 = (v_v3 & 0x7).comp_all(0);
    //
    let structural_res_0 = tmp_v0.to_u16() as u64;
    let structural_res_1 = tmp_v1.to_u16() as u64;
    let structural_res_2 = tmp_v2.to_u16() as u64;
    let structural_res_3 = tmp_v3.to_u16() as u64;

    *structurals = !(structural_res_0
        | (structural_res_1 << 16)
        | (structural_res_2 << 32)
        | (structural_res_3 << 48));

    let tmp_ws0 = (v_v0 & 0x17).comp_all(0);
    let tmp_ws1 = (v_v1 & 0x17).comp_all(0);
    let tmp_ws2 = (v_v2 & 0x17).comp_all(0);
    let tmp_ws3 = (v_v3 & 0x17).comp_all(0);

    let ws_res_0 = tmp_ws0.to_u16() as u64;
    let ws_res_1 = tmp_ws1.to_u16() as u64;
    let ws_res_2 = tmp_ws2.to_u16() as u64;
    let ws_res_3 = tmp_ws3.to_u16() as u64;

    *whitespace = !(ws_res_0 | (ws_res_1 << 16) | (ws_res_2 << 32) | (ws_res_3 << 48));
}

fn u8x16_bitmask(a: [u8; 16]) -> u16 {
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

fn u8x16_eq(a: [u8; 16], b: [u8; 16]) -> [u8; 16] {
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

fn bool_to_u8(b: bool) -> u8 {
    if b {
        0xFF
    } else {
        0x00
    }
}

fn u8x16_shr(a: [u8; 16], n: i32) -> [u8; 16] {
    [
        a[0] >> n,
        a[1] >> n,
        a[2] >> n,
        a[3] >> n,
        a[4] >> n,
        a[5] >> n,
        a[6] >> n,
        a[7] >> n,
        a[8] >> n,
        a[9] >> n,
        a[10] >> n,
        a[11] >> n,
        a[12] >> n,
        a[13] >> n,
        a[14] >> n,
        a[15] >> n,
    ]
}

fn v128_and(a: [u8; 16], b: [u8; 16]) -> [u8; 16] {
    [
        a[0] & b[0],
        a[1] & b[1],
        a[2] & b[2],
        a[3] & b[3],
        a[4] & b[4],
        a[5] & b[5],
        a[6] & b[6],
        a[7] & b[7],
        a[8] & b[8],
        a[9] & b[9],
        a[10] & b[10],
        a[11] & b[11],
        a[12] & b[12],
        a[13] & b[13],
        a[14] & b[14],
        a[15] & b[15],
    ]
}

fn u8x16_swizzle(a: [u8; 16], s: [u8; 16]) -> [u8; 16] {
    [
        if s[0] > 0x0f {
            0
        } else {
            a[(s[0] & 0x0f) as usize]
        },
        if s[1] > 0x0f {
            0
        } else {
            a[(s[1] & 0x0f) as usize]
        },
        if s[2] > 0x0f {
            0
        } else {
            a[(s[2] & 0x0f) as usize]
        },
        if s[3] > 0x0f {
            0
        } else {
            a[(s[3] & 0x0f) as usize]
        },
        if s[4] > 0x0f {
            0
        } else {
            a[(s[4] & 0x0f) as usize]
        },
        if s[5] > 0x0f {
            0
        } else {
            a[(s[5] & 0x0f) as usize]
        },
        if s[6] > 0x0f {
            0
        } else {
            a[(s[6] & 0x0f) as usize]
        },
        if s[7] > 0x0f {
            0
        } else {
            a[(s[7] & 0x0f) as usize]
        },
        if s[8] > 0x0f {
            0
        } else {
            a[(s[8] & 0x0f) as usize]
        },
        if s[9] > 0x0f {
            0
        } else {
            a[(s[9] & 0x0f) as usize]
        },
        if s[10] > 0x0f {
            0
        } else {
            a[(s[10] & 0x0f) as usize]
        },
        if s[11] > 0x0f {
            0
        } else {
            a[(s[11] & 0x0f) as usize]
        },
        if s[12] > 0x0f {
            0
        } else {
            a[(s[12] & 0x0f) as usize]
        },
        if s[13] > 0x0f {
            0
        } else {
            a[(s[13] & 0x0f) as usize]
        },
        if s[14] > 0x0f {
            0
        } else {
            a[(s[14] & 0x0f) as usize]
        },
        if s[15] > 0x0f {
            0
        } else {
            a[(s[15] & 0x0f) as usize]
        },
    ]
}

#[inline]
unsafe fn from_slice(input: &[u8]) -> [u8; 16] {
    [
        *input.get_unchecked(0),
        *input.get_unchecked(1),
        *input.get_unchecked(2),
        *input.get_unchecked(3),
        *input.get_unchecked(4),
        *input.get_unchecked(5),
        *input.get_unchecked(6),
        *input.get_unchecked(7),
        *input.get_unchecked(8),
        *input.get_unchecked(9),
        *input.get_unchecked(10),
        *input.get_unchecked(11),
        *input.get_unchecked(12),
        *input.get_unchecked(13),
        *input.get_unchecked(14),
        *input.get_unchecked(15),
    ]
}

fn bench_simd_json(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench-swizzle");
    let rand_bytes: [u8; 64] = StdRng::seed_from_u64(42).gen();
    let mut chunk = YamlChunkState::default();

    group.significance_level(0.05).sample_size(100);
    group.throughput(Throughput::Bytes(rand_bytes.len() as u64));
    group.bench_function("bench-simd-json", |b| {
        b.iter(|| unsafe {
            find_whitespace_and_structurals(
                rand_bytes,
                &mut chunk.characters.spaces,
                &mut chunk.characters.op,
            );
            black_box(chunk.characters.spaces | chunk.characters.op);
        });
    });
    group.finish();
}

fn bench_yam_u8x16(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench-swizzle");
    let rand_bytes: [u8; 64] = StdRng::seed_from_u64(42).gen();
    let mut chunk = YamlChunkState::default();

    group.significance_level(0.05).sample_size(100);
    group.throughput(Throughput::Bytes(rand_bytes.len() as u64));
    group.bench_function("bench-yam-u8x16", |b| {
        b.iter(|| unsafe {
            find_whitespace_and_structurals_u8x16(
                rand_bytes,
                &mut chunk.characters.spaces,
                &mut chunk.characters.op,
            );
            black_box(chunk.characters.spaces | chunk.characters.op);
        });
    });
    group.finish();
}

fn bench_yam(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench-swizzle");
    let rand_bytes: [u8; 64] = StdRng::seed_from_u64(42).gen();
    let scanner = NativeScanner::from_chunk(&rand_bytes);
    let chunk = &mut Default::default();

    group.significance_level(0.05).sample_size(100);
    group.throughput(Throughput::Bytes(rand_bytes.len() as u64));
    group.bench_function("bench-dark-yam", |b| {
        b.iter(|| {
            scanner.scan_whitespace_and_structurals(chunk);
        });
    });
    group.finish();
}

criterion_group!(benches, bench_yam, bench_simd_json, bench_yam_u8x16);
criterion_main!(benches);
