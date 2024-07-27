use std::{borrow::BorrowMut, collections::HashMap};

use axum::{extract::Query, Extension, Json};
use axum_ctx::{RespErr, RespErrCtx, RespErrExt, StatusCode};

use crate::{
    auth::{AuthSession, BackendPgDB},
    db::{
        chapter::Chapter,
        event::{get_events, EventContent},
        team::get_chapter_teams,
    },
    AppError,
};

pub async fn handler(
    auth_session: AuthSession,
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
) -> Result<maud::Markup, RespErr> {
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

    let (ids, event_types, contents) = events
        .into_iter()
        .map(|mut event| {
            let submission = event_submissions.remove(&event.id).ok_or(
                RespErr::new(StatusCode::BAD_REQUEST)
                    .user_msg("Not all events were included in request"),
            )?;
            match (event.contents.0.borrow_mut(), submission) {
                (
                    EventContent::SpreadGroup(ref mut spreads),
                    AnswerEventContent::SpreadGroup { mut selections },
                ) => {
                    if spreads.len() != selections.len() {
                        return Err(RespErr::new(StatusCode::BAD_REQUEST)
                            .user_msg("Spread Group Submissions do not cover all games"));
                    }
                    for spread in spreads.iter_mut().rev() {
                        spread.answer = Some(selections.pop().unwrap());
                    }
                    Ok((
                        event.id,
                        event.event_type,
                        serde_json::to_value(event.contents).unwrap(),
                    ))
                }
                (EventContent::UserInput(input), AnswerEventContent::UserInput { choices }) => {
                    input.acceptable_answers =
                        Some(choices.unwrap_or_default().into_iter().collect());
                    Ok((
                        event.id,
                        event.event_type,
                        serde_json::to_value(event.contents).unwrap(),
                    ))
                }
                _ => Err(RespErr::new(StatusCode::BAD_REQUEST)
                    .user_msg("Submitted event does not match its actual type")),
            }
        })
        .try_fold(
            (Vec::new(), Vec::new(), Vec::new()),
            |(mut ids, mut event_types, mut contents), curr_item| {
                let (curr_id, curr_type, curr_contents) = curr_item?;
                ids.push(curr_id);
                event_types.push(curr_type);
                contents.push(curr_contents);
                Ok::<_, RespErr>((ids, event_types, contents))
            },
        )?;

    sqlx::query!(
        r#"
        INSERT INTO events (id, event_type, contents)
        SELECT id, event_type, contents
        FROM UNNEST($1::INT[], $2::event_types[], $3::JSONB[]) AS a(id, event_type, contents)
        ON CONFLICT (id)
        DO UPDATE SET
            contents = EXCLUDED.contents
    "#,
        &ids,
        event_types as _,
        contents.as_slice()
    )
    .execute(&pool)
    .await
    .map_err(AppError::from)?;

    Ok(maud::html! {
        p { "Upload Successful" }
    })
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

    println!("{choices:?}");

    Ok(maud::html! {
        @for (i, choice) in choices.into_iter().enumerate() {
            p {(choice.choice)}
            input type="checkbox" name="choices" me-insert="array" value=(choice.choice) class="absolute opacity-0 peer" id={(event_id)"-"(i)} checked[choice.correct];
            label for={(event_id)"-"(i)} class="inline-grid w-full p-5 pt-0 pb-0 border border-black rounded-lg cursor-pointer hover:border-green-700 peer-checked:bg-green-500 peer-checked:border-green-600 hover:bg-green-100" {
                p {"Correct"}
            }
        }
    })
}
