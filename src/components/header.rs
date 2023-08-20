use leptos::*;
use leptos_router::{Outlet, ActionForm};

use crate::server::{Logout, get_username};

#[component]
pub fn Header(
    cx: Scope
) -> impl IntoView {


	let username = create_resource(
		cx,
		|| (),
		move |_| { get_username(cx) }
	);

	let logout = create_server_action::<Logout>(cx);

    view! {
        cx,
        <div>
			<Suspense fallback=move || view! { cx, <p>"Loading..."</p> }>
				{move || {
					match username.read(cx) {
						Some(Ok(username)) => view!{cx,
							<p>
								{format!("Username: {}", username)}
							</p>
						}.into_view(cx),
						_ => ().into_view(cx)
					}
				}}
			</Suspense>
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