use axum::{
    response::{IntoResponse, Redirect},
    Form,
};
use axum_ctx::{RespErrCtx, RespErrExt, StatusCode};

use crate::{auth::AuthSession, templates::base, AppError, AppNotification};

pub async fn signup_page(auth_session: AuthSession) -> impl IntoResponse {
    if auth_session.user.is_some() {
        return Redirect::to("/").into_response();
    }

    base(
		Some("Sign Up"),
		Some(maud::html!(
            (crate::templates::alertify())
        )),
		None,
		Some(maud::html!(
		div class="flex flex-col items-center justify-center pt-10" {
			div class="w-full max-w-xs" {
				form hx-post="/signup" class="px-8 pt-6 pb-8 mb-4 bg-white rounded shadow-md" {
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
					button class="px-4 py-2 font-bold text-white bg-green-500 rounded hover:bg-green-700 focus:outline-none focus:shadow-outline" type="submit" {
						"Sign Up"
					}
					div class="pt-3 text-sm font-bold" {
						p { "Already have an account?" }
						a class="text-green-500 hover:text-green-800" href="/login" { "Sign In" }
					}
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
}
pub async fn signup_form(
    mut auth_session: AuthSession,
    form: Form<SignUpForm>,
) -> Result<impl IntoResponse, AppNotification> {
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
        .signup(&form.username, &form.password)
        .await
        .map_err(AppError::from)?;

    auth_session
        .login(&user)
        .await
        .ctx(StatusCode::INTERNAL_SERVER_ERROR)
        .user_msg("Could not log in")?;

    Ok([("HX-Location", "/")].into_response())
}
