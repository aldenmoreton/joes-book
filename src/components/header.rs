use leptos::*;
use leptos_router::{Outlet, ActionForm};

use crate::server::Logout;

#[component]
pub fn Header() -> impl IntoView {
	let logout = create_server_action::<Logout>();

    view!{
        <div class="text-center bg-green-700">
			<a href="/" class="justify-center text-center">
				<button class="w-40 h-10 bg-green-500 rounded-md">"Home"</button>
			</a>
			<nav>
			// 	<a href="/">"Home"</a> |
			// 	<a href="/books">"Books"</a>
				<ActionForm action=logout>
					<button type="submit" class="button">"Log Out"</button>
				</ActionForm>
			</nav>
		</div>

		<Outlet/>
    }
}