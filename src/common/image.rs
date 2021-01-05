use crate::common::Margin;
use crate::{Image, ImageOnHeap, Result};
use image::flat::View;
use image::{imageops, Bgra, GenericImageView, ImageBuffer};

///
/// specialized version of crop for [`ImageOnHeap`] and [`Margin`]
///
#[cfg_attr(not(macos), allow(dead_code))]
pub fn crop(image: Image, margin: &Margin) -> Result<ImageOnHeap> {
    let mut img2: View<_, Bgra<u8>> = image.as_view()?;
    let (width, height) = (
        img2.width() - (margin.left + margin.right) as u32,
        img2.height() - (margin.top + margin.bottom) as u32,
    );
    let image_cropped = imageops::crop(
        &mut img2,
        margin.left as u32,
        margin.top as u32,
        width,
        height,
    );
    let mut buf = ImageBuffer::new(image_cropped.width(), image_cropped.height());

    for y in 0..height {
        for x in 0..width {
            buf.put_pixel(x, y, image_cropped.get_pixel(x, y));
        }
    }

    Ok(Box::new(buf.into_flat_samples()))
}

#[cfg(feature = "experimental")]
mod experimental {
    use crate::{ImageOnHeap, Result};
    use anyhow::Context;
    use image::flat::View;
    use image::{open, save_buffer, Bgra, GenericImageView, Pixel};

    fn draw_round_corners<P: Pixel>(
        image: &mut View<&mut [P::Subpixel], P>,
        fill_color: &P,
        radius: i32,
    ) {
        // all those are non in pixel coordinates but in normal coordinates (to reduce my mental load)
        let (width, height) = (image.width() as i32, image.height() as i32);
        let (top, bottom, left, right) = (height - radius, radius, radius, width - radius);
        let radius = radius.pow(2);

        // when
        for (y_range, ytr) in &[(top..height, top), (0..bottom, bottom)] {
            for y in y_range.start..y_range.end {
                for (x_range, xtr) in &[(right..width, right), (0..left, left)] {
                    for x in x_range.start..x_range.end {
                        // those coordinates describe 1/4 of a circle on the top right part of the image
                        // Note: circle formula: x^2 + y^2 = r^2
                        if (x - xtr).pow(2) + (y - ytr).pow(2) >= radius {
                            paint(image, x, height - y, fill_color);
                        }
                    }
                }
            }
        }
    }

    fn paint<P: Pixel>(image: &mut View<&mut [P::Subpixel], P>, x: i32, y: i32, color: &P) {
        let (x, y) = (x as u32, y as u32);
        for (i, channel) in color.channels().iter().enumerate() {
            // for channel in color {
            if let Some(sample) = image.get_mut_sample(i as u8, x, y) {
                // make that sample full, so the pixel appears to be white
                *sample = *channel;
            }
        }
    }

    #[test]
    #[cfg(test)]
    #[cfg(feature = "experimental")]
    fn should_round_corners() -> Result<()> {
        // given
        let mut image_raw = load_image("tests/frames/frame-macos-squared.tga")?;
        let mut image: View<_, Bgra<u8>> = image_raw.as_view_with_mut_samples()?;
        let fill_color = &Bgra([0xff, 0xff, 0xff, 0xff]);
        let radius = 10;

        // when
        draw_round_corners(&mut image, fill_color, radius);

        // then
        // TODO(verify) assert that only the corners have been modified accordingly

        save_buffer(
            "frame-round-corner.tga",
            &image_raw.samples,
            image_raw.layout.width,
            image_raw.layout.height,
            image_raw.color_hint.unwrap(),
        )
        .context("Cannot save a frame.")?;

        Ok(())
    }

    #[cfg(test)]
    #[cfg(feature = "experimental")]
    fn load_image(path: &str) -> Result<ImageOnHeap> {
        let image_org = open(path)?;
        let image = image_org.into_bgra8();
        let image_raw = ImageOnHeap::new(image.into_flat_samples());

        Ok(image_raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::open;

    #[test]
    fn should_crop() -> Result<()> {
        // given
        let image_raw = load_image("tests/frames/frame-macos-right-side-issue.tga")?;
        let (width, height) = (image_raw.layout.width, image_raw.layout.height);

        // when
        let cropped = crop(*image_raw, &Margin::new(1, 1, 1, 1))?;

        // then
        assert_eq!(cropped.layout.width, width - 2);
        assert_eq!(cropped.layout.height, height - 2);

        Ok(())
    }

    fn load_image(path: &str) -> Result<ImageOnHeap> {
        let image_org = open(path)?;
        let image = image_org.into_bgra8();
        let image_raw = ImageOnHeap::new(image.into_flat_samples());

        Ok(image_raw)
    }
}
