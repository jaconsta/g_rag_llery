use std::io::Cursor;

use crate::errors::ImageProcessError;
use base64::{Engine, engine::general_purpose};
use derive_getters::Getters;
use image::{DynamicImage, GenericImageView, ImageFormat};
// use reqwest::blocking::get;

fn into_error(
    e: Box<dyn std::error::Error>,
    err: ImageProcessError, //impl std::error::Error,
) -> ImageProcessError {
    log::error!("{e:?}");

    err
}

#[derive(Getters, Clone)]
pub struct ImageData {
    pub image: DynamicImage,
    pub height: u32,
    pub width: u32,
    pub aspect_ratio: f32,
}

impl ImageData {
    pub fn ratio_as_str(&self) -> String {
        // Calculate it again just in case
        // let (width, height) = self.image.dimensions();
        // let aspect_ratio = width as f32 / height as f32;

        let res = match self.aspect_ratio {
            r if 0.95 <= r && r <= 1.05 => "square",
            r if r < 0.95 && r >= 0.5 => "portrait",
            r if r < 0.5 => "tall",
            r if r > 2.0 => "wide",
            _ => "landscape",
        };

        String::from(res)
    }
}

// pub fn image_load_and_decode(url: &str) -> Result<DynamicImage, ImageProcessError> {
//     let resp = get(url).map_err(|e| into_error(Box::new(e), ImageProcessError::ImageLoad))?;
//     // let content_type = resp
//     //     .headers()
//     //     .get("content-type")
//     //     .and_then(|v| v.to_str().ok());
//     let bytes = resp
//         .bytes()
//         .map_err(|e| into_error(Box::new(e), ImageProcessError::ImageLoad))?;
//
//     // Detect format from bytes
//     let format = image::guess_format(&bytes)
//         .map_err(|e| into_error(Box::new(e), ImageProcessError::ImageGuessFormat))?;
//     log::info!("Detected format {:?}.", format);
//
//     let img = image::load_from_memory_with_format(&bytes, format)
//         .map_err(|e| into_error(Box::new(e), ImageProcessError::ImageLoad))?;
//     log::info!("Image dimensions: {:?}", img.dimensions());
//
//     Ok(img)
// }

pub fn image_from_bytes(bytes: &Vec<u8>) -> Result<DynamicImage, ImageProcessError> {
    let img = image::load_from_memory(bytes)
        .map_err(|e| into_error(Box::new(e), ImageProcessError::ImageLoad))?;
    log::info!("Image dimensions: {:?}", img.dimensions());

    Ok(img)
}

/// Webp always
/// Width capped at 512
pub fn create_thumbnail(img: &DynamicImage) -> ImageData {
    let (mut width, mut height) = img.dimensions();
    let aspect_ratio = width as f32 / height as f32;

    width = 512;
    height = (512_f32 * aspect_ratio) as u32;
    log::info!("Aspect ratio {width}:{height}");

    // Resize to 512x512
    let resized: DynamicImage = img.resize(width, height, image::imageops::FilterType::Lanczos3);
    ImageData {
        image: resized,
        height: height,
        width: width,
        aspect_ratio: aspect_ratio,
    }
}

/// Encodes the given Image into a webp base 64 image.
/// It also adds the image type prefix to the output
/// `data:image/webp;base64,`
pub fn to_base64(img: &DynamicImage) -> String {
    let img_base64 = to_llava_base64(img);
    format!("data:image/webp;base64,{}", img_base64)
}

/// Encodes the given image into a base64 webp.
/// Returns only the base64 content
pub fn to_llava_base64(img: &DynamicImage) -> String {
    let mut img_buf: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut img_buf), ImageFormat::WebP)
        .unwrap();

    let img_base64 = general_purpose::STANDARD.encode(img_buf);
    img_base64
}
