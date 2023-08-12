use leptos::*;
use leptos_router::Redirect;

use crate::auth::has_permission;


#[component]
pub fn Admin(cx: Scope) -> impl IntoView {
	let user = create_resource(
		cx,
		|| (),
		move |_| async move {
			has_permission(cx, "owner".into()).await.unwrap_or(false)
		}
	);

	view!{cx,
		<Suspense fallback=|| "Loading user data">
			{move || match user.read(cx) {
				Some(true) => AdminVerified(cx).into_view(cx),
				Some(false) => view! { cx, <Redirect path="/"/> }.into_view(cx),
				None => ().into_view(cx),
			}}
		</Suspense>
	}
}

#[component]
pub fn AdminVerified(cx: Scope) -> impl IntoView {
	view! {cx,
		<h1>"You must be important"</h1>
	}
}
