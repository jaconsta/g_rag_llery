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
}
