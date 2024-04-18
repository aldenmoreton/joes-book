use axum::{http::StatusCode, response::IntoResponse, Extension};
use thiserror::Error;

use crate::objects::book::{BookRole, BookSubscription};

#[derive(Debug, Error)]
pub enum Error {
    #[error("You don't have permission to create chapter")]
    Unauthorized,
}

impl IntoResponse for Error {
    fn into_response(self) -> askama_axum::Response {
        return (StatusCode::UNAUTHORIZED, self.to_string()).into_response();
    }
}

pub async fn handler(
    Extension(book_subscription): Extension<BookSubscription>,
) -> Result<(), Error> {
    if book_subscription.role != BookRole::Admin {
        return Err(Error::Unauthorized);
    }

    Ok(())
}
