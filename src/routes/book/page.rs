use axum::{extract::State, Extension};
use axum_ctx::RespErr;

use crate::{
    auth::{AuthSession, BackendPgDB},
    db::{
        book::{BookRole, BookSubscription},
        chapter::get_chapters,
    },
    templates::chapter_list,
    AppError, AppStateRef,
};

pub async fn handler(
    auth_session: AuthSession,
    Extension(book_subscription): Extension<BookSubscription>,
) -> Result<maud::Markup, RespErr> {
    let user = auth_session.user.ok_or(AppError::BackendUser)?;
    let BackendPgDB(pool) = auth_session.backend;

    let chapters = get_chapters(book_subscription.book_id, &pool)
        .await
        .map_err(AppError::from)?;

    Ok(crate::templates::authenticated(
        &user.username,
        Some(&book_subscription.name),
        None,
        None,
        Some(maud::html! {
            p {
                a href="/" class="text-blue-400 hover:underline" {"Home"} " > "
                a { (book_subscription.name) }
            }
        }),
        Some(maud::html! {
            h1 class="text-4xl font-extrabold" {(book_subscription.name)}
            @if book_subscription.role == BookRole::Admin {
                div class="flex justify-center" {
                    fieldset class="w-1/2 border border-orange-600" {
                        legend class="ml-3" { "Admin Section" }
                        a href="chapter/create/" {
                            button class="px-2 py-2 mt-1 font-bold text-white bg-orange-600 rounded hover:bg-orange-700" {
                                "Create New Chapter"
                            }
                        }
                        br;
                        a href="admin/" {
                            button class="px-2 py-2 mt-1 font-bold text-white bg-orange-600 rounded hover:bg-orange-700" {
                                "Admin"
                            }
                        }
                        (chapter_list::markup(book_subscription.book_id, chapters.iter().filter(|c| !c.is_visible).peekable()))
                    }
                }
            }

            div class="flex items-center justify-center" {
                details class="w-60" {
                    summary class="p-3 my-1 align-middle bg-green-500 rounded-lg shadow-md select-none" {
                        "Leaderboard"
                    }
                    div hx-get="leaderboard" hx-trigger="load" hx-swap="outerhtml" {
                        "Loading..."
                    }
                }
            }

            (chapter_list::markup(book_subscription.book_id, chapters.iter().filter(|c| c.is_visible).peekable()))
        }),
        None,
    ))
}

pub async fn leaderboard(
    State(state): State<AppStateRef>,
    book_subscription: Extension<BookSubscription>,
) -> Result<maud::Markup, AppError<'static>> {
    let pool = &state.pool;

    let rankings = sqlx::query!(
        r#"
        SELECT
            SUB1.USERNAME,
            COALESCE(SUM(POINTS), 0)::INT AS "point_total!"
        FROM
            (
                SELECT
                    USERS.ID AS USER_ID,
                    USERS.USERNAME
                FROM
                    USERS
                    JOIN SUBSCRIPTIONS ON USERS.ID = SUBSCRIPTIONS.USER_ID
                WHERE
                    SUBSCRIPTIONS.BOOK_ID = $1
            ) AS SUB1
            LEFT JOIN (
                SELECT
                    PICKS.USER_ID,
                    PICKS.POINTS
                FROM
                    PICKS
                WHERE
                    PICKS.BOOK_ID = $1
            ) AS SUB2 ON SUB1.USER_ID = SUB2.USER_ID
        GROUP BY
            SUB1.USER_ID,
            SUB1.USERNAME
        ORDER BY
            "point_total!" DESC,
            USERNAME
        "#,
        book_subscription.book_id
    )
    .fetch_all(pool)
    .await?;

    Ok(maud::html! {
        div class="flex justify-center align-middle" {
            table class="text-sm max-w-30" {
                thead class="text-xs text-gray-700 uppercase bg-green-400" {
                    tr {
                        th scope="col" class="px-6 py-3" { "Rank" }
                        th scope="col" class="px-6 py-3" { "User" }
                        th scope="col" class="px-6 py-3" { "Total Points" }
                    }
                }

                tbody {
                    @for (i, rank) in rankings.iter().enumerate() {
                        tr class="bg-white" {
                            td class="px-6 py-4" {(i+1)}
                            td class="px-6 py-4" {(rank.username)}
                            td class="px-6 py-4" {(rank.point_total)}
                        }
                    }
                }
            }
        }
    })
}
