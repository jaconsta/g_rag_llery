use derive_getters::Getters;
use sqlx::postgres::PgRow;
use sqlx::{QueryBuilder, Row};
use uuid::Uuid;

use crate::errors::QueryError;

#[derive(Debug, Clone, Getters, sqlx::FromRow)]
pub struct UserUpload {
    id: Uuid,
    /// Bucket path of the image used for Processing
    filename: String,
    filesize: i64, // Try i32, i64 is equivalent to BIGINT
    filehash: String,
    user_id: Option<String>,
    gallery_id: Option<Uuid>,
}

impl UserUpload {
    pub async fn new_for_upload(
        conn: &crate::DbConn,
        filename: &str,
        filesize: i32,
        filehash: &str,
        user_id: &str,
    ) -> Result<UserUpload, QueryError> {
        let user_upload = sqlx::query_as!(
            UserUpload,
            r#"
        with inserted_upload as (
            insert into user_upload (filename, filesize, filehash, user_id)
            values ($1, $2, $3, $4)
            returning id, filename, filesize, filehash, user_id, gallery_id
        )
            SELECT id, filename, filesize, filehash, user_id, gallery_id 
            from inserted_upload"#,
            filename,
            filesize,
            filehash,
            user_id
        )
        .fetch_one(conn)
        .await
        .map_err(|e| {
            log::error!("{e:?}");
            QueryError::Query
        })?;

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

    pub async fn get_unprocessed(
        conn: &crate::DbConn,
        limit: Option<usize>,
    ) -> Result<Vec<UserUpload>, QueryError> {
        let mut query_b = QueryBuilder::new(
            "SELECT id, filename, filesize, filehash, user_id, gallery_id FROM user_upload WHERE gallery_id is NULL ",
        );
        if let Some(l) = limit {
            query_b.push("LIMIT ");
            query_b.push_bind(format!("%{}%", l));
        }

        let query = query_b.build();
        let user_upload: Vec<UserUpload> = query
            .fetch_all(conn)
            .await
            .map_err(|e| {
                log::error!("{e:?}");
                QueryError::Query
            })?
            .iter()
            .map(|row| row.into())
            .collect();

        Ok(user_upload)
    }
}

impl From<&PgRow> for UserUpload {
    fn from(row: &PgRow) -> Self {
        UserUpload {
            id: row.get("id"),
            filename: row.get("filename"),
            filesize: (row.get::<i32, &str>("filesize")).into(),
            filehash: row.get("filehash"),
            user_id: row.get("user_id"),
            gallery_id: row.get("gallery_id"),
        }
    }
}
