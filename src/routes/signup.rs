use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
    Form,
};
use axum_ctx::{RespErrCtx, RespErrExt, StatusCode};

use crate::{auth::AuthSession, templates::base, AppError, AppNotification, AppStateRef};

pub async fn signup_page(
    auth_session: AuthSession,
    State(state): State<AppStateRef>,
) -> impl IntoResponse {
    if auth_session.user.is_some() {
        return Redirect::to("/").into_response();
    }

    base(
		Some("Sign Up"),
        None,
		Some(maud::html!(
            (crate::templates::alertify())
            script src="https://challenges.cloudflare.com/turnstile/v0/api.js?onload=onloadTurnstileCallback" defer {}
            script {
                "window.onloadTurnstileCallback = function () {
                    turnstile.render('#cf-turnstile-container', {
                        sitekey: '"(state.turnstile.site_key)"',
                        callback: function(token) {
                            document.getElementById('submit-button').disabled = false;
                        },
                        theme: 'light',
                        action: 'signup',
                    });
                };"
            }
        )),
		None,
		Some(maud::html!(
		div class="flex flex-col items-center justify-center pt-10" {
			div class="w-full max-w-xs" {
				form hx-post="/signup" hx-on--after-on-load="if (event.detail.xhr.status !== 200) {document.getElementById('submit-button').disabled = true;turnstile.reset('#cf-turnstile-container');}" {
                    div class="px-8 pt-6 pb-8 mb-4 bg-white rounded shadow-md" {
                        div class="mb-4" {
                            label class="block mb-2 text-sm font-bold text-gray-700" for="username" {
                                "Username"
                            }
                            input class="w-full px-3 py-2 leading-tight text-gray-700 border rounded shadow appearance-none focus:outline-none focus:shadow-outline" id="username" name="username" type="text" placeholder="Username";
                        }
                        div class="mb-6" {
                            label class="block mb-2 text-sm font-bold text-gray-700" for="password" { "Password" }
                            input class="w-full px-3 py-2 mb-3 leading-tight text-gray-700 border rounded shadow appearance-none focus:outline-none focus:shadow-outline" id="password" name="password" type="password" placeholder="Password";

                            label class="block mb-2 text-sm font-bold text-gray-700" for="password_confirmation" { "Confirm Password" }
                            input id="password_confirmation" name="password_confirmation" type="password" placeholder="Password" class="w-full px-3 py-2 mb-3 leading-tight text-gray-700 border rounded shadow appearance-none focus:outline-none focus:shadow-outline";
                        }
                        button id="submit-button" class="px-4 py-2 font-bold text-white bg-green-500 rounded disabled:cursor-wait disabled:bg-gray-400 hover:bg-green-700 focus:outline-none focus:shadow-outline" type="submit" {
                            "Sign Up"
                        }
                    }
                    div id="cf-turnstile-container" {}
				}
                div class="pt-3 text-sm font-bold" {
                    p { "Already have an account?" }
                    a class="text-green-500 hover:text-green-800" href="/login" { "Sign In" }
                }
			}
		}
    )),
	None).into_response()
}

#[derive(serde::Deserialize)]
pub struct SignUpForm {
    username: String,
    password: String,
    password_confirmation: String,
    #[serde(rename = "cf-turnstile-response")]
    pub turnstile_response: String,
}
pub async fn signup_form(
    mut auth_session: AuthSession,
    State(state): State<AppStateRef>,
    Form(form): Form<SignUpForm>,
) -> Result<impl IntoResponse, AppNotification> {
    let cf_validate: Result<cf_turnstile::SiteVerifyResponse, cf_turnstile::error::TurnstileError> =
        state
            .turnstile
            .client
            .siteverify(cf_turnstile::SiteVerifyRequest {
                response: form.turnstile_response,
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

    if form
        .username
        .chars()
        .any(|c| c.is_whitespace() || !c.is_ascii_alphanumeric())
    {
        return Err(AppNotification(
            StatusCode::BAD_REQUEST,
            "Username is not allowed".into(),
        ));
    }

    let pool = auth_session.backend.0.clone();

    if form.password != form.password_confirmation {
        return Err(AppNotification(
            StatusCode::CONFLICT,
            "Password does not match confirmation".into(),
        ));
    }

    let existing_user = sqlx::query!(
        "
		SELECT id
		FROM users
		WHERE username = $1
		",
        form.username
    )
    .fetch_optional(&pool)
    .await
    .map_err(AppError::from)?;

    if existing_user.is_some() {
        return Err(AppNotification(
            StatusCode::CONFLICT,
            "Username already taken".into(),
        ));
    }

    let user = auth_session
        .backend
        .signup(&form.username.to_lowercase(), &form.password)
        .await
        .map_err(AppError::from)?;

    auth_session
        .login(&user)
        .await
        .ctx(StatusCode::INTERNAL_SERVER_ERROR)
        .user_msg("Could not log in")?;

    Ok([("HX-Location", "/")].into_response())
}
