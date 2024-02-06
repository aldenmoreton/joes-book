use askama::Template;
use axum::{extract::Path, http::StatusCode, routing::get, Router};

use crate::{auth::{AuthSession, BackendPgDB}, objects::{chapter::{get_chapter, Chapter}, event::{get_events, Event}}};

pub fn router() -> Router {
	Router::new()
		.route("/:chapter_id", get(chapter))
}

#[derive(Template)]
#[template(path = "pages/chapter.html")]
struct ChapterPage {
	username: String,
	meta: Chapter,
	events: Vec<Event>
}

async fn chapter(auth_session: AuthSession, Path((_book_id, chapter_id)): Path<(i64, i64)>) -> Result<ChapterPage, StatusCode> {
	let user = auth_session.user.ok_or(StatusCode::UNAUTHORIZED)?;
	let BackendPgDB(pool) = auth_session.backend;

	let meta = get_chapter(chapter_id, &pool).await?;
	let events = get_events(chapter_id, &pool).await?;

	Ok(ChapterPage { username: user.username, meta, events })
}
