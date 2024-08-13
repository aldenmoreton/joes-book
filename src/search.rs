use axum::{extract::Query, Extension};
use axum_ctx::RespErr;

use crate::{
    auth::AuthSession,
    db::{book::BookSubscription, team::search},
    AppError,
};

#[derive(Debug, serde::Deserialize)]
pub struct TeamSearchParams {
    location: String,
    name: String,
}

pub async fn team(
    auth_session: AuthSession,
    Query(TeamSearchParams { location, name }): Query<TeamSearchParams>,
) -> Result<maud::Markup, RespErr> {
    if name.is_empty() {
        return Ok(crate::templates::team_search::markup(Vec::new(), &location));
    }

    let pool = auth_session.backend.0;
    let teams = search(&name, Some(10), &pool)
        .await
        .map_err(AppError::from)?;

    Ok(crate::templates::team_search::markup(teams, &location))
}

#[derive(Debug, serde::Deserialize)]
pub struct UserSearchParams {
    username: String,
}

pub async fn user(
    auth_session: AuthSession,
    Query(UserSearchParams {
        username: search_username,
    }): Query<UserSearchParams>,
    Extension(book_subscription): Extension<BookSubscription>,
) -> Result<maud::Markup, RespErr> {
    let pool = auth_session.backend.0;

    if search_username.is_empty() {
        return Ok(maud::html!());
    }

    let matching_users = sqlx::query!(
        "
            SELECT u.id, u.username
            FROM users AS u
            LEFT JOIN (
                SELECT *
                FROM subscriptions
                WHERE subscriptions.book_id = $2
            ) AS s ON u.id = s.user_id
            WHERE LOWER(u.username) LIKE '%' || LOWER($1) || '%' AND s.user_id IS NULL
            ",
        search_username,
        book_subscription.book_id
    )
    .fetch_all(&pool)
    .await
    .map_err(AppError::from)?;

    Ok(maud::html!(
        @for user in matching_users {
            li {
                button
                    name="username"
                    value=(user.username)
                    hx-post={"/book/"(book_subscription.book_id)"/admin/add-user"}
                    hx-vals={r#"{"user_id":""#(user.id)r#""}"#}
                    hx-target="previous tbody"
                    hx-on-click=r#"document.querySelector('input[type="search"]').value=""; document.querySelector('ul').innerHTML="""#
                    hx-swap="beforeend" {
                        (user.username)
                    }
            }
        }
    ))
}
