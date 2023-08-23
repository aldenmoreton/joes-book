use leptos::*;

#[component]
pub fn Home(
    cx: Scope,
) -> impl IntoView {

    view! {
        cx,
		<h1>"Welcome to the home page"</h1>
		<a href="/books"><h1>"Go To Books"</h1></a>
    }
}