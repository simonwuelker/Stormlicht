// Constants taken from stb_image
#![allow(clippy::excessive_precision)]

use super::quantization_table::QuantizationTable;

use std::num::Wrapping;

#[rustfmt::skip]
#[allow(dead_code)] // Only needed for encoder
pub const ORDER_TO_MATRIX_INDEX: [(i32, i32); 64] = [
    (0,0),
    (0,1), (1,0),         
    (2,0), (1,1), (0,2),
    (0,3), (1,2), (2,1), (3,0),
    (4,0), (3,1), (2,2), (1,3), (0,4),
    (0,5), (1,4), (2,3), (3,2), (4,1), (5,0),
    (6,0), (5,1), (4,2), (3,3), (2,4), (1,5), (0,6),
    (0,7), (1,6), (2,5), (3,4), (4,3), (5,2), (6,1), (7,0),
    (7,1), (6,2), (5,3), (4,4), (3,5), (2,6), (1,7),
    (2,7), (3,6), (4,5), (5,4), (6,3), (7,2),
    (7,3), (6,4), (5,5), (4,6), (3,7),
    (4,7), (5,6), (6,5), (7,4),
    (7,5), (6,6), (5,7),
    (6,7), (7,6),
    (7,7)
];

#[rustfmt::skip]
pub const MATRIX_INDEX_TO_ORDER: [[usize; 8]; 8] = [
    [ 0,  1,  5,  6, 14, 15, 27, 28],
    [ 2,  4,  7, 13, 16, 26, 29, 42],
    [ 3,  8, 12, 17, 25, 30, 41, 43],
    [ 9, 11, 18, 24, 31, 40, 44, 53],
    [10, 19, 23, 32, 39, 45, 52, 54],
    [20, 22, 33, 38, 46, 51, 55, 60],
    [21, 34, 37, 47, 50, 56, 59, 61],
    [35, 36, 48, 49, 57, 58, 62, 63],
];

#[inline]
#[must_use]
fn dequantize(coefficient: i16, quantization: u16) -> Wrapping<i32> {
    Wrapping(coefficient as i32) * Wrapping(quantization as i32)
}

pub fn dequantize_and_perform_idct(
    coefficients: &[i16; 64],
    quantization_table: &QuantizationTable,
    output: &mut [u8],
) {
    // Similar to stbi__idct_block in stb_image
    let mut tmp = [Wrapping(0); 64];

    // Columns
    for i in 0..8 {
        if coefficients[i + 8] == 0
            && coefficients[i + 16] == 0
            && coefficients[i + 32] == 0
            && coefficients[i + 40] == 0
            && coefficients[i + 48] == 0
            && coefficients[i + 56] == 0
        {
            let dc_term = dequantize(coefficients[i], quantization_table[i]) << 2;

            tmp[i] = dc_term;
            tmp[i + 8] = dc_term;
            tmp[i + 16] = dc_term;
            tmp[i + 24] = dc_term;
            tmp[i + 32] = dc_term;
            tmp[i + 40] = dc_term;
            tmp[i + 48] = dc_term;
            tmp[i + 56] = dc_term;
        } else {
            let s0 = dequantize(coefficients[i], quantization_table[i]);
            let s1 = dequantize(coefficients[i + 8], quantization_table[i + 8]);
            let s2 = dequantize(coefficients[i + 16], quantization_table[i + 16]);
            let s3 = dequantize(coefficients[i + 24], quantization_table[i + 24]);
            let s4 = dequantize(coefficients[i + 32], quantization_table[i + 32]);
            let s5 = dequantize(coefficients[i + 40], quantization_table[i + 40]);
            let s6 = dequantize(coefficients[i + 48], quantization_table[i + 48]);
            let s7 = dequantize(coefficients[i + 56], quantization_table[i + 56]);

            let [x0, x1, x2, x3] = make_kernel_x(s0, s2, s4, s6, Wrapping(512));
            let [t0, t1, t2, t3] = make_kernel_y(s1, s3, s5, s7);

            tmp[i] = (x0 + t3) >> 10;
            tmp[i + 8] = (x1 + t2) >> 10;
            tmp[i + 16] = (x2 + t1) >> 10;
            tmp[i + 24] = (x3 + t0) >> 10;
            tmp[i + 32] = (x3 - t0) >> 10;
            tmp[i + 40] = (x2 - t1) >> 10;
            tmp[i + 48] = (x1 - t2) >> 10;
            tmp[i + 56] = (x0 - t3) >> 10;
        }
    }

    let output = output.array_chunks_mut::<8>();
    for (out_row, tmp_row) in output.zip(tmp.array_chunks::<8>()) {
        // constants scaled things up by 1<<12, plus we had 1<<2 from first
        // loop, plus horizontal and vertical each scale by sqrt(8) so together
        // we've got an extra 1<<3, so 1<<17 total we need to remove.
        // so we want to round that, which means adding 0.5 * 1<<17,
        // aka 65536. Also, we'll end up with -128 to 127 that we want
        // to encode as 0..255 by adding 128, so we'll add that before the shift
        const X_SCALE: Wrapping<i32> = Wrapping(65536 + (128 << 17));

        let [s0, rest @ ..] = tmp_row;
        if *rest == [Wrapping(0); 7] {
            let dc_term = stbi_clamp((stbi_fsh(*s0) + X_SCALE) >> 17);
            out_row.fill(dc_term);
        } else {
            let ([x0, x1, x2, x3], [t0, t1, t2, t3]) = make_kernel(*tmp_row, X_SCALE);

            out_row[0] = stbi_clamp((x0 + t3) >> 17);
            out_row[1] = stbi_clamp((x1 + t2) >> 17);
            out_row[2] = stbi_clamp((x2 + t1) >> 17);
            out_row[3] = stbi_clamp((x3 + t0) >> 17);
            out_row[4] = stbi_clamp((x3 - t0) >> 17);
            out_row[5] = stbi_clamp((x2 - t1) >> 17);
            out_row[6] = stbi_clamp((x1 - t2) >> 17);
            out_row[7] = stbi_clamp((x0 - t3) >> 17);
        }
    }
}

#[inline]
#[must_use]
fn make_kernel(
    [s0, s1, s2, s3, s4, s5, s6, s7]: [Wrapping<i32>; 8],
    x_scale: Wrapping<i32>,
) -> ([Wrapping<i32>; 4], [Wrapping<i32>; 4]) {
    (
        make_kernel_x(s0, s2, s4, s6, x_scale),
        make_kernel_y(s1, s3, s5, s7),
    )
}

fn make_kernel_x(
    s0: Wrapping<i32>,
    s2: Wrapping<i32>,
    s4: Wrapping<i32>,
    s6: Wrapping<i32>,
    x_scale: Wrapping<i32>,
) -> [Wrapping<i32>; 4] {
    let p2 = s2;
    let p3 = s6;
    let p1 = (p2 + p3) * stbi_f2f(0.5411961);
    let t2 = p1 + p3 * stbi_f2f(-1.847759065);
    let t3 = p1 + p2 * stbi_f2f(0.765366865);

    let t0 = stbi_fsh(s0 + s4);
    let t1 = stbi_fsh(s0 - s4);

    let x0 = t0 + t3;
    let x3 = t0 - t3;
    let x1 = t1 + t2;
    let x2 = t1 - t2;

    [x0 + x_scale, x1 + x_scale, x2 + x_scale, x3 + x_scale]
}

#[inline]
#[must_use]
fn make_kernel_y(
    s1: Wrapping<i32>,
    s3: Wrapping<i32>,
    s5: Wrapping<i32>,
    s7: Wrapping<i32>,
) -> [Wrapping<i32>; 4] {
    let p1 = s7 + s1;
    let p2 = s5 + s3;
    let p3 = s7 + s3;
    let p4 = s5 + s1;
    let p5 = (p3 + p4) * stbi_f2f(1.175875602);

    let p1 = p5 + p1 * stbi_f2f(-0.899976223);
    let p2 = p5 + p2 * stbi_f2f(-2.562915447);
    let p3 = p3 * stbi_f2f(-1.961570560);
    let p4 = p4 * stbi_f2f(-0.390180644);

    let t0 = s7 * stbi_f2f(0.298631336) + p1 + p3;
    let t1 = s5 * stbi_f2f(2.053119869) + p2 + p4;
    let t2 = s3 * stbi_f2f(3.072711026) + p2 + p3;
    let t3 = s1 * stbi_f2f(1.501321110) + p1 + p4;

    [t0, t1, t2, t3]
}

#[inline]
#[must_use]
const fn stbi_f2f(x: f32) -> Wrapping<i32> {
    Wrapping((x * 4096.0 + 0.5) as i32)
}

#[inline]
#[must_use]
fn stbi_fsh(x: Wrapping<i32>) -> Wrapping<i32> {
    x << 12
}

#[inline]
#[must_use]
fn stbi_clamp(x: Wrapping<i32>) -> u8 {
    x.0.clamp(0, 255) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test case from
    // https://github.com/image-rs/jpeg-decoder/blob/master/src/idct.rs#L581
    #[test]
    fn test_dequantize_and_perform_idct() {
        #[rustfmt::skip]
        let coefficients: [i16; 8 * 8] = [
            -14, -39, 58, -2, 3, 3, 0, 1,
            11, 27, 4, -3, 3, 0, 1, 0,
            -6, -13, -9, -1, -2, -1, 0, 0,
            -4, 0, -1, -2, 0, 0, 0, 0,
            3, 0, 0, 0, 0, 0, 0, 0,
            -3, -2, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0
        ];

        #[rustfmt::skip]
        let quantization_table: [u16; 8 * 8] = [
            8, 6, 5, 8, 12, 20, 26, 31,
            6, 6, 7, 10, 13, 29, 30, 28,
            7, 7, 8, 12, 20, 29, 35, 28,
            7, 9, 11, 15, 26, 44, 40, 31,
            9, 11, 19, 28, 34, 55, 52, 39,
            12, 18, 28, 32, 41, 52, 57, 46,
            25, 32, 39, 44, 52, 61, 60, 51,
            36, 46, 48, 49, 56, 50, 52, 50
        ];

        let mut output = [0; 8 * 8];

        dequantize_and_perform_idct(&coefficients, &quantization_table, &mut output);

        #[rustfmt::skip]
        let expected_output = [
            118, 92, 110, 83, 77, 93, 144, 198,
            172, 116, 114, 87, 78, 93, 146, 191,
            194, 107, 91, 76, 71, 93, 160, 198,
            196, 100, 80, 74, 67, 92, 174, 209,
            182, 104, 88, 81, 68, 89, 178, 206,
            105, 64, 59, 59, 63, 94, 183, 201,
            35, 27, 28, 37, 72, 121, 203, 204,
            37, 45, 41, 47, 98, 154, 223, 208
        ];

        for i in 0..64 {
            println!("{} {}", output[i], expected_output[i]);
            assert!((output[i] as i16 - expected_output[i] as i16).abs() <= 1);
        }
    }
}
