use axum::extract::Query;
use axum_ctx::RespErr;

use crate::{
    auth::AuthSession,
    db::team::{search, Team},
    AppError,
};

#[derive(Debug, serde::Deserialize)]
pub struct Params {
    location: String,
    name: String,
}

#[derive(askama::Template)]
#[template(path = "components/team_search_results.html")]
pub struct SearchResults {
    location: String,
    teams: Vec<Team>,
}

pub async fn handler(
    auth_session: AuthSession,
    Query(Params { location, name }): Query<Params>,
) -> Result<SearchResults, RespErr> {
    if name.is_empty() {
        return Ok(SearchResults {
            location,
            teams: Vec::new(),
        });
    }

    let pool = auth_session.backend.0;
    let teams = search(&name, Some(10), &pool)
        .await
        .map_err(AppError::from)?;
    Ok(SearchResults { location, teams })
}
