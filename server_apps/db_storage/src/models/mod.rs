use derive_getters::Getters;
use futures_util::TryStreamExt;
use pgvector::Vector;
use sqlx::Row;
use std::time::SystemTime;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use uuid::Uuid;

use crate::DbConn;
use crate::errors::QueryError;

#[derive(Debug, Clone, Getters, sqlx::FromRow)]
pub struct Gallery {
    id: Uuid,
    path: String,
    thumbnail_path: Option<String>,
    thumbnail_height: Option<i32>,
    thumbnail_width: Option<i32>,
    embeddings_id: Option<i64>,
    created_at: time::OffsetDateTime,
    updated_at: time::OffsetDateTime,
}

impl Gallery {
    pub fn new(path: String) -> Self {
        let now = OffsetDateTime::now_utc();
        Gallery {
            id: Uuid::nil(),
            path,
            thumbnail_path: None,
            thumbnail_height: None,
            thumbnail_width: None,
            embeddings_id: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn set_thumbnail(&mut self, path: String) -> Self {
        self.thumbnail_path = Some(path);
        self.updated_at = OffsetDateTime::now_utc();
        self.to_owned()
    }
    pub fn set_embeddings(&mut self, fk_id: i64) -> Self {
        self.embeddings_id = Some(fk_id);
        self.updated_at = OffsetDateTime::now_utc();
        self.to_owned()
    }

    pub async fn create(&self, conn: &crate::DbConn) -> Result<Gallery, QueryError> {
        let gallery = sqlx::query_as!(
            Gallery,
            r#"
             with inserted_gallery as (
                 insert into gallery(path, created_at, updated_at)
                 values ($1, $2, $3)
                 returning id, path, embeddings_id, thumbnail_path, thumbnail_width, thumbnail_height, created_at, updated_at
             )
             select id, path, created_at, updated_at, embeddings_id, thumbnail_path, thumbnail_width, thumbnail_height
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

    pub async fn link_thumbnail(
        &self,
        conn: &crate::DbConn,
        thumbnail: &str,
        height: u32,
        width: u32,
        ratio: String,
    ) -> Result<(), QueryError> {
        let _ = sqlx::query!(
            "UPDATE gallery SET thumbnail_path=$2, thumbnail_height=$3, thumbnail_width=$4, thumbnail_ratio=$5 where id=$1",
            self.id,
            thumbnail,
            height as i32,
            width as i32,
            ratio 
        )
        .execute(conn)
        .await
        .map_err(|e| {
            log::error!("whyme");
            log::error!("{e:?}");
            QueryError::Query
        })?;

        Ok(())
    }
    pub async fn link_embeddings(
        &self,
        conn: &crate::DbConn,
        embedding_id: i64,
    ) -> Result<(), QueryError> {
        let _ = sqlx::query!(
            "UPDATE gallery SET embeddings_id=$1 where id=$2",
            embedding_id,
            self.id
        )
        // .bind(embedding_id)
        // .bind(self.id.to_string())
        .execute(conn)
        .await
        .map_err(|e| {
            log::error!("{e:?}");
            QueryError::Query
        })?;

        Ok(())
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
        // let embeddings = sqlx::query!(
        //     r#"
        //      with i_embeddings as (
        //          insert into gallery_rag_embeddings(path, keywords, description, embedding)
        //          values ($1, $2, $3, $4)
        //          returning id
        //      )
        //      select id
        //      from i_embeddings
        //  "#,
        //     self.path,
        //     &self.keywords,
        //     self.description,
        //     &embe
        // )
        // .fetch_one(conn)
        // .await
        // .map_err(|e| {
        //     log::error!("{e:?}");
        //     QueryError::Query
        // })?;
        // self.id = embeddings.id;

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
        //  SELECT id, path, keywords, description, embedding
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
        // loop {
        //     let m = embeddings_rows.try_next().await ;
        //     if let Err(e) = &m {
        //         println!("em try next {e:?}");
        //         continue;
        //     }

        //     let mn = m.unwrap();
        //
        //     if mn.is_none() {
        //         break;
        //     }
        //     let row = mn.unwrap();

        //      let row_vec: Vector = row.get("embedding");
        //     res.push(GalleryEmbeddings { id: row.get("id"), path: row.get("path"), keywords: row.get("keywords"), description: row.get("description"), embedding: row_vec.to_vec() });

        // }
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

        // .map_err(|e| {
        //     log::error!("{e:?}");
        //     QueryError::Query
        // })?;

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
    async fn it_links_thumbnail_gallery() {
        let (conn, mut gallery_itm) = create_reg().await;
        let pre_id = gallery_itm.id.clone();
        let gallery_itm = gallery_itm.set_thumbnail("/some/path.jpg".into());
        let r = gallery_itm
            .link_thumbnail(&conn, "/some/path.jpg", 3, 4, "portrait".into())
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
                .is_some_and(|f| f == "/some/path.jpg")
        );
    }

    #[tokio::test]
    async fn it_creates_embeddings() {
        let (conn, _gallery_itm) = create_reg().await;

        let mut embe = GalleryEmbeddings::new("/some/thumbnail.webp".into(), vec![1.0; 512])
            .set_keywords(vec!["keyone".to_string(), "keytwo".to_string()])
            .set_description("This summary is provided by ai".into());

        embe.create(&conn).await.unwrap();

        assert!(embe.id > 0);
    }

    #[tokio::test]
    async fn it_updates_fk_embeddings() {
        let (conn, mut gallery_itm) = create_reg().await;

        let mut embe = GalleryEmbeddings::new("/some/thumbnail.webp".into(), vec![1.0; 512])
            .set_keywords(vec!["keyone".to_string(), "keytwo".to_string()])
            .set_description("This summary is provided by ai".into());

        embe.create(&conn).await.unwrap();

        gallery_itm.set_embeddings(embe.id);
        gallery_itm.link_embeddings(&conn, embe.id).await.unwrap();
    }

    #[tokio::test]
    async fn it_finds_gallery_with_embeddings() {
        let (conn, _) = create_reg().await;

        let embe = GalleryEmbeddings::find_nearest(vec![1.0; 512], &conn)
            .await
            .unwrap();
        println!("{:?}", embe);

        assert!(embe.len() > 1);
    }

    #[tokio::test]
    async fn it_update_gallery_description() {
        let (conn, _) = create_reg().await;

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
    }
}
