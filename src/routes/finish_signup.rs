use axum::{
    extract::State,
    response::{ErrorResponse, IntoResponse, Redirect},
    Form,
};
use axum_ctx::RespErr;
use reqwest::StatusCode;

use crate::{auth::AuthSession, templates::base, AppError, AppNotification};

use super::session::OauthProfile;

pub async fn get(
    cookie_jar: axum_extra::extract::CookieJar,
    State(state): State<crate::AppStateRef>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let pool = &state.pool;

    let signup_token = cookie_jar
        .get("signup_token")
        .ok_or(Redirect::to("/login"))?;

    let oauth_profile = sqlx::query!(
        "
		SELECT oauth.content AS content
		FROM signup_tokens
		JOIN oauth ON signup_tokens.sub = oauth.sub AND signup_tokens.provider = oauth.provider
		WHERE token = $1
		",
        signup_token.value()
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)?
    .ok_or(Redirect::to("/login"))?;

    let OauthProfile::Google(profile) = serde_json::from_value(oauth_profile.content)
        .map_err(|e| RespErr::new(StatusCode::INTERNAL_SERVER_ERROR).log_msg(e.to_string()))?;

    Ok(base(
            Some("Finish Signing Up"),
            None,
            Some(maud::html!{
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
            }),
            None,
            Some(maud::html!(
                div class="flex flex-col items-center justify-center pt-10" {
                    div class="w-full max-w-xs" {
                        form
                            hx-post="/finish-signup"
                            hx-swap="afterend"
                            hx-confirm="Are you sure you don't have an existing account? If you do please click 'Link Existing Account'."
                            hx-on--after-on-load="if (event.detail.xhr.status !== 200) {document.getElementById('submit-button').disabled = true;turnstile.reset('#cf-turnstile-container');}"
                            {
                            div class="flex flex-col items-center justify-center px-8 pt-6 pb-6 mb-4 bg-white rounded shadow-md" {
                                @if let (Some(picture), Some(first), Some(last)) = (profile.extra.get("picture"), profile.extra.get("given_name"), profile.extra.get("famly_name")) {
                                    img src=(picture) class="m-2";

                                    div class="mb-4" {
                                        label class="block mb-2 text-sm font-bold text-left text-gray-700" for="username" {
                                            "First Name"
                                        }
                                        input disabled class="w-full px-3 py-2 leading-tight text-gray-700 border rounded shadow appearance-none disabled:bg-gray-200 disabled:cursor-not-allowed focus:outline-none focus:shadow-outline" id="username" name="username" type="text" value=(first);
                                    }

                                    div class="mb-4" {
                                        label class="block mb-2 text-sm font-bold text-left text-gray-700" for="username" {
                                            "Last Name"
                                        }
                                        input disabled class="w-full px-3 py-2 leading-tight text-gray-700 border rounded shadow appearance-none disabled:bg-gray-200 disabled:cursor-not-allowed focus:outline-none focus:shadow-outline" id="username" name="username" type="text" value=(last);
                                    }
                                }


                                div class="mb-4" {
                                    label class="block mb-2 text-sm font-bold text-left text-gray-700" for="username" {
                                        "Display Name"
                                    }
                                    input class="w-full px-3 py-2 leading-tight text-gray-700 border rounded shadow appearance-none disabled:bg-gray-200 disabled:cursor-not-allowed focus:outline-none focus:shadow-outline" id="username" name="username" type="text" placeholder="Choose Display Name";
                                }

                                div id="cf-turnstile-container" {}

                                div class="flex items-center space-x-4" {
                                    a href="/legacy-login" {
                                        button type="button" class="p-1 font-bold text-black bg-white border-2 border-green-500 rounded hover:text-white hover:bg-green-700 focus:outline-none focus:shadow-outline" style="font-size: 85%;" {
                                            "Link Existing" br; "Account"
                                        }
                                    }
                                    button disabled id="submit-button" class="px-4 py-2 font-bold text-white bg-green-500 rounded disabled:cursor-wait disabled:bg-gray-400 hover:bg-green-700 focus:outline-none focus:shadow-outline" type="submit" style="font-size: 150%;" {
                                        "Sign Up"
                                    }
                                }
                            }
                        }
                    }
                }
            )),
            None,
        )
        .into_response())
}

#[derive(serde::Deserialize)]
pub struct FinishSignupForm {
    username: String,
    #[serde(rename = "cf-turnstile-response")]
    turnstile_response: String,
}

pub async fn post(
    mut auth_session: AuthSession,
    cookie_jar: axum_extra::extract::CookieJar,
    State(state): State<crate::AppStateRef>,
    Form(form): Form<FinishSignupForm>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let cf_validate: Result<cf_turnstile::SiteVerifyResponse, cf_turnstile::error::TurnstileError> =
        state
            .turnstile
            .client
            .siteverify(cf_turnstile::SiteVerifyRequest {
                response: form.turnstile_response,
                ..Default::default()
            })
            .await;

    if !cf_validate.map(|v| v.success).unwrap_or(false) {
        return Err(AppNotification(
            StatusCode::UNAUTHORIZED,
            "You did not pass our check for robots".into(),
        )
        .into());
    }

    if form.username.is_empty()
        || form
            .username
            .chars()
            .any(|c| c.is_whitespace() || !c.is_ascii_alphanumeric())
    {
        return Err(
            AppNotification(StatusCode::BAD_REQUEST, "Username is not allowed".into()).into(),
        );
    }

    let signup_token = cookie_jar
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

    let user = sqlx::query_as!(
        crate::auth::BackendUser,
        r#"
        INSERT INTO USERS (username)
        VALUES ($1)
        ON CONFLICT (username) DO NOTHING
        RETURNING id, username, password AS "pw_hash"
        "#,
        form.username
    )
    .fetch_optional(&mut *transaction)
    .await
    .map_err(AppError::from)?
    .ok_or(maud::html!(p class="text-red-500" {"Username already taken." br; "If this is your profile please click 'Link Existing Account'"}))?;

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
    .map_err(AppError::from)?;

    auth_session
        .login(&user)
        .await
        .map_err(|e| RespErr::new(StatusCode::INTERNAL_SERVER_ERROR).log_msg(e.to_string()))?;

    transaction.commit().await.map_err(AppError::from)?;

    Ok((cookie_jar.remove("signup_token"), [("HX-Location", "/")]).into_response())
}
