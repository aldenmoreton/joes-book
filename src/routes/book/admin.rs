use axum::{extract::Query, Extension, Form};
use axum_ctx::RespErr;

use crate::{auth::AuthSession, db::book::BookSubscription, templates::authenticated, AppError};

pub async fn handler(
    auth_session: AuthSession,
    Extension(book_subscription): Extension<BookSubscription>,
) -> Result<maud::Markup, RespErr> {
    let user = auth_session.user.ok_or(AppError::BackendUser)?;
    let pool = auth_session.backend.0;

    let users = sqlx::query!(
        "
        SELECT u.id, u.username, s.role
        FROM users AS u
        JOIN subscriptions AS s ON u.id=s.user_id
        JOIN books AS b on s.book_id=b.id
        WHERE b.id = $1 AND u.id != $2
        ",
        book_subscription.book_id,
        book_subscription.user_id
    )
    .fetch_all(&pool)
    .await
    .map_err(AppError::from)?;

    Ok(authenticated(
        &user.username,
        Some(format!("{} - Admin", book_subscription.name).as_str()),
        None,
        None,
        Some(maud::html! {
            div class="flex flex-col items-center justify-center" {
                div class="relative mt-5 overflow-x-auto rounded-lg" {
                table class="w-full text-sm text-left text-gray-500 rtl:text-right" {
                    thead class="text-xs text-gray-700 uppercase bg-gray-100" {
                        tr {
                            th scope="col" class="px-6 py-3 rounded-s-lg" { "username" }
                            th scope="col" class="px-6 py-3" { "status" }
                            th scope="col" class="px-6 py-3 rounded-e-lg" { "action" }
                        }
                    }

                    tbody {
                        tr class="bg-white" {
                            td scope="row" class="px-6 py-4 font-medium text-gray-900 whitespace-nowrap" { (user.username) }
                            td class="px-6 py-4" { (format!("{:?}", book_subscription.role)) }
                            td class="px-6 py-4" { button { "Heavy is The Head" br; "That Wears The Crown" } }
                        }

                        @for user in users {
                            tr class="bg-white" hx-target="this" {
                                td class="px-6 py-4 font-medium text-gray-900 whitespace-nowrap" { (user.username) }
                                td class="px-6 py-4" { (user.role) }
                                td class="px-6 py-4" { button hx-post="remove-user" hx-vals={r#"{"user_id":""#(user.id)r#""}"#} class="px-2 py-2 mt-1 font-bold text-white bg-orange-600 rounded hover:bg-orange-700" { "Remove" } }
                            }
                        }
                    }
                    tfoot {
                        tr class="font-semibold text-gray-900 bg-green-400" {
                            th scope="row" class="px-6 py-3 text-base" { "Add Member" }
                            th colspan="2" {
                                input
                                    name="username"
                                    hx-get="user-search"
                                    hx-trigger="input changed delay:200ms, search"
                                    hx-target="next ul"
                                    type="search"
                                    autocomplete="off"
                                    placeholder="username";
                               ul {}
                            }
                        }
                    }
                }
                }
            }
        }),
        None,
    ))
}

#[derive(serde::Deserialize)]
pub struct AddUserParams {
    user_id: i32,
    username: String,
}

pub async fn add_user(
    auth_session: AuthSession,
    Extension(book_subscription): Extension<BookSubscription>,
    user_params: Form<AddUserParams>,
) -> Result<maud::Markup, RespErr> {
    let pool = auth_session.backend.0;

    // let user_id = user_params
    //     .user_id
    //     .parse::<i32>()
    //     .map_err(|_| AppError::Parse("Could not parse user id"))?;

    let res = sqlx::query!(
        "
            INSERT INTO subscriptions (user_id, book_id, role)
            VALUES ($1, $2, 'participant')
            ON CONFLICT (user_id, book_id)
            DO NOTHING
            RETURNING user_id
        ",
        user_params.user_id,
        book_subscription.book_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(AppError::from)?;

    if res.is_none() {
        return Ok(maud::html!());
    }

    Ok(maud::html! {
        tr class="bg-white" hx-target="this" {
            td class="px-6 py-4 font-medium text-gray-900 whitespace-nowrap" { (user_params.username) }
            td class="px-6 py-4" { "participant" }
            td class="px-6 py-4" { button hx-post="remove-user" hx-vals={r#"{"user_id":""#(user_params.user_id)r#""}"#} class="px-2 py-2 mt-1 font-bold text-white bg-orange-600 rounded hover:bg-orange-700" { "Remove" } }
        }
    })
}

#[derive(Debug, serde::Deserialize)]
pub struct UserSearchParams {
    username: String,
}

pub async fn search_user(
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
                    hx-on-click=r#"document.querySelector('input[type="search"]').value=""; document.querySelector('ul').innerHTML="";"#
                    hx-swap="beforeend" {
                        (user.username)
                    }
            }
        }
    ))
}

#[derive(serde::Deserialize)]
pub struct RemoveUserForm {
    user_id: i32,
}

pub async fn remove_user(
    auth_session: AuthSession,
    book: Extension<BookSubscription>,
    form: Form<RemoveUserForm>,
) -> Result<(), RespErr> {
    let pool = auth_session.backend.0;

    sqlx::query!(
        "
        DELETE FROM subscriptions
        WHERE user_id = $1 AND book_id = $2
        ",
        form.user_id,
        book.book_id
    )
    .execute(&pool)
    .await
    .map_err(AppError::from)?;

    Ok(())
}
