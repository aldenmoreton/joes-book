use askama::Template;
use axum::{
    extract::Query,
    http::{HeaderMap, StatusCode, Uri},
    response::{IntoResponse, Redirect},
    routing::get,
    Form, Router,
};
use serde::Deserialize;

use crate::auth::{AuthSession, LoginCreds};

pub fn router() -> Router {
    Router::new().route("/", get(login_page).post(login_form))
}

#[derive(Template)]
#[template(path = "pages/login.html")]
struct LoginPage;

async fn login_page(auth_session: AuthSession) -> impl IntoResponse {
    if auth_session.user.is_some() {
        return Redirect::to("/").into_response();
    }

    LoginPage {}.into_response()
}

#[derive(Debug, Deserialize)]
struct RedirectPath {
    next: String,
}

type RedirectQuery = Query<RedirectPath>;

async fn login_form(
    mut auth_session: AuthSession,
    headers: HeaderMap,
    Form(creds): Form<LoginCreds>,
) -> impl IntoResponse {
    let auth = auth_session.authenticate(creds).await;

    let user = match auth {
        Ok(Some(user)) => user,
        Ok(None) => return StatusCode::UNAUTHORIZED.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if auth_session.login(&user).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let desired_redirect = headers
        .get("referer")
        .and_then(|referer| referer.to_str().unwrap().parse::<Uri>().ok())
        .and_then(|uri| RedirectQuery::try_from_uri(&uri).ok())
        .map(|query: RedirectQuery| query.0.next)
        .unwrap_or("/".to_string());

    Redirect::to(&desired_redirect).into_response()
}
