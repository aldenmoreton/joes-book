use leptos::*;
use leptos_router::ActionForm;

use crate::server::Logout;

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