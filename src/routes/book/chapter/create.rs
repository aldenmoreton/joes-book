// use axum::{http::StatusCode, response::IntoResponse};

// use crate::objects::book::BookSubscription;

// #[derive(Debug, Error)]
// pub enum Error {
//     #[error("You don't have permission to create chapter")]
//     Unauthorized,
// }

// impl IntoResponse for Error {
//     fn into_response(self) -> askama_axum::Response {
//         return (StatusCode::UNAUTHORIZED, self.to_string()).into_response();
//     }
// }

pub async fn handler(// Extension(book_subscription): Extension<BookSubscription>,
) -> Result<(), ()> {
    Ok(())
}
