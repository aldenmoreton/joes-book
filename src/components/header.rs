use leptos::*;
use leptos_router::{A, Outlet};

use crate::auth::get_user;

#[component]
pub fn Header(
    cx: Scope
) -> impl IntoView {
	let user = create_resource(
		cx,
		|| (),
		move |_| { get_user(cx) }
	);

    view! {
        cx,
        <div>
			<Transition fallback=move || view! { cx, <p>"Loading..."</p> }>
				{move || {
					user.read(cx).map(
						|user| view! {cx,
							<A href="/"><h3>"Home"</h3></A>
							<p>{format!("Username: {}", user.unwrap().unwrap().username)}</p>
							<A href="logout"><p>"Logout"</p></A>
							<hr/>
							<Outlet/>
						}
					)
				}}
			</Transition>
        </div>
    }
}