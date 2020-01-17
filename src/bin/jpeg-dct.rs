use rustdct::{DCT2, DCTplanner};
use rustdct::algorithm::Type2And3Naive;
use image::{GenericImageView, ImageBuffer, Luma};
use rayon::slice::{ParallelSlice, ParallelSliceMut};

use std::cmp::Ordering;
use std::f32::consts::{PI, SQRT_2};
use rayon::iter::{IndexedParallelIterator, ParallelIterator};

fn main() {
    std::fs::create_dir_all("jpeg-dct").unwrap();

    let image_path = std::env::args().nth(1).expect("expected image path");
    let image = image::open(&image_path)
        .expect(&format!("failed to open {}", image_path));

    let luma = image.to_luma();

    luma.save("jpeg-dct/luma.png").unwrap();

    let data = luma.iter().map(|&l| l as f32).collect::<Vec<_>>();

    let mut planner = DCTplanner::new();
    let width = image.width();
    let height = image.height();

    let dct_data = dct_2d(&mut planner, data, width, height);

    let dc_offset_image = ImageBuffer::<Luma<_>, _>::from_pixel(width, height, Luma([dct_data[0] as u16 as u8]));

    dc_offset_image.save("jpeg-dct/dc-offset.png").unwrap();

    let inverse_dct = inverse_dct_2d(&mut planner, dct_data.clone(), width, height);

    let inversed_dct = ImageBuffer::<Luma<_>, _>::from_vec(
        width,
        height,
        inverse_dct.iter().map(|&d| d as u16 as u8).collect()
    ).unwrap();

    inversed_dct.save("jpeg-dct/inversed.png").unwrap();

    for &pct in [0.001953125, /*0.00390625, 0.0078125, 0.015625, 0.03125, 0.0625, 0.125, 0.25*/].iter() {
        let crop_width = (width as f32 * pct) as usize;
        let crop_height = (height as f32 * pct) as usize;
        let mut dct_data = dct_data.clone();

        for (y, chunk) in dct_data.chunks_mut(width as usize).enumerate() {
            if y < crop_height {
                for px in &mut chunk[crop_width..] {
                    *px = 0.0;
                }
            } else {
                for px in chunk {
                    *px = 0.0;
                }
            }
        }

        let inverse_dct = inverse_dct_2d(&mut planner, dct_data, width, height);

        let inversed_dct = ImageBuffer::<Luma<_>, _>::from_vec(
            width,
            height,
            inverse_dct.iter().map(|&d| d as u16 as u8).collect()
        ).unwrap();

        inversed_dct.save(format!("jpeg-dct/cropped-{}.png", pct * 100.0)).unwrap();
    }
}

// perform a DCT-II on rows of the data and then the columns (2D DCT-II)
fn dct_2d(planner: &mut DCTplanner<f32>, mut data: Vec<f32>, width: u32, height: u32) -> Vec<f32> {
    let (width, height) = (width as usize, height as usize);

    assert_eq!(data.len(), width * height);

//    let hdct = Type2And3Naive::new(width);
//    let vdct = Type2And3Naive::new(height);

    let mut scratch = vec![0f32; width * height];

    data.par_chunks(width).zip(scratch.par_chunks_mut(width))
        .for_each(|(row, row_out)| dct_ii(row, row_out));

//    for (row, row_out) in data.chunks_mut(width).zip(scratch.chunks_mut(width)) {
//        dct_ii(row, row_out);
//    }

    // swap rows and columns
    transpose::transpose(&scratch, &mut data, width, height);

    data.par_chunks(height).zip(scratch.par_chunks_mut(height))
        .for_each(|(col, col_out)| dct_ii(col, col_out));

//    for (col, col_out) in data.chunks_mut(height).zip(scratch.chunks_mut(height)) {
//        dct_ii(col, col_out);
//    }

    // swap back
    transpose::transpose(&scratch, &mut data, height, width);

    data
}



// DCT-III is the inverse of DCT-II
fn inverse_dct_2d(planner: &mut DCTplanner<f32>, mut data: Vec<f32>, width: u32, height: u32) -> Vec<f32> {
    let (width, height) = (width as usize, height as usize);

    assert_eq!(data.len(), width * height);

//    let hdct = planner.plan_dct3(width);
//    let vdct = planner.plan_dct3(height);

    let mut scratch = vec![0f32; width * height];

//    for (row, row_out) in data.chunks_mut(width).zip(scratch.chunks_mut(width)) {
//        dct_iii(row, row_out);
//    }

    data.par_chunks(width).zip(scratch.par_chunks_mut(width))
        .for_each(|(row, row_out)| dct_iii(row, row_out));

    // swap rows and columns
    transpose::transpose(&scratch, &mut data, width, height);

//    for (col, col_out) in data.chunks_mut(height).zip(scratch.chunks_mut(height)) {
//        dct_iii(col, col_out);
//    }

    data.par_chunks(height).zip(scratch.par_chunks_mut(height))
        .for_each(|(col, col_out)| dct_iii(col, col_out));

    // swap back
    transpose::transpose(&scratch, &mut data, height, width);

    data
}

fn dct_ii(input: &[f32], output: &mut [f32]) {
    let len = input.len() as f32;

    for (k, xo) in output.iter_mut().enumerate() {
        *xo = (2.0 / len).sqrt() * input.iter().enumerate().map(|(n, xi)| {
            xi * (PI * (n as f32 + 0.5) * k as f32 / len).cos()
        }).sum::<f32>();

        if k == 0 {
            *xo *= 1.0 / SQRT_2;
        }
    }
}

fn dct_iii(input: &[f32], output: &mut [f32]) {
    let len = input.len() as f32;

    for (k, xo) in output.iter_mut().enumerate() {
        *xo = (input[0] / SQRT_2 + input.iter().enumerate().skip(1).map(|(n, xi)| {
            xi * (PI * n as f32 * (k as f32 + 0.5) / len).cos()
        }).sum::<f32>()) * (2.0 / len).sqrt();
    }
}
