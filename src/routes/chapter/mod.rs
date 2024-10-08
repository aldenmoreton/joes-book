pub mod admin;
pub mod create;
pub mod page;

pub mod mw {
    use axum::{
        body::Body,
        extract::{Path, Request},
        http::Response,
        middleware::Next,
        response::{ErrorResponse, Redirect},
    };

    use crate::{
        auth::{AuthSession, BackendPgDB},
        db::chapter::get_chapter,
    };

    #[derive(serde::Deserialize)]
    pub struct ChapterIdPath {
        book_id: i32,
        chapter_id: i32,
    }

    pub async fn chapter_ext(
        auth_session: AuthSession,
        Path(ChapterIdPath {
            chapter_id,
            book_id: _b,
        }): Path<ChapterIdPath>,
        mut request: Request,
        next: Next,
    ) -> Result<Response<Body>, ErrorResponse> {
        let BackendPgDB(pool) = auth_session.backend;

        let chapter = get_chapter(chapter_id, &pool)
            .await
            .map_err(|_| Redirect::to("/"))?;

        request.extensions_mut().insert(chapter);

        Ok(next.run(request).await)
    }
}
