pub mod create_book {
    use axum::{
        http::StatusCode,
        response::{IntoResponse, Redirect},
        Form,
    };
    use serde::Deserialize;

    use crate::auth::{authz::has_perm, AuthSession};

    #[derive(Deserialize)]
    pub struct Params {
        #[serde(rename(deserialize = "book-name"))]
        book_name: String,
    }

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Book Name is Invalid")]
        BookName,
        #[error("Internal Error")]
        SQLX(#[from] sqlx::Error),
        #[error("You do not have authorization to create a book")]
        Unauthorized,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> askama_axum::Response {
            match self {
                Error::Unauthorized => StatusCode::UNAUTHORIZED.into_response(),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response(),
            }
        }
    }

    pub async fn handler(
        auth_session: AuthSession,
        Form(Params { book_name }): Form<Params>,
    ) -> Result<Redirect, Error> {
        if book_name.len() > 30 || book_name.chars().any(|c| !c.is_alphabetic() && c != ' ') {
            return Err(Error::BookName);
        }

        let user = auth_session.user.unwrap();
        let pool = auth_session.backend.0;

        if !has_perm("admin", user.id, &pool).await? {
            return Err(Error::Unauthorized);
        }
        let mut transaction = pool.begin().await?;

        let record = sqlx::query!(
            "INSERT INTO books (name)
            VALUES ($1)
            ON CONFLICT DO NOTHING
            RETURNING id",
            book_name
        )
        .fetch_one(&mut *transaction)
        .await?;

        sqlx::query!(
            "INSERT INTO subscriptions (user_id, book_id, role)
            VALUES ($1, $2, $3)",
            user.id,
            record.id,
            "admin"
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;

        Ok(Redirect::to(&format!("/book/{}", record.id)))
    }
}
