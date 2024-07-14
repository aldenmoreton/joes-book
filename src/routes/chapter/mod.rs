pub mod admin;
pub mod create;
pub mod page;

pub mod mw {
    use axum::{
        body::Body,
        extract::{Path, Request},
        http::{Response, StatusCode},
        middleware::Next,
    };

    use crate::{
        auth::{AuthSession, BackendPgDB},
        db::chapter::get_chapter,
    };

    pub async fn chapter_ext(
        auth_session: AuthSession,
        Path((_, chapter_id)): Path<(i32, i32)>,
        mut request: Request,
        next: Next,
    ) -> Result<Response<Body>, (StatusCode, String)> {
        let BackendPgDB(pool) = auth_session.backend;

        let Ok(chapter) = get_chapter(chapter_id, &pool).await else {
            return Err((StatusCode::NOT_FOUND, "Could not find chapter".to_string()));
        };
        request.extensions_mut().insert(chapter);

        Ok(next.run(request).await)
    }
}
