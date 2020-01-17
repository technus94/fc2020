use std::env;
use image::{ImageBuffer, Pixel, Rgb, Rgba, Primitive};

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
        let in_range = |x, y| x > 0 && x + 1 < width && y > 0 && y + 1 < height;

        let is_gradient = |(x1, y1), (x2, y2)| {
            if in_range(x1, y1) && in_range(x2, y2) {
                let px1 = image.get_pixel(x1, y1);
                let px2 = image.get_pixel(x2, y2);

                px1.channels().iter().zip(px.channels())
                    .map(|(l, r)| l.wrapping_sub(*r))
                    .eq(px.channels().iter().zip(px2.channels())
                        .map(|(l, r)| l.wrapping_sub(*r)))
            } else {
                false
            }
        };

        let is_gradient = (x > 0 && is_gradient((x - 1, y), (x + 1, y))) ||
            (y > 0 && is_gradient((x, y - 1), (x, y + 1))) ||
            (x > 0 && y > 0 && is_gradient((x - 1, y - 1), (x + 1, y + 1)));

        if is_gradient {
            let mut px = px.to_rgba();
            px.blend(&overlay);
            image_out.put_pixel(x, y, px.to_rgb());
        } else {
            image_out.put_pixel(x, y, px.clone());
        }
    }

    image_out.save("out.png").expect("failed to write output file")
}
