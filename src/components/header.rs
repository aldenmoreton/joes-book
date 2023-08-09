use leptos::*;
use leptos_router::{Outlet, ActionForm};

use crate::{auth::get_user, components::Logout};

#[component]
pub fn Header(
    cx: Scope
) -> impl IntoView {
	let user = create_resource(
		cx,
		|| (),
		move |_| { get_user(cx) }
	);

	let logout = create_server_action::<Logout>(cx);

    view! {
        cx,
        <div>
			<Transition fallback=move || view! { cx, <p>"Loading..."</p> }>
				{move || {
					user.read(cx).map(
						|user| view! {cx,
							<p>{format!("Username: {}", user.unwrap().unwrap().username)}</p>
						}
					)
				}}
			</Transition>
			<nav>
				<a href="/">"Home"</a> |
				<a href="/books">"Books"</a>
				<ActionForm action=logout>
					<button type="submit" class="button">"Log Out"</button>
				</ActionForm>
			</nav>
			<hr/>
			<Outlet/>
        </div>
    }
}