use tonic::{Request, Response, Status};

pub use gallery_view_rpc::gallery_view_server::{GalleryView, GalleryViewServer};
use gallery_view_rpc::{
    EmptyRequest, FilterGalleryRequest, FilterOptionResponse, GalleryImagesResponse,
    SignedLinkResponse, UploadImageRequest,
};
use uuid::Uuid;

use crate::{
    bucket::BucketClient,
    gallery_view::{
        gallery_view_rpc::GalleryImage,
        model::{FileUpload, UserId},
    },
};

pub mod gallery_view_rpc {
    tonic::include_proto!("gallery_view");
}

pub mod model {
    use db_storage::models::{
        UserUpload,
        user_photos::{FilterableProperties, UserPhoto},
    };
    use derive_getters::Getters;
    use uuid::Uuid;

    use crate::{
        bucket::{Bucket, BucketClient},
        error::{Error, Result},
    };

    pub type UserId = Uuid;

    #[derive(Debug, Clone, Copy, Getters)]
    pub struct FileUpload<'a> {
        name: &'a str,
        hash: &'a str,
        size: i32,
    }

    impl<'a> FileUpload<'a> {
        pub fn new(name: &'a str, hash: &'a str, size: i32) -> FileUpload<'a> {
            return FileUpload { name, hash, size };
        }
    }

    pub struct UserGallery<'a> {
        conn: db_storage::DbConn,
        bucket: BucketClient<'a>,
    }

    impl<'a> UserGallery<'a> {
        pub fn new(db: db_storage::DbConn, bucket: BucketClient<'a>) -> Self {
            Self { conn: db, bucket }
        }

        pub async fn request_upload(&self, id: UserId, upload: &FileUpload<'_>) -> Result<String> {
            let filename = format!("feeder/{}", upload.name());
            // Check if is duplicated
            match UserUpload::get_by_filename(&self.conn, &filename).await {
                Ok(_) => return Err(Box::new(Error::Duplicated)),
                Err(_) => { /* Assume -> err means empty result */ }
            };

            // Create the record
            UserUpload::new_for_upload(&self.conn, &filename, *upload.size(), upload.hash(), id)
                .await?;

            // The user only needs the upload url at this point.
            Ok(self
                .bucket
                .get_upload_signed_url(&filename, Bucket::Feeder)
                .await?)
        }

        pub async fn get(&self, id: UserId) -> Result<(Vec<UserPhoto>, i64)> {
            let mut user_photos = UserPhoto::get_photos(&self.conn, id.into()).await?;
            let count = UserPhoto::count_photos(&self.conn, id).await?;

            for photo in user_photos.iter_mut() {
                match photo.thumbnail_path() {
                    Some(url) => {
                        match self
                            .bucket
                            .get_download_signed_url(url, Bucket::Ragged)
                            .await
                        {
                            Ok(url) => photo.set_signed_url(url),
                            Err(e) => log::error!("{e:?}"),
                        };
                    }
                    _ => (),
                };
            }

            Ok((user_photos, count))
        }

        pub async fn filters(
            &self,
            id: UserId,
        ) -> Result<db_storage::models::user_photos::FilterableProperties> {
            Ok(FilterableProperties::get_for_user(&self.conn, id.into()).await?)
        }
    }
}

impl From<Vec<db_storage::models::user_photos::UserPhoto>> for GalleryImagesResponse {
    fn from(value: Vec<db_storage::models::user_photos::UserPhoto>) -> Self {
        Self {
            images: value
                .iter()
                .map(|f| GalleryImage {
                    img_url: f.thumbnail_path().as_ref().map_or("", |f| f).to_string(),
                    aria_text: f.img_aria().as_ref().map_or("", |f| f).to_string(),
                    aspect: f.thumbnail_ratio().as_ref().map_or("", |f| f).to_string(),
                    theme: f.theme().as_ref().map_or("", |f| f).to_string(),
                    alt_text: f.img_alt().as_ref().map_or("", |f| f).to_string(),
                })
                .collect(),
            count: 0,
        }
    }
}

#[derive(Debug)]
pub struct GalleryService<'a> {
    conn: db_storage::DbConn,
    bucket: BucketClient<'a>,
}

impl<'a> GalleryService<'a> {
    pub fn new(conn: db_storage::DbConn, bucket: BucketClient<'a>) -> GalleryService<'a> {
        Self { conn, bucket }
    }
}

#[tonic::async_trait]
impl<'a> GalleryView for GalleryService<'static> {
    async fn upload_image(
        &self,
        request: Request<UploadImageRequest>,
    ) -> std::result::Result<Response<SignedLinkResponse>, Status> {
        let user_id: UserId = Uuid::nil();
        let req_info = request.get_ref();
        let file_info = FileUpload::new(&req_info.filename, &req_info.filehash, req_info.filesize);

        let uploadurl =
            crate::gallery_view::model::UserGallery::new(self.conn.clone(), self.bucket.clone())
                .request_upload(user_id, &file_info)
                .await;

        match uploadurl {
            Ok(bucket_link) => Ok(Response::new(SignedLinkResponse { bucket_link })),
            Err(_) => Ok(Response::new(SignedLinkResponse {
                bucket_link: "None".to_string(),
            })),
        }
    }

    async fn list_gallery(
        &self,
        _request: Request<FilterGalleryRequest>,
    ) -> std::result::Result<tonic::Response<GalleryImagesResponse>, Status> {
        let user_id: UserId = Uuid::nil();
        let get_response =
            crate::gallery_view::model::UserGallery::new(self.conn.clone(), self.bucket.clone())
                .get(user_id)
                .await;

        let (user_photos, count) = get_response.unwrap();

        let mut gallery_results: GalleryImagesResponse = user_photos.into();
        gallery_results.count = count as i32;

        Ok(Response::new(gallery_results))
    }

    async fn filter_options(
        &self,
        _request: Request<EmptyRequest>,
    ) -> std::result::Result<Response<FilterOptionResponse>, Status> {
        let user_id: UserId = Uuid::nil();
        let filters =
            crate::gallery_view::model::UserGallery::new(self.conn.clone(), self.bucket.clone())
                .filters(user_id)
                .await;

        let filters = filters.unwrap();
        Ok(Response::new(FilterOptionResponse {
            aspects: filters.aspects().clone(),
            themes: filters.themes().clone(),
        }))
    }
}
