//! Library service — gRPC unary RPCs mapping onto the DB rows (behavior in data).
//! v1 is single-user: `UserId` comes from the bearer-token middleware extension; the schema
//! already carries per-user rows so multi-user (post-v1) rides on the same shape.

use tonic::{Request, Response, Status};

use crate::auth::UserId;
use crate::pb as pb_types;
pub use crate::pb::library_server::Library;

type Grpc<T> = Result<Response<T>, Status>;

#[derive(Clone)]
pub struct LibraryDb {
    pub pool: crate::DbPool,
}

fn summary_of(id: i64, source_key: &str, external_id: &str, title: &str, cover: Option<&String>, desc: Option<&String>) -> pb_types::MangaSummary {
    pb_types::MangaSummary {
        id,
        source_key: source_key.to_string(),
        external_id: external_id.to_string(),
        title: title.to_string(),
        cover_url: cover.cloned().unwrap_or_default(),
        description: desc.cloned().unwrap_or_default(),
    }
}

impl LibraryDb {
    async fn resolve_manga_row(
        &self,
        source_key: &str,
        external_id: &str,
        title: &str,
        cover: &str,
        desc: &str,
    ) -> Result<MangaRow, Status> {
        // Upsert semantics: the manga row is keyed by (source_id, source_key) UNIQUE.
        sqlx::query("INSERT OR IGNORE INTO manga (source_id, source_key, title) SELECT id, ?, ? FROM source WHERE key = ?")
            .bind(external_id)
            .bind(title)
            .bind(source_key)
            .execute(&self.pool)
            .await
            .map_err(db_err)?;
        let row: (i64, Option<String>, Option<String>) = sqlx::query_as(
            "SELECT m.id, m.title, ? , ? FROM manga m JOIN source s ON s.id = m.source_id WHERE m.source_key = ?",
        )
        .bind(if cover.is_empty() { None } else { Some(cover) })
        .bind(if desc.is_empty() { None } else { Some(desc) })
        .bind(external_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(db_err)?
        .ok_or(Status::not_found("source unknown"))?;
        Ok(MangaRow {
            id: row.0,
            title: row.1.unwrap_or_default(),
            cover: row.2,
            description: None,
        })
    }
}

struct MangaRow {
    id: i64,
    title: String,
    cover: Option<String>,
    description: Option<String>,
}

fn db_err(e: sqlx::Error) -> Status {
    Status::internal(format!("db: {e}"))
}

#[tonic::async_trait]
impl Library for LibraryDb {
    async fn add_manga(
        &self,
        request: Request<pb_types::AddMangaRequest>,
    ) -> Grpc<pb_types::MangaSummary> {
        let r = request.into_inner();
        let row = self
            .resolve_manga_row(&r.source_key, &r.external_id, &r.title, &r.cover_url, &r.description)
            .await?;

        // cover/description are not columns in the draft schema — keep them out of scope for
        // the row; the summary returns what the caller sent (source of truth: the adapter).
        if r.category_id != 0 {
            sqlx::query("INSERT OR IGNORE INTO manga_category (manga_id, category_id) VALUES (?, ?)")
                .bind(row.id)
                .bind(r.category_id)
                .execute(&self.pool)
                .await
                .map_err(db_err)?;
        }
        Ok(Response::new(summary_of(
            row.id, &r.source_key, &r.external_id, &row.title, row.cover.as_ref(), row.description.as_ref(),
        )))
    }

    async fn list_library(
        &self,
        request: Request<pb_types::ListLibraryRequest>,
    ) -> Grpc<pb_types::ListLibraryResponse> {
        let r = request.into_inner();
        let rows: Vec<(i64, String, Option<String>)> = if r.category_id == 0 {
            sqlx::query_as("SELECT m.id, m.title, m.source_key FROM manga m ORDER BY m.title")
                .fetch_all(&self.pool)
                .await
                .map_err(db_err)?
        } else {
            sqlx::query_as(
                "SELECT m.id, m.title, m.source_key FROM manga m \
                 JOIN manga_category mc ON mc.manga_id = m.id \
                 WHERE mc.category_id = ? ORDER BY m.title",
            )
            .bind(r.category_id)
            .fetch_all(&self.pool)
            .await
            .map_err(db_err)?
        };
        let items = rows
            .into_iter()
            .map(|(id, title, skey)| pb_types::MangaSummary {
                id,
                title,
                source_key: skey.unwrap_or_default(),
                external_id: String::new(),
                cover_url: String::new(),
                description: String::new(),
            })
            .collect();
        Ok(Response::new(pb_types::ListLibraryResponse { items }))
    }

    async fn get_manga(
        &self,
        request: Request<pb_types::GetMangaRequest>,
    ) -> Grpc<pb_types::MangaSummary> {
        let r = request.into_inner();
        let row: Option<(i64, String, String)> = sqlx::query_as(
            "SELECT m.id, m.title, s.key FROM manga m JOIN source s ON s.id = m.source_id WHERE m.id = ?",
        )
        .bind(r.id)
        .fetch_optional(&self.pool)
        .await
        .map_err(db_err)?;
        match row {
            Some((id, title, source_key)) => {
                Ok(Response::new(summary_of(id, &source_key, &source_key, &title, None, None)))
            }
            None => Err(Status::not_found("manga not found")),
        }
    }

    async fn create_category(
        &self,
        request: Request<pb_types::CreateCategoryRequest>,
    ) -> Grpc<pb_types::Category> {
        let r = request.into_inner();
        if r.name.trim().is_empty() {
            return Err(Status::invalid_argument("category name required"));
        }
        let res = sqlx::query("INSERT OR IGNORE INTO category (name) VALUES (?)")
            .bind(r.name.trim())
            .execute(&self.pool)
            .await
            .map_err(db_err)?;
        let id = res.last_insert_rowid();
        let id = if id == 0 {
            let (existing,): (i64,) = sqlx::query_as("SELECT id FROM category WHERE name = ?")
                .bind(r.name.trim())
                .fetch_one(&self.pool)
                .await
                .map_err(db_err)?;
            existing
        } else {
            id
        };
        Ok(Response::new(pb_types::Category { id, name: r.name.trim().to_string(), delete_after_read: false }))
    }

    async fn list_categories(
        &self,
        _request: Request<pb_types::ListCategoriesRequest>,
    ) -> Grpc<pb_types::ListCategoriesResponse> {
        let rows: Vec<(i64, String, i32)> =
            sqlx::query_as("SELECT id, name, delete_after_read FROM category ORDER BY name")
                .fetch_all(&self.pool)
                .await
                .map_err(db_err)?;
        let items = rows
            .into_iter()
            .map(|(id, name, dar)| pb_types::Category { id, name, delete_after_read: dar != 0 })
            .collect();
        Ok(Response::new(pb_types::ListCategoriesResponse { items }))
    }

    async fn add_manga_to_category(
        &self,
        request: Request<pb_types::AddToCategoryRequest>,
    ) -> Grpc<pb_types::Empty> {
        let r = request.into_inner();
        sqlx::query("INSERT OR IGNORE INTO manga_category (manga_id, category_id) VALUES (?, ?)")
            .bind(r.manga_id)
            .bind(r.category_id)
            .execute(&self.pool)
            .await
            .map_err(db_err)?;
        Ok(Response::new(pb_types::Empty {}))
    }

    async fn upsert_progress(
        &self,
        request: Request<pb_types::UpsertProgressRequest>,
    ) -> Grpc<pb_types::Empty> {
        let user = request.extensions().get::<UserId>().copied().unwrap_or(UserId(1));
        let r = request.into_inner();
        let ratio = r.scroll_ratio.clamp(0.0, 1.0);
        sqlx::query(
            "INSERT INTO reading_progress (user_id, chapter_id, page_n, scroll_ratio) VALUES (?, ?, ?, ?) \
             ON CONFLICT (user_id, chapter_id) DO UPDATE SET page_n = excluded.page_n, scroll_ratio = excluded.scroll_ratio",
        )
        .bind(user.0)
        .bind(r.chapter_id)
        .bind(r.page_n.max(0))
        .bind(ratio)
        .execute(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(Response::new(pb_types::Empty {}))
    }
}
