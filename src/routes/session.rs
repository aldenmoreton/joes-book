use axum::{
    body::Body,
    extract::{Query, State},
    http::{HeaderMap, Response, StatusCode, Uri},
    response::{IntoResponse, Redirect},
    Form,
};
use axum_ctx::{RespErr, RespErrCtx, RespErrExt};
use serde::Deserialize;

use crate::{
    auth::{AuthSession, LoginCreds, UserCredentials},
    AppNotification, AppStateRef,
};

pub async fn login_page(auth_session: AuthSession, state: State<AppStateRef>) -> Response<Body> {
    if auth_session.user.is_some() {
        return Redirect::to("/").into_response();
    }

    crate::templates::login_page::markup(&state.turnstile.site_key).into_response()
}

#[derive(Debug, Deserialize)]
struct RedirectPath {
    next: String,
}

type RedirectQuery = Query<RedirectPath>;

pub async fn login_form(
    mut auth_session: AuthSession,
    headers: HeaderMap,
    State(state): State<AppStateRef>,
    Form(creds): Form<LoginCreds>,
) -> Result<impl IntoResponse, AppNotification> {
    let cf_validate: Result<cf_turnstile::SiteVerifyResponse, cf_turnstile::error::TurnstileError> =
        state
            .turnstile
            .client
            .siteverify(cf_turnstile::SiteVerifyRequest {
                response: creds.turnstile_response,
                ..Default::default()
            })
            .await;

    tracing::debug!("{cf_validate:?}");

    if !cf_validate.map(|v| v.success).unwrap_or(false) {
        return Err(AppNotification(
            StatusCode::UNAUTHORIZED,
            "You did not pass our check for robots".into(),
        ));
    }

    let auth = auth_session
        .authenticate(UserCredentials {
            username: creds.username,
            password: creds.password,
        })
        .await;

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

pub async fn logout(mut auth_session: self::AuthSession) -> Result<Response<Body>, RespErr> {
    auth_session
        .logout()
        .await
        .ctx(StatusCode::INTERNAL_SERVER_ERROR)
        .log_msg("Could not log out user")
        .user_msg("Logout unsuccessful")?;

    Ok([("HX-Redirect", "/login")].into_response())
}
