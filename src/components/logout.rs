use leptos::*;
use leptos_router::ActionForm;

use cfg_if::cfg_if;

cfg_if! {
	if #[cfg(feature = "ssr")] {
		use crate::components::auth;
	}
}

#[server(Logout, "/api")]
pub async fn logout(cx: Scope) -> Result<(), ServerFnError> {
    let auth = auth(cx)?;

    auth.logout_user();
    leptos_axum::redirect(cx, "/");

    Ok(())
}

#[component]
pub fn Logout(
    cx: Scope
) -> impl IntoView {
    let logout = create_server_action::<Logout>(cx);

    view! {
        cx,
        <div id="loginbox">
            <ActionForm action=logout>
                <button type="submit" class="button">"Log Out"</button>
            </ActionForm>
        </div>
    }
}