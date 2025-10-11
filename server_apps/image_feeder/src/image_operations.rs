use std::io::Cursor;

use crate::errors::ImageProcessError;
use base64::{Engine, engine::general_purpose};
use image::{DynamicImage, GenericImageView, ImageFormat};
use reqwest::blocking::get;

fn into_error(
    e: Box<dyn std::error::Error>,
    err: ImageProcessError, //impl std::error::Error,
) -> ImageProcessError {
    log::error!("{e:?}");

    err
}

pub fn image_load_and_decode(url: &str) -> Result<DynamicImage, ImageProcessError> {
    let resp = get(url).map_err(|e| into_error(Box::new(e), ImageProcessError::ImageLoad))?;
    // let content_type = resp
    //     .headers()
    //     .get("content-type")
    //     .and_then(|v| v.to_str().ok());
    let bytes = resp
        .bytes()
        .map_err(|e| into_error(Box::new(e), ImageProcessError::ImageLoad))?;

    // Detect format from content-type
    //if let some(ct) = content_type {}

    // Detect format from bytes
    let format = image::guess_format(&bytes)
        .map_err(|e| into_error(Box::new(e), ImageProcessError::ImageGuessFormat))?;
    log::info!("Detected format {:?}.", format);

    let img = image::load_from_memory_with_format(&bytes, format)
        .map_err(|e| into_error(Box::new(e), ImageProcessError::ImageLoad))?;
    log::info!("Image dimensions: {:?}", img.dimensions());

    Ok(img)
}

pub fn image_from_bytes(
    bytes: &Vec<u8>,
    // expected_format: Option<&str>,
) -> Result<DynamicImage, ImageProcessError> {
    // Detect format from bytes
    // let format = match expected_format {
    // Some(f) => f,
    // None =>
    //  image::guess_format(&bytes)
    //     .map_err(|e| into_error(Box::new(e), ImageProcessError::ImageLoad)),};
    // log::info!("Detected format {:?}.", format);

    // let img = image::load_from_memory_with_format(&bytes, format)
    //     .map_err(|e| into_error(Box::new(e), ImageProcessError::ImageLoad))?;
    // log::info!("Image dimensions: {:?}", img.dimensions());
    let img = image::load_from_memory(bytes)
        .map_err(|e| into_error(Box::new(e), ImageProcessError::ImageLoad))?;
    log::info!("Image dimensions: {:?}", img.dimensions());

    Ok(img)
}

/// Webp always
/// Width kept on 512
pub fn create_thumbnail(img: &DynamicImage) -> DynamicImage {
    let (mut width, mut height) = img.dimensions();
    let aspect_ratio = width / height;

    width = 512;
    height = 512 * aspect_ratio;
    log::info!("Aspect ratio {width}:{height}");

    // Resize to 512x512
    let resized: DynamicImage = img.resize(width, height, image::imageops::FilterType::Lanczos3);
    resized
}

pub fn to_base64(img: &DynamicImage) -> String {
    let mut img_buf: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut img_buf), ImageFormat::WebP)
        .unwrap();

    let img_base64 = general_purpose::STANDARD.encode(img_buf);
    format!("data:image/webp;base64,{}", img_base64)
}
