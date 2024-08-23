use std::{borrow::BorrowMut, collections::HashMap};

use axum::{extract::Query, response::Html, Extension, Json};
use axum_ctx::{RespErr, RespErrCtx, RespErrExt, StatusCode};

use crate::{
    auth::{AuthSession, BackendPgDB},
    db::{
        book::BookSubscription,
        chapter::Chapter,
        event::{get_events, EventContent},
        team::get_chapter_teams,
    },
    AppError,
};

pub async fn handler(
    auth_session: AuthSession,
    Extension(book_subscription): Extension<BookSubscription>,
    Extension(chapter): Extension<Chapter>,
) -> Result<maud::Markup, RespErr> {
    let user = auth_session.user.ok_or(AppError::BackendUser)?;
    let BackendPgDB(pool) = auth_session.backend;

    let events = get_events(chapter.chapter_id, &pool)
        .await
        .map_err(AppError::from)?;

    let relevent_teams = get_chapter_teams(chapter.chapter_id, &pool)
        .await
        .map_err(AppError::from)?;

    Ok(crate::templates::chapter_admin::markup(
        &user.username,
        &book_subscription.name,
        chapter,
        events,
        relevent_teams,
    ))
}

#[derive(Debug, serde::Deserialize)]
pub struct AnswerSubmission {
    events: Vec<AnswerEvent>,
}

#[derive(Debug, serde::Deserialize)]
pub struct AnswerEvent {
    #[serde(rename = "event-id")]
    event_id: String,
    #[serde(flatten)]
    content: AnswerEventContent,
}

#[derive(Debug, serde::Deserialize)]
#[serde(
    tag = "type",
    rename_all(deserialize = "kebab-case"),
    rename_all_fields = "kebab-case"
)]
pub enum AnswerEventContent {
    SpreadGroup { selections: Vec<String> },
    UserInput { choices: Option<Vec<String>> },
}

pub async fn post(
    auth_session: AuthSession,
    Extension(chapter): Extension<Chapter>,
    Json(AnswerSubmission {
        events: event_submissions,
    }): Json<AnswerSubmission>,
) -> Result<Html<&'static str>, RespErr> {
    let pool = auth_session.backend.0;

    let events = get_events(chapter.chapter_id, &pool)
        .await
        .map_err(AppError::from)?;

    let mut event_submissions = event_submissions
        .into_iter()
        .map(|event| Ok((event.event_id.parse::<i32>()?, event.content)))
        .collect::<Result<HashMap<_, _>, std::num::ParseIntError>>()
        .ctx(StatusCode::BAD_REQUEST)
        .user_msg("Could not parse event id")?;

    let (ids, contents) = events
        .clone()
        .into_iter()
        .map(|mut event| {
            let submission = event_submissions.remove(&event.id).ok_or(
                RespErr::new(StatusCode::BAD_REQUEST)
                    .user_msg("Not all events were included in request"),
            )?;
            match (event.contents.0.borrow_mut(), submission) {
                (
                    EventContent::SpreadGroup(ref mut spreads),
                    AnswerEventContent::SpreadGroup { selections },
                ) => {
                    if spreads.len() != selections.len() {
                        return Err(RespErr::new(StatusCode::BAD_REQUEST)
                            .user_msg("Spread Group Submissions do not cover all games"));
                    }

                    spreads
                        .iter_mut()
                        .zip(selections.into_iter())
                        .for_each(|(spread, selection)| spread.answer = Some(selection));

                    Ok((event.id, serde_json::to_value(event.contents).unwrap()))
                }
                (EventContent::UserInput(input), AnswerEventContent::UserInput { choices }) => {
                    input.acceptable_answers =
                        Some(choices.unwrap_or_default().into_iter().collect());
                    Ok((event.id, serde_json::to_value(event.contents).unwrap()))
                }
                _ => Err(RespErr::new(StatusCode::BAD_REQUEST)
                    .user_msg("Submitted event does not match its actual type")),
            }
        })
        .try_fold(
            (Vec::new(), Vec::new()),
            |(mut ids, mut contents), curr_item| {
                let (curr_id, curr_contents) = curr_item?;
                ids.push(curr_id);
                contents.push(curr_contents);
                Ok::<_, RespErr>((ids, contents))
            },
        )?;

    let mut transaction = pool.begin().await.map_err(AppError::from)?;

    sqlx::query!(
        r#"
        UPDATE events AS e
        SET contents = c.contents
        FROM UNNEST($1::INT[], $2::JSONB[]) AS c(id, contents)
        WHERE e.id = c.id
        "#,
        &ids,
        contents.as_slice()
    )
    .execute(&mut *transaction)
    .await
    .map_err(AppError::from)?;

    sqlx::query!(
        "
        UPDATE PICKS
        SET
            POINTS = CALCULATIONS.POINTS_AWARDED
        FROM
            (
                SELECT
                    EVENT_ID,
                    USER_ID,
                    SUM(POINTS_AWARDED) AS POINTS_AWARDED
                FROM
                    (
                        SELECT
                            EVENT_ID,
                            USER_ID,
                            CASE
                                WHEN CORRECT THEN WAGER
                                ELSE 0
                            END AS POINTS_AWARDED
                        FROM
                            (
                                SELECT
                                    E.ID AS EVENT_ID,
                                    P.USER_ID,
                                    JSONB_ARRAY_ELEMENTS(P.WAGER)::INT AS WAGER,
                                    JSONB_ARRAY_ELEMENTS(E.CONTENTS -> 'spread_group') ->> 'answer' = JSONB_ARRAY_ELEMENTS(P.CHOICE) #>> '{}' AS CORRECT
                                FROM
                                    EVENTS AS E
                                    JOIN PICKS AS P ON E.ID = P.EVENT_ID
                                WHERE
                                    E.EVENT_TYPE = 'spread_group'
                                    AND E.CHAPTER_ID = $1
                            ) as t2
                    ) as t1
                GROUP BY
                    EVENT_ID,
                    USER_ID
                UNION
                SELECT
                    E.ID AS EVENT_ID,
                    P.USER_ID,
                    CASE
                        WHEN E.CONTENTS -> 'user_input' -> 'acceptable_answers' @> P.CHOICE THEN P.WAGER::INTEGER
                        ELSE 0
                    END AS POINTS_AWARDED
                FROM
                    EVENTS AS E
                    JOIN PICKS AS P ON E.ID = P.EVENT_ID
                WHERE
                    E.EVENT_TYPE = 'user_input'
                    AND E.CHAPTER_ID = $1
            ) AS CALCULATIONS
        WHERE
            PICKS.EVENT_ID = CALCULATIONS.EVENT_ID
            AND PICKS.USER_ID = CALCULATIONS.USER_ID
        ",
        chapter.chapter_id
    ).execute(&mut *transaction)
    .await
    .map_err(AppError::from)?;

    transaction.commit().await.map_err(AppError::from)?;

    Ok(Html(
        "
        <script>
            alertify.set('notifier','position', 'top-center');
            alertify.success('Answers Saved', 2);
        </script>
        ",
    ))
}

#[derive(Debug, serde::Deserialize)]
pub struct ToggleParam {
    toggle: bool,
}

pub async fn open(
    auth_session: AuthSession,
    Extension(chapter): Extension<Chapter>,
    Query(ToggleParam { toggle }): Query<ToggleParam>,
) -> Result<maud::Markup, RespErr> {
    let pool = auth_session.backend.0;

    sqlx::query!(
        r#"
        UPDATE chapters
        SET is_open = $1
        WHERE book_id = $2
    "#,
        toggle,
        chapter.book_id
    )
    .execute(&pool)
    .await
    .map_err(AppError::from)?;

    Ok(crate::templates::chapter_admin::chapter_open_button(toggle))
}

pub async fn visible(
    auth_session: AuthSession,
    Extension(chapter): Extension<Chapter>,
    Query(ToggleParam { toggle }): Query<ToggleParam>,
) -> Result<maud::Markup, RespErr> {
    let pool = auth_session.backend.0;

    sqlx::query!(
        r#"
        UPDATE chapters
        SET is_visible = $1
        WHERE book_id = $2
        RETURNING id
    "#,
        toggle,
        chapter.book_id
    )
    .fetch_one(&pool)
    .await
    .map_err(AppError::from)?;

    Ok(crate::templates::chapter_admin::chapter_visible_button(
        toggle,
    ))
}

#[derive(Debug, serde::Deserialize)]
pub struct UserInputParams {
    #[serde(rename(deserialize = "event-id"))]
    event_id: i32,
}

pub async fn user_input(
    auth_session: AuthSession,
    Query(UserInputParams { event_id }): Query<UserInputParams>,
) -> Result<maud::Markup, RespErr> {
    let BackendPgDB(pool) = auth_session.backend;

    let choices = sqlx::query!(
        r#"
            SELECT
                DISTINCT P.CHOICE #>> '{}' AS "choice!",
                coalesce(E.CONTENTS -> 'user_input' -> 'acceptable_answers' @> JSONB_BUILD_ARRAY(P.CHOICE), false) AS "correct!"
            FROM
                PICKS AS P
                JOIN EVENTS AS E ON P.EVENT_ID = E.ID
            WHERE
                P.EVENT_ID = $1
        "#,
        event_id
    )
    .fetch_all(&pool)
    .await
    .map_err(AppError::from)?;

    Ok(maud::html! {
        @for (i, choice) in choices.into_iter().enumerate() {
            input type="checkbox" name="choices" me-insert="array" value=(choice.choice) class="absolute opacity-0 peer" id={(event_id)"-"(i)} checked[choice.correct];
            label for={(event_id)"-"(i)} class="w-10/12 p-5 pt-0 pb-0 border border-black rounded-lg cursor-pointer hover:border-green-700 peer-checked:bg-green-500 peer-checked:border-green-600 hover:bg-green-100" {
                (choice.choice)
            }
        }
    })
}
