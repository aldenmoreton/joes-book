use axum::Extension;
use axum_ctx::RespErr;

use crate::{
    auth::{AuthSession, BackendPgDB},
    db::{chapter::Chapter, event::get_events, team::get_chapter_teams},
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

pub async fn update(body: String) {
    println!("Update body: {body}")
}
