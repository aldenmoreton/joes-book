use axum::{extract::State, Extension};

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
) -> Result<maud::Markup, AppError<'static>> {
    let user = auth_session.user.ok_or(AppError::BackendUser)?;
    let BackendPgDB(pool) = auth_session.backend;

    let chapters = get_chapters(book_subscription.id, &pool).await?;
    let guest_chapters = if let BookRole::Guest { ref chapter_ids } = book_subscription.role {
        Some(chapter_ids)
    } else {
        None
    };

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
                        (chapter_list::markup(book_subscription.id, chapters.iter().filter(|c| !c.is_visible).peekable()))
                    }
                }
            }

            div class="flex items-center justify-center" {
                details class="flex items-center w-max" {
                    summary class="p-3 my-1 align-middle bg-green-500 rounded-lg shadow-md select-none" {
                        "Leaderboard"
                    }
                    div hx-get="leaderboard" hx-trigger="load" hx-swap="outerhtml" class="flex items-center" {
                        "Loading..."
                    }
                }
            }

            @if let Some(guest_chapters) = guest_chapters {
                (chapter_list::markup(book_subscription.id, chapters.iter().filter(|c| c.is_visible && guest_chapters.contains(&c.chapter_id)).peekable()))
            } @else {
                (chapter_list::markup(book_subscription.id, chapters.iter().filter(|c| c.is_visible).peekable()))
            }
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
            USERNAME AS "username!",
            EARNED_POINTS::INT AS "earned_points!",
            ADDED_POINTS::INT AS "added_points!",
            EARNED_POINTS::INT + ADDED_POINTS::INT AS "total_points!"
        FROM
            (
                -- Join user earned points and added points
                SELECT
                    USERNAME,
                    COALESCE(SUM(EARNED_POINTS), 0) AS EARNED_POINTS,
                    COALESCE(SUM(ADDED_POINTS), 0) AS ADDED_POINTS
                FROM
                    (
                        SELECT
                            USER_ID,
                            USERNAME,
                            SUM(EARNED_POINTS) AS EARNED_POINTS
                        FROM
                            (
                                -- Get earned points
                                SELECT
                                    BOOK_USERS.USER_ID,
                                    COALESCE(USER_GROUPS.NAME, BOOK_USERS.USERNAME) AS USERNAME,
                                    COALESCE(PICK_POINTS.POINTS, 0) AS EARNED_POINTS
                                FROM
                                    -- Get users in group
                                    (
                                        SELECT
                                            USERS.ID AS USER_ID,
                                            USERS.USERNAME
                                        FROM
                                            USERS
                                            JOIN SUBSCRIPTIONS ON USERS.ID = SUBSCRIPTIONS.USER_ID
                                        WHERE
                                            SUBSCRIPTIONS.BOOK_ID = $1
                                    ) AS BOOK_USERS
                                    -- Get earned points
                                    LEFT JOIN (
                                        SELECT
                                            PICKS.USER_ID,
                                            PICKS.POINTS
                                        FROM
                                            PICKS
                                        WHERE
                                            PICKS.BOOK_ID = $1
                                    ) AS PICK_POINTS ON BOOK_USERS.USER_ID = PICK_POINTS.USER_ID
                                    -- Get user groups
                                    LEFT JOIN (
                                        SELECT
                                            SUBSCRIPTION_GROUPS.USER_ID,
                                            SUBSCRIPTION_GROUPS.NAME
                                        FROM
                                            SUBSCRIPTION_GROUPS
                                        WHERE
                                            SUBSCRIPTION_GROUPS.BOOK_ID = $1
                                    ) AS USER_GROUPS ON BOOK_USERS.USER_ID = USER_GROUPS.USER_ID
                            ) AS EARNED_POINTS
                        GROUP BY
                            USER_ID,
                            USERNAME
                    ) AS GROUPED_EARNED_POINTS
                    -- Get added points
                    LEFT JOIN (
                        SELECT
                            USER_ID,
                            SUM(ADDED_POINTS.POINTS) AS ADDED_POINTS
                        FROM
                            ADDED_POINTS
                        WHERE
                            ADDED_POINTS.BOOK_ID = $1
                        GROUP BY
                            USER_ID
                    ) AS ADDED_POINTS ON GROUPED_EARNED_POINTS.USER_ID = ADDED_POINTS.USER_ID
                GROUP BY
                    GROUPED_EARNED_POINTS.USERNAME
            ) AS SUB1
        ORDER BY
            "total_points!" DESC
        "#,
        book_subscription.id
    )
    .fetch_all(pool)
    .await?;

    Ok(maud::html! {
        div class="flex justify-center w-max" {
            table class="text-sm w-max" {
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
                            td class="px-6 py-4" {
                                (rank.username)
                                br;
                                @if rank.added_points > 0 {
                                    span class="text-red-500" {"Added Points: "(rank.added_points)}
                                }
                            }
                            td class="px-6 py-4" {(rank.total_points)}
                        }
                    }
                }
            }
        }
    })
}
