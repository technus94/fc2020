use image::{ImageBuffer, GenericImageView, Rgb, Luma, Pixel, imageops, FilterType};
use image::DynamicImage::ImageBgr8;

fn main() {
    std::fs::create_dir_all("jpeg-chroma-subsample").unwrap();

    let image_path = std::env::args().nth(1).expect("expected image path");
    let image = image::open(&image_path)
        .expect(&format!("failed to open {}", image_path));

    let rgb = image.to_rgb();
    let luma = image.to_luma();

    luma.save("jpeg-chroma-subsample/luma.png").unwrap();

    let mut red_pixels = (*rgb).to_owned();
    red_pixels.chunks_mut(3).for_each(|px| {
        px[1] = 0;
        px[2] = 0;
    });

    ImageBuffer::<Rgb<u8>, _>::from_vec(rgb.width(), rgb.height(), red_pixels)
        .unwrap()
        .save("jpeg-chroma-subsample/red.png")
        .unwrap();

    let mut green_pixels = (*rgb).to_owned();
    green_pixels.chunks_mut(3).for_each(|px| {
        px[0] = 0;
        px[2] = 0;
    });

    ImageBuffer::<Rgb<u8>, _>::from_vec(rgb.width(), rgb.height(), green_pixels)
        .unwrap()
        .save("jpeg-chroma-subsample/green.png")
        .unwrap();

    let mut blue_pixels = (*rgb).to_owned();
    blue_pixels.chunks_mut(3).for_each(|px| {
        px[0] = 0;
        px[1] = 0;
    });

    ImageBuffer::<Rgb<u8>, _>::from_vec(rgb.width(), rgb.height(), blue_pixels)
        .unwrap()
        .save("jpeg-chroma-subsample/blue.png")
        .unwrap();

    let ycbcr: Vec<u8> = rgb.chunks(3).zip(luma.iter())
        .flat_map(|(rgb, &luma)| {
            let r = rgb[0] as f64;
            let g = rgb[1] as f64;
            let b = rgb[2] as f64;
            vec![
                luma,
                (128. - 0.168736 * r - 0.331264 * g + 0.5 * b) as u8,
                (128. + 0.5 * r - 0.418688 * g - 0.081312 * b) as u8
            ]
        })
        .collect();

    // show Cb difference by converting back to RGB with the Cr channel set to 0
    let chroma_blue = ImageBuffer::<Rgb<u8>, _>::from_vec(
        image.width(),
        image.height(),
        ycbcr.chunks(3).flat_map(|ycbr| ycbr_to_rgb(&[ycbr[0], ycbr[1], 0])).collect(),
    ).unwrap();

    chroma_blue.save("jpeg-chroma-subsample/chroma-blue.png").unwrap();

    // vice versa
    let chroma_red = ImageBuffer::<Rgb<u8>, _>::from_vec(
        image.width(),
        image.height(),
        ycbcr.chunks(3).flat_map(|ycbr| ycbr_to_rgb(&[ycbr[0], 0, ycbr[2]])).collect(),
    ).unwrap();

    chroma_red.save("jpeg-chroma-subsample/chroma-red.png").unwrap();

    let chroma_blue_ss = imageops::resize(&chroma_blue, dbg!(image.width() / 2), image.height() / 2, FilterType::Lanczos3);
    let chroma_blue_ss = imageops::resize(&chroma_blue_ss, dbg!(image.width()), image.height(), FilterType::Nearest);

    chroma_blue_ss.save("jpeg-chroma-subsample/chroma-blue-ss.png").unwrap();

    let chroma_red_ss = imageops::resize(&chroma_red, image.width() / 2, image.height() / 2, FilterType::Lanczos3);
    let chroma_red_ss = imageops::resize(&chroma_red_ss, image.width(), image.height(), FilterType::Nearest);

    chroma_red_ss.save("jpeg-chroma-subsample/chroma-red-ss.png").unwrap();
}

#[inline]
fn ycbr_to_rgb(ycbr: &[u8]) -> Vec<u8> {
    let y = ycbr[0] as f64;
    let cb = ycbr[1] as f64;
    let cr = ycbr[2] as f64;

    vec![
        (y + 1.402 * (cr - 128.)) as u8,
        (y - 0.344136 * (cb - 128.) - 0.714136 * (cr - 128.)) as u8,
        (y + 1.772 * (cb - 128.)) as u8
    ]
}
