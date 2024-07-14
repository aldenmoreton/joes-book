use axum::extract::Query;
use axum_ctx::RespErr;

use crate::{auth::AuthSession, db::team::search, AppError};

#[derive(Debug, serde::Deserialize)]
pub struct Params {
    location: String,
    name: String,
}

pub async fn handler(
    auth_session: AuthSession,
    Query(Params { location, name }): Query<Params>,
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
