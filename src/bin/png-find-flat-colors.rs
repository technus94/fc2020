use std::env;
use image::{ImageBuffer, Pixel, Rgb, Rgba};

fn main() {
    let image_path = env::args().nth(1).expect("expected image path");

    let image = image::open(&image_path)
        .expect(&format!("failed to open {}", image_path))
        .into_rgb();

    let width = image.width();
    let height = image.height();

    let mut image_out = ImageBuffer::<Rgb<u8>, _>::new(width, height);

    let overlay = Rgba([255, 0, 0, 127]);

    for (x, y, px) in image.enumerate_pixels() {
        let (x, y) = (x as i32, y as i32);
        let num_matching =
            [(x - 1, y), (x, y - 1)].iter().cloned()
        // (x - 1 .. x + 1).map(|x| (x, y)).chain((y - 1 .. y + 1).map(|y| (x, y)))
        // (-dist .. dist).flat_map(|xd| (-dist .. dist).map(move |yd| (x as i32 + xd, y as i32 + yd)))
            .filter(|&(x_, y_)| x != x_ || y != y_)
            .filter(|(x, y)| (0 .. width as _).contains(x) && (0 .. height as _).contains(y))
            .filter(|&(x, y)| image.get_pixel(x as u32, y as u32) == px)
            .count();

        let (x, y) = (x as u32, y as u32);

        if num_matching > 1 {
            let mut px = px.to_rgba();
            px.blend(&overlay);
            image_out.put_pixel(x, y, px.to_rgb());
        } else {
            image_out.put_pixel(x, y, px.clone());
        }
    }

    image_out.save("out.png").expect("failed to write output file")
}
