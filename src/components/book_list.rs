use askama::Template;
use axum_ctx::RespErr;

use crate::{
    auth::{AuthSession, BackendPgDB},
    db::book::{get_books, BookSubscription},
    AppError,
};

#[derive(Template)]
#[template(path = "components/book_list.html")]
pub struct BookList {
    books: Vec<BookSubscription>,
}

pub async fn book_list(auth_session: AuthSession) -> Result<BookList, RespErr> {
    let BackendPgDB(pool) = auth_session.backend;
    let user = auth_session.user.ok_or(AppError::BackendUser)?;

    // let Ok(books) = get_books(user.id, &pool).await else {
    //     return "Error Getting Books".into_response();
    // };

    let books = get_books(user.id, &pool).await.map_err(AppError::from)?;

    Ok(BookList { books })
}
