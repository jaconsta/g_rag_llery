use errors::DbError;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

mod errors;
pub mod models;

pub type DbConn = Pool<Postgres>;

pub async fn db_connect(url: &str) -> Result<DbConn, DbError> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(url)
        .await
        .map_err(|e| {
            eprintln!("{e:?}");
            DbError::Connection
        })?;

    Ok(pool)
}

async fn create_extensions(conn: &DbConn) -> Result<(), DbError> {
    sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
        .execute(conn)
        .await
        .map_err(|e| {
            eprintln!("{e:?}");
            DbError::Extensions
        })?;
    sqlx::query("CREATE EXTENSION IF NOT EXISTS vectorscale CASCADE;")
        .execute(conn)
        .await
        .map_err(|e| {
            eprintln!("{e:?}");
            DbError::Extensions
        })?;
    Ok(())
}

async fn create_first_tables(conn: &DbConn, should_delete: Option<bool>) -> Result<(), DbError> {
    if should_delete.is_some_and(|f| f) {
        let drop_if_exists_gallery = "DROP TABLE IF EXISTS gallery CASCADE";
        let drop_if_exists_gallery_emb = "DROP TABLE IF EXISTS gallery_rag_embeddings CASCADE";
        for drop_query in [drop_if_exists_gallery, drop_if_exists_gallery_emb].into_iter() {
            sqlx::query(drop_query).execute(conn).await.map_err(|e| {
                eprintln!("{e:?}");
                DbError::Migration
            })?;
        }
    }
    let gallery_embeddings = "CREATE TABLE IF NOT EXISTS gallery_rag_embeddings(
            id bigserial primary key not null,
            path text,
            keywords text[],
            description text, 
            theme text,
            img_aria text,
            img_alt text,
            embedding vector(512) not null,
            created_at timestamptz not null default now()
        )";

    let gallery = "CREATE TABLE IF NOT EXISTS gallery(
            id uuid primary key default gen_random_uuid(),
            path text not null, 
            thumbnail_path text, 
            thumbnail_height int,
            thumbnail_width int,
            thumbnail_ratio text,
            embeddings_id bigint REFERENCES gallery_rag_embeddings(id) ON DELETE CASCADE, 
            created_at timestamptz not null default now(),
            updated_at timestamptz not null default now()
        )";

    let index_gallery_emb = "CREATE INDEX IF NOT EXISTS gallery_rag_embeddings_idx 
            ON  gallery_rag_embeddings  
            USING diskann (embedding);
    ";

    let user_upload = "CREATE TABLE IF NOT EXISTS user_upload(
            id uuid primary key default gen_random_uuid(),
            filename text not null,
            filesize int not null,
            filehash text not null,
            user_id uuid,
            created_at timestamptz not null default now(),
            updated_at timestamptz not null default now()
    )";

    // Nice to do
    // Next, create a function that sets the updated_at field to the current timestamp:
    // ```
    // CREATE OR REPLACE FUNCTION set_updated_at()
    // RETURNS TRIGGER AS $$
    // BEGIN
    //     NEW.updated_at = NOW();
    //     RETURN NEW;
    // END;
    // $$ LANGUAGE plpgsql;
    // ```
    // Then, create a trigger that executes this function before any update operation on the table:
    // ```
    // CREATE TRIGGER set_updated_at_trigger
    //     BEFORE UPDATE ON example_table
    //     FOR EACH ROW
    //     EXECUTE PROCEDURE set_updated_at();
    // ```
    for create_query in [gallery_embeddings, gallery, index_gallery_emb, user_upload].into_iter() {
        sqlx::query(create_query).execute(conn).await.map_err(|e| {
            eprintln!("{e:?}");
            DbError::Migration
        })?;
    }

    Ok(())
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    static postgres_url: &str = "postgres://postgres:postgres@localhost:5432";

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[tokio::test]
    async fn it_connects() {
        let pool = super::db_connect(postgres_url).await.unwrap();

        // Make a simple query to return the given parameter.
        let row: (i64,) = sqlx::query_as("SELECT $1")
            .bind(150_i64)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(row.0, 150);
    }

    #[tokio::test]
    async fn it_create_extensions() {
        let pool = super::db_connect(postgres_url).await.unwrap();

        super::create_extensions(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn it_create_tables() {
        let pool = super::db_connect(postgres_url).await.unwrap();

        super::create_first_tables(&pool, None).await.unwrap();
    }
}
