use fastembed::{ImageEmbedding, ImageEmbeddingModel, ImageInitOptions};
use image::DynamicImage;

pub fn get_img_embeddings(img: DynamicImage) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let mut model = ImageEmbedding::try_new(
        ImageInitOptions::new(ImageEmbeddingModel::ClipVitB32).with_show_download_progress(true),
    )?;

    let images = vec![img];
    // Generate embeddings with de default batch size, 256
    let embeddings = model.embed_images(images)?;
    let embedding = embeddings[0].to_owned();

    println!("Embeddings dimention {:?}", embedding);
    Ok(embedding)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, RgbImage};

    #[test]
    fn it_generates_for_black_box() {
        let rgb: RgbImage = RgbImage::new(10, 10);
        let gray_image = DynamicImage::ImageRgb8(rgb);

        let emb = get_img_embeddings(gray_image).unwrap();

        assert!(emb.len() == 512);
        assert!(
            emb[0..6]
                == vec![
                    -0.013126711,
                    -0.022995966,
                    -0.04937373,
                    -0.0063058883,
                    0.013601426,
                    -0.0037615884
                ]
        )
    }
}
