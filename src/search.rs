use axum::extract::{Query, State};

use crate::{db::team::search, AppError, AppStateRef};

#[derive(Debug, serde::Deserialize)]
pub struct TeamSearchParams {
    location: String,
    name: String,
}

pub async fn team(
    State(state): State<AppStateRef>,
    Query(TeamSearchParams { location, name }): Query<TeamSearchParams>,
) -> Result<maud::Markup, AppError<'static>> {
    if name.is_empty() {
        return Ok(crate::templates::team_search::markup(Vec::new(), &location));
    }

    let pool = &state.pool;
    let teams = search(&name, Some(10), pool).await?;

    Ok(crate::templates::team_search::markup(teams, &location))
}
