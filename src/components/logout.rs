use leptos::*;
use leptos_router::ActionForm;

use crate::server::Logout;

#[component]
pub fn Logout() -> impl IntoView {
    let logout = create_server_action::<Logout>();

    view!{
        <div id="loginbox">
            <ActionForm action=logout>
                <button type="submit" class="button">"Log Out"</button>
            </ActionForm>
        </div>
    }
}