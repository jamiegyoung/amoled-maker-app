use std::{fmt, io, path::Path};

use crate::Message;

use ::image::{io::Reader as ImageReader, Bgra, DynamicImage, ImageBuffer};
use iced::{
    image::{viewer, Handle, Viewer},
    Align, Column, Container, Element, HorizontalAlignment, Length, Row, Text,
};
use image::ImageError;

// #[derive(Debug, Default, Clone)]
// struct ImagePixels {
//     width: u32,
//     height: u32,
//     pixels: Vec<u8>,
// }

const THUMBNAIL_MAX_WIDTH: u32 = 1024;
const THUMBNAIL_MIN_WIDTH: u32 = 256;
const THUMBNAIL_MAX_HEIGHT: u32 = 1024;
const THUMBNAIL_MIN_HEIGHT: u32 = 256;

#[derive(Debug)]
pub enum AmoledConversionError {
    DecodeError(io::Error),
    ImageError(ImageError),
    // ImageParseError,
}

impl From<ImageError> for AmoledConversionError {
    fn from(e: ImageError) -> Self {
        AmoledConversionError::ImageError(e)
    }
}

impl From<std::io::Error> for AmoledConversionError {
    fn from(e: io::Error) -> AmoledConversionError {
        AmoledConversionError::DecodeError(e)
    }
}

impl fmt::Display for AmoledConversionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AmoledConversionError::DecodeError(e) => write!(f, "Decode error: {}", e),
            AmoledConversionError::ImageError(e) => write!(f, "Image error: {}", e),
            // AmoledImageError::ImageParseError => write!(f, "Image parse error"),
        }
    }
}

// impl From<std::io::Error> for DecodeError {
//     fn from(err: std::io::Error) -> Self {
//         todo!()
//     }
// }

struct PixelInfo {
    pixels: usize,
    black_pixels: usize,
}

type Result<T> = std::result::Result<T, AmoledConversionError>;

#[derive(Debug, Clone)]
pub struct AmoledImageConverter {
    width: u32,
    height: u32,
    // data: ImagePixels,
    image: ImageBuffer<Bgra<u8>, Vec<u8>>,
    converted_image: ImageBuffer<Bgra<u8>, Vec<u8>>,
    pub thumbnail: ImageBuffer<Bgra<u8>, Vec<u8>>,
    converted_thumbnail: ImageBuffer<Bgra<u8>, Vec<u8>>,
    first_image_viewer: viewer::State,
    second_image_viewer: viewer::State,
    black_point: u8,
    image_handle: Handle,
    converted_image_black_pixel_percentage: usize,
}

impl AmoledImageConverter {
    pub fn from_path(path: &Path, black_point: u8) -> Result<AmoledImageConverter> {
        let decoded_image = ImageReader::open(path)?.decode()?;
        AmoledImageConverter::new(decoded_image, black_point)
    }

    pub fn new(image: DynamicImage, black_point: u8) -> Result<AmoledImageConverter> {
        let bgra8_image = image.to_bgra8();
        let thumbnail = image
            .thumbnail(
                AmoledImageConverter::clamp(
                    bgra8_image.width() / 2,
                    THUMBNAIL_MIN_WIDTH,
                    THUMBNAIL_MAX_WIDTH,
                ),
                AmoledImageConverter::clamp(
                    bgra8_image.height() / 2,
                    THUMBNAIL_MIN_HEIGHT,
                    THUMBNAIL_MAX_HEIGHT,
                ),
            )
            .to_bgra8();

        let mut converted_thumbnail = thumbnail.clone();
        AmoledImageConverter::generate_black_image(&mut converted_thumbnail, black_point);
        // This is quite a heavy task and can maybe be otimized if we don't need the original image's black point ratio.
        let mut converted_image = bgra8_image.clone();
        let converted_info =
            AmoledImageConverter::generate_black_image(&mut converted_image, black_point);
        return Ok(AmoledImageConverter {
            width: bgra8_image.width(),
            height: bgra8_image.height(),
            image: bgra8_image.to_owned(),
            converted_image: converted_image,
            thumbnail,
            converted_thumbnail,
            image_handle: Handle::from_memory(bgra8_image.as_raw().to_owned()),
            first_image_viewer: iced::image::viewer::State::new(),
            second_image_viewer: iced::image::viewer::State::new(),
            black_point,
            converted_image_black_pixel_percentage:
                AmoledImageConverter::calc_black_pixel_percentage(
                    converted_info.black_pixels,
                    converted_info.pixels,
                ),
        });

        // If the image cannot load into a brga8 image, return an empty image.
    }

    fn calc_black_pixel_percentage(black_pixel_count: usize, pixel_count: usize) -> usize {
        black_pixel_count * 100 / pixel_count
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }

    pub fn set_black_point(&mut self, black_point: u8) {
        self.black_point = black_point;
        self.converted_thumbnail = self.thumbnail.clone();
        let converted_image_info =
            AmoledImageConverter::generate_black_image(&mut self.converted_thumbnail, black_point);
        self.converted_image_black_pixel_percentage =
            AmoledImageConverter::calc_black_pixel_percentage(
                converted_image_info.black_pixels,
                converted_image_info.pixels,
            );
        self.converted_image = self.image.clone();
        AmoledImageConverter::generate_black_image(&mut self.converted_image, black_point);
    }

    fn clamp(x: u32, min: u32, max: u32) -> u32 {
        if x > max {
            return max;
        }
        if x < min {
            return min;
        }
        x
    }

    fn count_black_pixels(image: &ImageBuffer<Bgra<u8>, Vec<u8>>) -> usize {
        let count = image.pixels().fold(0, |acc: usize, x| {
            let b: u8 = x[0];
            let g: u8 = x[1];
            let r: u8 = x[2];
            if b == 0 && g == 0 && r == 0 {
                return acc + 1;
            }
            acc
        });
        count
    }

    fn generate_black_image(
        image: &mut ImageBuffer<Bgra<u8>, Vec<u8>>,
        black_point: u8,
    ) -> PixelInfo {
        let pixel_iter = image.pixels_mut();
        let mut black_pixel_count: usize = 0;
        let mut pixel_count: usize = 0;
        for x in pixel_iter {
            pixel_count += 1;
            let b = x[0];
            if b > black_point {
                continue;
            }

            let g = x[1];
            if g > black_point {
                continue;
            }

            let r = x[2];
            if r > black_point {
                continue;
            }

            black_pixel_count += 1;
            if r != 0 || g != 0 || b != 0 {
                *x = Bgra([0, 0, 0, x[3]]);
            }
        }

        PixelInfo {
            pixels: pixel_count,
            black_pixels: black_pixel_count,
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        Container::new(
            Row::new()
                .padding(10)
                .spacing(20)
                .push(
                    AmoledImageConverter::view_thumbnail(
                        &self.thumbnail,
                        &mut self.first_image_viewer,
                        AmoledImageConverter::count_black_pixels(&self.image) * 100
                            / self.image.pixels().len(),
                    )
                    .align_items(Align::End),
                )
                .push(
                    AmoledImageConverter::view_thumbnail(
                        &self.converted_thumbnail,
                        &mut self.second_image_viewer,
                        self.converted_image_black_pixel_percentage,
                    )
                    .align_items(Align::Start),
                ),
        )
        .width(Length::Shrink)
        .into()
    }

    fn view_thumbnail<'a>(
        thumbnail: &ImageBuffer<Bgra<u8>, Vec<u8>>,
        viewer: &'a mut iced::image::viewer::State,
        black_pixel_percentage: usize,
    ) -> Column<'a, Message> {
        Column::new().width(Length::Fill).push(
            Column::new()
                .align_items(Align::Center)
                .spacing(20)
                .push(
                    Viewer::new(
                        viewer,
                        Handle::from_pixels(
                            thumbnail.width(),
                            thumbnail.height(),
                            thumbnail.as_raw().to_owned(),
                        ),
                    )
                    .height(Length::Fill),
                )
                .push(
                    Text::new(format!("{}% Black", black_pixel_percentage))
                        .horizontal_alignment(HorizontalAlignment::Center),
                ),
        )
    }
}
