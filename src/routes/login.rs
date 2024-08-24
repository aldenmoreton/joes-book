use axum::{
    body::Body,
    extract::Query,
    http::{HeaderMap, Response, StatusCode, Uri},
    response::{IntoResponse, Redirect},
    Form,
};
use serde::Deserialize;

use crate::{
    auth::{AuthSession, LoginCreds},
    AppNotification,
};

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
) -> Result<impl IntoResponse, AppNotification> {
    let auth = auth_session.authenticate(creds).await;

    let user = match auth {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err(AppNotification(
                StatusCode::UNAUTHORIZED,
                "Invalid Username or Password".into(),
            ));
        }
        Err(_) => {
            return Err(AppNotification(
                StatusCode::REQUEST_TIMEOUT,
                "Our Fault! Please try again.".into(),
            ))
        }
    };

    if auth_session.login(&user).await.is_err() {
        return Err(AppNotification(
            StatusCode::REQUEST_TIMEOUT,
            "Our Fault! Please try again.".into(),
        ));
    }

    let desired_redirect = headers
        .get("referer")
        .and_then(|referer| referer.to_str().unwrap().parse::<Uri>().ok())
        .and_then(|uri| RedirectQuery::try_from_uri(&uri).ok())
        .map(|query: RedirectQuery| query.0.next)
        .unwrap_or("/".to_string());

    Ok([("HX-Location", desired_redirect)].into_response())
}
