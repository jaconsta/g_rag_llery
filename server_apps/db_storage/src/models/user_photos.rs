use derive_getters::Getters;
use uuid::Uuid;

use crate::errors::QueryResult;

#[derive(Debug, Getters)]
pub struct UserPhoto {
    thumbnail_path: Option<String>,
    thumbnail_ratio: Option<String>,

    theme: Option<String>,
    img_alt: Option<String>,
    img_aria: Option<String>,
}

struct Counted {
    count: Option<i64>,
}

impl UserPhoto {
    pub async fn get_photos(conn: &crate::DbConn, user_id: Uuid) -> QueryResult<Vec<Self>> {
        Ok(sqlx::query_as!(UserPhoto, "SELECT g.thumbnail_path, g.thumbnail_ratio, ge.img_aria, ge.img_alt, ge.theme from gallery g join user_upload u on u.gallery_id=g.id join gallery_rag_embeddings ge on g.embeddings_id = ge.id where u.user_id=$1", user_id)
            .fetch_all(conn)
            .await?
        )
    }
    pub async fn count_photos(conn: &crate::DbConn, user_id: Uuid) -> QueryResult<i64> {
        let count = sqlx::query_as!(Counted, "SELECT count(1) from gallery g join user_upload u on u.gallery_id=g.id where u.user_id=$1", user_id)
            .fetch_one(conn)
            .await?;
        Ok(count.count.unwrap_or(0))
    }

    pub fn set_signed_url(&mut self, url: String) {
        self.thumbnail_path = Some(url);
    }
}

#[derive(Debug, Getters)]
pub struct FilterableProperties {
    aspects: Vec<String>,
    themes: Vec<String>,
}

struct FilterableProperty {
    ratio: Option<String>,
    theme: Option<String>,
}

impl From<Vec<FilterableProperty>> for FilterableProperties {
    fn from(value: Vec<FilterableProperty>) -> Self {
        FilterableProperties {
            aspects: value
                .iter()
                .as_ref()
                .iter()
                .filter(|x| x.ratio.is_some())
                .map(|x| x.ratio.clone().unwrap())
                .fold(vec![], |mut acc, ratio| {
                    if acc.contains(&ratio) {
                        acc.push(ratio.clone());
                    };

                    acc
                }),
            themes: value
                .iter()
                .as_ref()
                .iter()
                .map(|x| x.theme.clone().unwrap_or("Unthemed".into()))
                .fold(vec![], |mut acc, ratio| {
                    if acc.contains(&ratio) {
                        acc.push(ratio.clone());
                    };

                    acc
                }),
        }
    }
}

impl FilterableProperties {
    pub async fn get_for_user(conn: &crate::DbConn, user_id: Uuid) -> QueryResult<Self> {
        let filtered = sqlx::query_as!(FilterableProperty, "SELECT  distinct g.thumbnail_ratio as ratio, ge.theme as theme from gallery g join user_upload u on u.gallery_id=g.id join gallery_rag_embeddings ge on g.embeddings_id = ge.id where u.user_id=$1 group by g.thumbnail_ratio, ge.theme", user_id)
            .fetch_all(conn)
            .await?;

        Ok(filtered.into())
    }
}
