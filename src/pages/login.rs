use askama::Template;
use serde::Deserialize;
use axum_login::AuthSession;
use axum::{response::{IntoResponse, Redirect}, Router, routing::get, extract::Query, Form, http::{HeaderMap, Uri, StatusCode}};

use crate::auth::{BackendPgDB, LoginCreds};

pub fn router() -> Router {
	Router::new()
		.route("/", get(login_page).post(login_form))
}

#[derive(Template)]
#[template(path = "login.html")]
struct Login;

async fn login_page() -> impl IntoResponse {
	Login{}
}

#[derive(Debug, Deserialize)]
struct RedirectPath {
	next: String
}

type RedirectQuery = Query<RedirectPath>;

async fn login_form(
	mut auth_session: AuthSession<BackendPgDB>,
	headers: HeaderMap,
	Form(creds): Form<LoginCreds>,
) -> impl IntoResponse {
	let auth = auth_session.authenticate(creds).await;
	println!("{auth:?}");
	let user = match auth {
        Ok(Some(user)) => user,
        Ok(None) => return StatusCode::UNAUTHORIZED.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response()
    };

    if auth_session.login(&user).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

	let desired_redirect = headers
		.get("referer")
		.and_then(|referer| Some(referer.to_str().unwrap().parse::<Uri>().unwrap()))
		.and_then(|uri| RedirectQuery::try_from_uri(&uri).ok())
		.and_then(|query: RedirectQuery| Some(query.0.next))
		.unwrap_or("/".to_string());

	Redirect::to(&desired_redirect).into_response()
}
