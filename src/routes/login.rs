use axum::{
    body::Body,
    extract::Query,
    http::{HeaderMap, Response, StatusCode, Uri},
    response::{IntoResponse, Redirect},
    Form,
};
use axum_ctx::RespErr;
use serde::Deserialize;

use crate::auth::{AuthSession, LoginCreds};

pub async fn login_page(auth_session: AuthSession) -> Response<Body> {
    if auth_session.user.is_some() {
        return Redirect::to("/").into_response();
    }

    crate::templates::login_page::markup().into_response()
}

#[derive(Debug, Deserialize)]
struct RedirectPath {
    next: String,
}

type RedirectQuery = Query<RedirectPath>;

pub async fn login_form(
    mut auth_session: AuthSession,
    headers: HeaderMap,
    Form(creds): Form<LoginCreds>,
) -> Result<Redirect, RespErr> {
    let auth = auth_session.authenticate(creds).await;

    let user = match auth {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err(RespErr::new(StatusCode::UNAUTHORIZED).user_msg("Invalid Credentials"))
        }
        Err(_) => {
            return Err(RespErr::new(StatusCode::INTERNAL_SERVER_ERROR).user_msg("Error logging in"))
        }
    };

    if auth_session.login(&user).await.is_err() {
        return Err(RespErr::new(StatusCode::INTERNAL_SERVER_ERROR).user_msg("Error logging in"));
    }

    let desired_redirect = headers
        .get("referer")
        .and_then(|referer| referer.to_str().unwrap().parse::<Uri>().ok())
        .and_then(|uri| RedirectQuery::try_from_uri(&uri).ok())
        .map(|query: RedirectQuery| query.0.next)
        .unwrap_or("/".to_string());

    Ok(Redirect::to(&desired_redirect))
}
