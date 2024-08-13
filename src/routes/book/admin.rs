use axum::{Extension, Form};
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
                            tr class="bg-white" {
                                td class="px-6 py-4 font-medium text-gray-900 whitespace-nowrap" { (user.username) }
                                td class="px-6 py-4" { (user.role) }
                                td class="px-6 py-4" { button { "Remove" } }
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
                                    hx-trigger="input changed delay:500ms, search"
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
    user_id: String,
    username: String,
}

pub async fn add_user(
    auth_session: AuthSession,
    Extension(book_subscription): Extension<BookSubscription>,
    user_params: Form<AddUserParams>,
) -> Result<maud::Markup, RespErr> {
    let pool = auth_session.backend.0;

    let user_id = user_params
        .user_id
        .parse::<i32>()
        .map_err(|_| AppError::Parse("Could not parse user id"))?;

    let res = sqlx::query!(
        "
            INSERT INTO subscriptions (user_id, book_id, role)
            VALUES ($1, $2, 'participant')
            ON CONFLICT (user_id, book_id)
            DO NOTHING
            RETURNING user_id
        ",
        user_id,
        book_subscription.book_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(AppError::from)?;

    if res.is_none() {
        return Ok(maud::html!());
    }

    Ok(maud::html! {
        tr class="bg-white" {
            td class="px-6 py-4 font-medium text-gray-900 whitespace-nowrap" { (user_params.username) }
            td class="px-6 py-4" { "participant" }
            td class="px-6 py-4" { button { "Remove" } }
        }
    })
}
