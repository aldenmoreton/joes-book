pub mod admin;
pub mod create;
pub mod page;

pub mod mw {
    use std::collections::HashMap;

    use axum::{
        body::Body,
        extract::{Path, Request},
        http::{Response, StatusCode},
        middleware::Next,
        Extension,
    };
    use axum_ctx::RespErr;

    use crate::{
        auth::{AuthSession, BackendPgDB},
        db::book::{get_book, BookRole, BookSubscription},
        AppError,
    };

    pub async fn require_member(
        Path(path): Path<HashMap<String, String>>,
        auth_session: AuthSession,
        mut request: Request,
        next: Next,
    ) -> Result<Response<Body>, RespErr> {
        let user = auth_session.user.ok_or(AppError::BackendUser)?;
        let BackendPgDB(pool) = auth_session.backend;

        let Some(Ok(book_id)) = path.get("book_id").map(|id| id.parse()) else {
            return Err(RespErr::new(StatusCode::BAD_REQUEST).user_msg("Could not find Book ID"));
        };

        let book_subscription = match get_book(user.id, book_id, &pool).await {
            Ok(BookSubscription {
                role: BookRole::Unauthorized,
                ..
            }) => return Err(AppError::Unauthorized("Not a member of this book".into()).into()),
            Err(_) => {
                return Err(RespErr::new(StatusCode::NOT_FOUND).user_msg("Could not find book"))
            }
            Ok(user) => user,
        };

        request.extensions_mut().insert(book_subscription);
        Ok(next.run(request).await)
    }

    pub async fn require_admin(
        Extension(book_subscription): Extension<BookSubscription>,
        request: Request,
        next: Next,
    ) -> Result<Response<Body>, RespErr> {
        if book_subscription.role != BookRole::Admin {
            return Err(AppError::Unauthorized(
                "You do not have admin privilages for this book".into(),
            )
            .into());
        }

        Ok(next.run(request).await)
    }
}
