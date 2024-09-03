use axum::{
    body::Body,
    extract::{Query, State},
    http::{HeaderMap, Response, StatusCode, Uri},
    response::{ErrorResponse, IntoResponse, Redirect},
    Form,
};
use axum_ctx::{RespErr, RespErrCtx, RespErrExt};
use axum_extra::extract::CookieJar;

use crate::{
    auth::{AuthSession, LoginCreds, UserCredentials},
    templates::base,
    AppError, AppNotification, AppStateRef,
};

pub async fn login_page(State(state): State<AppStateRef>) -> impl IntoResponse {
    base(
        Some("Joe's Book"),
        None,
        Some(maud::html!{
            link rel="stylesheet" href="/public/styles/google-sign-in.css";
        }),
        None,
        Some(maud::html! {
            div class="flex flex-col items-center justify-center h-dvh w-dvw" {
                div class="px-4 py-4 bg-white shadow-lg rounded-xl" {
                    h1 class="text-2xl font-bold" { "Sign In" }
                    div class="flex items-center justify-center my-16 h-fit" {
                        a href={"https://accounts.google.com/o/oauth2/v2/auth?scope=openid%20profile%20email&client_id="(state.google_oauth.client_id().as_str())"&response_type=code&redirect_uri=http://localhost:8000/api/auth/google"} {
                            button class="gsi-material-button" {
                                div class="gsi-material-button-state" {}
                                div class="gsi-material-button-content-wrapper" {
                                    div class="gsi-material-button-icon" {
                                        svg version="1.1" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48" xmlns:xlink="http://www.w3.org/1999/xlink" style="display: block;" {
                                            path fill="#EA4335" d="M24 9.5c3.54 0 6.71 1.22 9.21 3.6l6.85-6.85C35.9 2.38 30.47 0 24 0 14.62 0 6.51 5.38 2.56 13.22l7.98 6.19C12.43 13.72 17.74 9.5 24 9.5z" {}
                                            path fill="#4285F4" d="M46.98 24.55c0-1.57-.15-3.09-.38-4.55H24v9.02h12.94c-.58 2.96-2.26 5.48-4.78 7.18l7.73 6c4.51-4.18 7.09-10.36 7.09-17.65z" {}
                                            path fill="#FBBC05" d="M10.53 28.59c-.48-1.45-.76-2.99-.76-4.59s.27-3.14.76-4.59l-7.98-6.19C.92 16.46 0 20.12 0 24c0 3.88.92 7.54 2.56 10.78l7.97-6.19z" {}
                                            path fill="#34A853" d="M24 48c6.48 0 11.93-2.13 15.89-5.81l-7.73-6c-2.15 1.45-4.92 2.3-8.16 2.3-6.26 0-11.57-4.22-13.47-9.91l-7.98 6.19C6.51 42.62 14.62 48 24 48z" {}
                                            path fill="none" d="M0 0h48v48H0z" {}
                                        }
                                    }
                                    span class="gsi-material-button-contents" {"Continue with Google"}
                                    span style="display: none;" {"Continue with Google"}
                                }
                            }
                        }
                    }
                    div hx-target="this" {
                        p hx-get="/login/explaination" hx-swap="outerhtml" class="text-blue-400 underline hover:cursor-pointer" {"Where did the" br; "old Sign In Go?"}
                    }
                }
            }
        }),
        None,
    ).into_response()
}

pub async fn login_explaination() -> maud::Markup {
    maud::html! {
        p class="max-w-60" {
            "For security reasons, we no longer support logging in with username and password. "
            "Don't worry, you will be able to link your old account during the new login process."
        }
    }
}

pub async fn legacy_login_page(cookies: CookieJar, state: State<AppStateRef>) -> Response<Body> {
    if cookies.get("signup_token").is_none() {
        return Redirect::to("/login").into_response();
    }

    crate::templates::legacy_login_page::markup(&state.turnstile.site_key).into_response()
}

#[derive(Debug, serde::Deserialize)]
struct RedirectPath {
    next: String,
}

type RedirectQuery = Query<RedirectPath>;

pub async fn legacy_login_form(
    mut auth_session: AuthSession,
    headers: HeaderMap,
    cookies: CookieJar,
    State(state): State<AppStateRef>,
    Form(creds): Form<LoginCreds>,
) -> Result<impl IntoResponse, ErrorResponse> {
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
        )
        .into());
    }

    let signup_token = cookies
        .get("signup_token")
        .ok_or([("HX-Redirect", "/login")])?;

    let mut transaction = state.pool.begin().await.map_err(AppError::from)?;

    let oauth_profile = sqlx::query!(
        "
        DELETE FROM signup_tokens
        WHERE token = $1
        RETURNING sub, provider
        ",
        signup_token.value()
    )
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|e| AppNotification::from(AppError::from(e)))?
    .ok_or([("HX-Redirect", "/login")])?;

    let user = auth_session
        .authenticate(UserCredentials {
            username: creds.username,
            password: creds.password,
        })
        .await
        .map_err(|_| AppNotification::from(AppError::Internal))?
        .ok_or(AppNotification(
            StatusCode::UNAUTHORIZED,
            "Invalid Username or Password".into(),
        ))?;

    sqlx::query!(
        "
        UPDATE oauth
        SET user_id = $1
        WHERE sub = $2 AND provider = $3
        ",
        user.id,
        oauth_profile.sub,
        oauth_profile.provider
    )
    .execute(&mut *transaction)
    .await
    .map_err(|e| AppNotification::from(AppError::from(e)))?;

    auth_session.login(&user).await.map_err(|_| {
        AppNotification(
            StatusCode::REQUEST_TIMEOUT,
            "Our Fault! Please try again.".into(),
        )
    })?;

    let desired_redirect = headers
        .get("referer")
        .and_then(|referer| referer.to_str().unwrap().parse::<Uri>().ok())
        .and_then(|uri| RedirectQuery::try_from_uri(&uri).ok())
        .map(|query: RedirectQuery| query.0.next)
        .unwrap_or("/".to_string());

    transaction.commit().await.map_err(AppError::from)?;

    Ok((
        cookies.remove("signup_token"),
        [("HX-Location", desired_redirect)].into_response(),
    ))
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

#[derive(Debug, serde::Deserialize)]
pub enum OauthProfile {
    #[serde(rename = "google")]
    Google(google::GoogleOauth),
}

pub mod google {
    use axum::{
        extract::{Query, State},
        response::{ErrorResponse, IntoResponse, Redirect},
    };
    use axum_ctx::RespErrCtx;
    use axum_extra::extract::CookieJar;
    use oauth2::TokenResponse as _;
    use reqwest::StatusCode;

    use crate::{auth::AuthSession, AppError};

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct GoogleOauth {
        pub email: String,
        pub email_verified: bool,
        pub family_name: String,
        pub given_name: String,
        pub name: String,
        pub picture: String,
        pub sub: String,
    }

    #[derive(Debug, serde::Deserialize)]
    pub struct GoogleAuthRequest {
        code: String,
    }

    pub async fn google_oauth(
        mut auth_session: AuthSession,
        cookie_jar: CookieJar,
        State(state): State<crate::AppStateRef>,
        Query(query): Query<GoogleAuthRequest>,
    ) -> Result<impl IntoResponse, ErrorResponse> {
        let token = state
            .google_oauth
            .exchange_code(oauth2::AuthorizationCode::new(query.code))
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .map_err(|_| AppError::Internal)?;

        let profile: GoogleOauth = state
            .requests
            .get("https://openidconnect.googleapis.com/v1/userinfo")
            .bearer_auth(token.access_token().secret())
            .send()
            .await
            .map_err(|_| AppError::Internal)?
            .json()
            .await
            .map_err(|_| AppError::Internal)?;

        let pool = &state.pool;

        let user = sqlx::query_as!(
            crate::auth::BackendUser,
            "
            SELECT users.id, users.username, users.password as pw_hash
            FROM users
            JOIN oauth ON users.id = oauth.user_id
            WHERE oauth.sub = $1 AND oauth.provider = $2
            ",
            profile.sub,
            "google"
        )
        .fetch_optional(pool)
        .await
        .map_err(AppError::from)?;

        if let Some(user) = user {
            auth_session
                .login(&user)
                .await
                .ctx(StatusCode::INTERNAL_SERVER_ERROR)?;
            return Err(Redirect::to("/").into());
        }

        let content = serde_json::to_value(profile.clone()).map_err(|_| AppError::Internal)?;
        sqlx::query!(
            "
            INSERT INTO oauth(sub, provider, content)
            VALUES ($1, $2, jsonb_build_object('google', $3::JSONB))
            ON CONFLICT (sub, provider)
            DO NOTHING
            ",
            profile.sub,
            "google",
            content
        )
        .execute(pool)
        .await
        .map_err(AppError::from)?;

        let signup_token = sqlx::query!(
            "
            INSERT INTO signup_tokens(sub, provider)
            VALUES ($1, $2)
            RETURNING token
            ",
            profile.sub,
            "google"
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?
        .token;

        let cookie =
            tower_sessions::cookie::Cookie::build(("signup_token", signup_token.to_string()))
                .http_only(true)
                .same_site(tower_sessions::cookie::SameSite::Lax)
                .path("/")
                .build();

        Ok((
            cookie_jar.add(cookie).remove("book_session"),
            Redirect::to("/finish-signup"),
        )
            .into_response())
    }
}
