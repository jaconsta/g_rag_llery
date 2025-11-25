use derive_getters::Getters;
use futures_util::TryStreamExt;
use pgvector::Vector;
use sqlx::Row;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::DbConn;
use crate::errors::{QueryError, QueryResult};
pub mod user_photos;

pub struct NewThumbnail<'a> {
    pub path: &'a str,
    pub height: i32,
    pub width: i32,
    pub ratio: &'a str,
}

pub struct NewEmbeddings {
    pub embeddings_id: i64,
}

#[derive(Debug, Clone, Getters, sqlx::FromRow)]
pub struct Gallery {
    id: Uuid,
    path: String,
    thumbnail_path: Option<String>,
    thumbnail_height: Option<i32>,
    thumbnail_width: Option<i32>,
    thumbnail_ratio: Option<String>,
    embeddings_id: Option<i64>,
    created_at: time::OffsetDateTime,
    updated_at: time::OffsetDateTime,
}

impl Gallery {
    pub fn new(path: &str) -> Self {
        let now = OffsetDateTime::now_utc();
        Gallery {
            id: Uuid::nil(),
            path: path.to_string(),
            thumbnail_path: None,
            thumbnail_height: None,
            thumbnail_width: None,
            thumbnail_ratio: None,
            embeddings_id: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub async fn create(&self, conn: &crate::DbConn) -> QueryResult<Gallery> {
        let gallery = sqlx::query_as!(
            Gallery,
            r#"
             with inserted_gallery as (
                 insert into gallery(path, created_at, updated_at)
                 values ($1, $2, $3)
                 returning id, path, embeddings_id, thumbnail_path, thumbnail_width, thumbnail_height, thumbnail_ratio,created_at, updated_at
             )
             select id, path, created_at, updated_at, embeddings_id, thumbnail_path, thumbnail_width, thumbnail_ratio, thumbnail_height
             from inserted_gallery
         "#,
            self.path,
            self.created_at,
            self.updated_at
        )
        .fetch_one(conn)
        .await
        .map_err(|e| {
            log::error!("{e:?}");
            QueryError::Query
        })?;

        Ok(gallery)
    }

    /// Takes thumbnail , embeddings and original moved bucket information and update
    /// the record in db. Initially was 3 methods but merged them into 1 to reduce db
    /// operations.
    pub async fn update_with_processed<'a>(
        &mut self,
        conn: &crate::DbConn,
        path: &str,
        thumbnail: NewThumbnail<'a>,
        embeddings: NewEmbeddings,
    ) -> QueryResult<()> {
        let updated_at = OffsetDateTime::now_utc();
        let _ = sqlx::query!(
            "UPDATE gallery SET path=$2, thumbnail_path=$3, thumbnail_height=$4, thumbnail_width=$5, thumbnail_ratio=$6, embeddings_id=$7,updated_at=$8 where id=$1",
            self.id,
            path, 
            thumbnail.path,
            thumbnail.height,
            thumbnail.width,
            thumbnail.ratio ,
            embeddings.embeddings_id,
            updated_at
        )
        .execute(conn)
        .await
        .map_err(|e| {
            log::error!("Failed on update {e:?}");
            QueryError::Query
        })?;

        self.path = path.to_string();
        self.thumbnail_path = Some(thumbnail.path.to_string());
        self.thumbnail_height = Some(thumbnail.height);
        self.thumbnail_width = Some(thumbnail.width);
        self.thumbnail_ratio= Some(thumbnail.ratio.to_string());
        self.embeddings_id = Some(embeddings.embeddings_id);
        self.updated_at = updated_at;

        Ok(())
    }

    /// Deletes
    /// Consumes itself to drop the value.
    pub async fn delete_one(self, conn: &crate::DbConn) -> QueryResult<()> {
         sqlx::query!(
            "DELETE from gallery where id=$1",
            self.id
        )
        .execute(conn)
        .await
        .map_err(|e| {
            log::error!("Failed on delete {e:?}");
            QueryError::Query
        })?;

        log::info!("Deleted galery id={}", self.id);
        Ok(())
    }

     pub async fn list_for_user(conn: &crate::DbConn, user_id: &Uuid) -> QueryResult<Vec<Gallery>> {
          let user_posts = sqlx::query_as!(
             Gallery,
             "SELECT g.* from gallery g inner join user_upload u on u.gallery_id=g.id where u.user_id=$1",
            user_id
         )
         .fetch_all(conn)
         .await?;

         Ok(user_posts)
     }
}

#[derive(Debug, Clone, Getters, sqlx::FromRow)]
pub struct GalleryEmbeddings {
    #[getter(copy)]
    id: i64,
    /// Bucket path of the image used for Processing
    path: String,
    /// Keywords assigned by the LLM
    keywords: Vec<String>,
    /// Description provided by the LLM
    description: Option<String>,
    /// Single word to contextualize the object (Source LLM)
    theme: Option<String>,
    /// Image aria label
    img_aria: Option<String>,
    /// Image aria alt  
    img_alt: Option<String>,
    /// Embeddings generated using CLIP
    embedding: Vec<f32>,
}

impl GalleryEmbeddings {
    pub fn new(path: String, embedding: Vec<f32>) -> Self {
        GalleryEmbeddings {
            id: 0,
            path,
            keywords: Vec::new(),
            description: None,
            embedding,
            theme: None,
            img_aria: None,
            img_alt: None,
        }
    }
    pub fn set_keywords(&mut self, keywords: Vec<String>) -> Self {
        self.keywords = keywords;
        self.to_owned()
    }
    pub fn set_description(&mut self, description: String) -> Self {
        self.description = Some(description);
        self.to_owned()
    }
    pub fn set_embedding(&mut self, embedding: Vec<f32>) -> Self {
        self.embedding = embedding;
        self.to_owned()
    }

    pub async fn create(&mut self, conn: &crate::DbConn) -> Result<(), QueryError> {
        let embe = Vector::from(self.embedding.clone());

        let embeddings_row = sqlx::query(
            r#"
              with i_embeddings as (
                  insert into gallery_rag_embeddings(path, keywords, description, embedding)
                  values ($1, $2, $3, $4)
                  returning id
              )
              select id
              from i_embeddings
          "#,
        )
        .bind(self.path.clone())
        .bind(self.keywords.clone())
        .bind(self.description.clone())
        .bind(&embe)
        .fetch_one(conn)
        .await
        .map_err(|e| {
            log::error!("{e:?}");
            QueryError::Query
        })?;

        let id: i64 = embeddings_row.try_get("id").unwrap();
        self.id = id;

        Ok(())
    }

    pub async fn find_nearest(
        embedding: Vec<f32>,
        conn: &crate::DbConn,
    ) -> Result<Vec<GalleryEmbeddings>, QueryError> {
        let embed_vec = Vector::from(embedding);
        let mut embeddings_rows = sqlx::query(
            r#"
              SELECT * 
              FROM gallery_rag_embeddings
              ORDER BY embedding <=> $1 LIMIT 10 
          "#,
        )
        .bind(embed_vec)
        .fetch(conn);

        let mut res = Vec::new();
        while let Ok(Some(row)) = embeddings_rows.try_next().await {
            let row_vec: Vector = row.get("embedding");
            res.push(GalleryEmbeddings {
                id: row.get("id"),
                path: row.get("path"),
                keywords: row.get("keywords"),
                description: row.get("description"),
                embedding: row_vec.to_vec(),
                theme: row.get("theme"),
                img_aria: row.get("img_aria"),
                img_alt: row.get("img_alt"),
            });
        }

        Ok(res)
    }

    pub async fn link_genai_descriptors(
        &self,
        conn: &DbConn,
        keywords: &Vec<String>,
        description: &str,
        theme: &str,
        alt: &str,
        aria: &str,
    ) -> Result<(), QueryError> {
        sqlx::query!(
            r#"
              UPDATE gallery_rag_embeddings SET keywords=$2, description=$3, theme=$4, img_alt=$5, img_aria=$6
              WHERE id=$1
          "#,
            self.id,
            keywords,
            description,
            theme,
            alt,
            aria
        )
        .execute(conn)
        .await
        .map_err(|e| {
            log::error!("{e:?}");

            QueryError::Query
        })?;

        Ok(())
    }

    // Deletes
    pub async fn delete_one(self, conn: &crate::DbConn) -> Result<(), QueryError> {
         sqlx::query!(
            "DELETE from gallery_rag_embeddings where id=$1",
    self.id
        )
        .execute(conn)
        .await
        .map_err(|e| {
            log::error!("Failed on delete {e:?}");
            QueryError::Query
        })?;

        log::info!("Deleted gallery_rag_embeddings id={}",self.id);
        Ok(())
    }
}

#[derive(Debug, Clone, Getters, sqlx::FromRow)]
pub struct UserUpload {
    id: Uuid,
    /// Bucket path of the image used for Processing
    filename: String,
    filesize: i64,
    filehash: String,
    user_id: Option<Uuid>,
    gallery_id: Option<Uuid>,
}

impl UserUpload {
    pub async fn new_for_upload(conn: &crate::DbConn, filename: &str, filesize: i32, filehash: &str, user_id: Uuid) -> Result<UserUpload, QueryError> {
        let user_upload = sqlx::query_as!(UserUpload, r#"
        with inserted_upload as (
            insert into user_upload (filename, filesize, filehash, user_id)
            values ($1, $2, $3, $4)
            returning id, filename, filesize, filehash, user_id, gallery_id
        )
            SELECT id, filename, filesize, filehash, user_id, gallery_id 
            from inserted_upload"#, filename, filesize, filehash, user_id).fetch_one(conn).await.map_err(|e| {log::error!("{e:?}"); QueryError::Query})?;
        
        Ok(user_upload)
    }

    pub async fn get_by_filename(conn: &crate::DbConn, filename: &str) -> Result<Self, QueryError> {
        let user_upload = sqlx::query_as!(UserUpload, r#"
            SELECT id, filename, filesize, filehash, user_id, gallery_id from user_upload where filename = $1"#, filename).fetch_one(conn).await.map_err(|e| {log::error!("{e:?}"); QueryError::Query})?;

        Ok(user_upload)
    }

    pub async fn set_gallery_id(
        &mut self,
        conn: &crate::DbConn,
        gallery_id: &Uuid,
    ) -> Result<(), QueryError> {
        println!("set_gallery_id {}", self.id);
        let _ = sqlx::query!(
            r#"
            UPDATE user_upload set gallery_id = $1 where id = $2"#,
            gallery_id,
            self.id
        )
        .execute(conn)
        .await
        .map_err(|e| {
            log::error!("{e:?}");
            QueryError::Query
        })?;

        self.gallery_id = Some(gallery_id.clone());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_creates_gallery() {
        let postgres_url = std::env!("DATABASE_URL");
        let pool = crate::db_connect(postgres_url).await.unwrap();

        let gallery_item = Gallery::new("/test/place.jpg".into());

        let res = gallery_item.create(&pool).await.unwrap();

        assert!(!res.id.is_nil());
        assert!(&res.path == "/test/place.jpg");
    }

    async fn create_reg() -> (crate::DbConn, Gallery) {
        let postgres_url = std::env!("DATABASE_URL");
        let pool = crate::db_connect(postgres_url).await.unwrap();

        let gallery_item = Gallery::new("/test/place.jpg".into());

        let res = gallery_item.create(&pool).await.unwrap();

        (pool, res)
    }

    #[tokio::test]
    async fn it_links_processed_to_gallery() {
        let (conn, mut gallery_itm) = create_reg().await;
        let pre_id = gallery_itm.id.clone();

        let mut embe = GalleryEmbeddings::new("/some/thumbnail.webp".into(), vec![1.0; 512])
            .set_keywords(vec!["keyone".to_string(), "keytwo".to_string()])
            .set_description("This summary is provided by ai".into());

        embe.create(&conn).await.unwrap();


        // it_updates_original_feeded_after_move
        let updated_path = "/new/some/feeded.jpg";
        // it_links_thumbnail_gallery
        let thumbnail = NewThumbnail{
                        path: "/some/thumbnailpath.jpg", height: 3 as i32, width: 4, ratio: "portrait" } ;
        // it_updates_fk_embeddings
        let embeddings = NewEmbeddings{embeddings_id: embe.id()};

        let r = gallery_itm.update_with_processed(
            &conn,updated_path, thumbnail, embeddings )
            .await;
        if let Err(e) = r {
            println!("{e:?}");
            assert!(false);
            return;
        }

        let _r = r.unwrap();

        assert!(pre_id == gallery_itm.id);
        assert!(
            gallery_itm
                .thumbnail_path
                .as_ref()
                .is_some_and(|f| f == "/some/path.jpg")
        );

        // Clean after
        let _  = gallery_itm.delete_one(&conn);
    }

    #[tokio::test]
    async fn it_creates_embeddings() {
        let (conn, gallery_itm) = create_reg().await;

        let mut embe = GalleryEmbeddings::new("/some/thumbnail.webp".into(), vec![1.0; 512])
            .set_keywords(vec!["keyone".to_string(), "keytwo".to_string()])
            .set_description("This summary is provided by ai".into());

        embe.create(&conn).await.unwrap();

        assert!(embe.id > 0);

        // Clean after
        let _ = gallery_itm.delete_one(&conn).await;
        let _ = embe.delete_one(&conn).await;
    }

    #[tokio::test]
    async fn it_finds_gallery_with_embeddings() {
        let (conn, gallery_itm) = create_reg().await;

        let embe = GalleryEmbeddings::find_nearest(vec![1.0; 512], &conn)
            .await
            .unwrap();
        println!("{:?}", embe);

        assert!(embe.len() > 1);

        // Clean after
        let _ = gallery_itm.delete_one(&conn).await;
        for e in embe.iter().to_owned() {
        let _ = <GalleryEmbeddings as Clone>::clone(&e).delete_one(&conn).await;
        }
    }

    #[tokio::test]
    async fn it_update_gallery_description() {
        // This test is flaky because it depends on "it_creates_embeddings"
        let (conn, gallery_itm) = create_reg().await;

        let mut embe = GalleryEmbeddings::new("/some/thumbnail.webp".into(), vec![1.0; 512])
            .set_keywords(vec!["keyone".to_string(), "keytwo".to_string()])
            .set_description("This summary is provided by ai".into());
        embe.create(&conn).await.unwrap();

        let keywords = vec!["newone".to_string(), "newtwo".to_string()];
        let description = "This a new description";
        embe.link_genai_descriptors(&conn, &keywords, description, "theme", "aria", "alt")
            .await
            .unwrap();

        let result = sqlx::query!(
            "SELECT id, description, keywords from gallery_rag_embeddings where id=$1",
            embe.id()
        )
        .fetch_one(&conn)
        .await;
        let gallery_embed = result.iter().next().unwrap();

        assert!(gallery_embed.id == embe.id());
        assert!(
            gallery_embed
                .description
                .clone()
                .is_some_and(|f| f == description)
        );
        assert!(
            gallery_embed
                .keywords
                .clone()
                .is_some_and(|f| f == keywords)
        );

        // Clean after
        let _ = gallery_itm.delete_one(&conn).await;
        let _ = embe.delete_one(&conn).await;
    }
}

