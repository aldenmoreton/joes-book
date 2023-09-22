use leptos::*;

#[component]
pub fn Home(
    cx: Scope,
) -> impl IntoView {

    view! {
        cx,
		<h1>"Welcome to the home page"</h1>
    <a href="/books">
        <button class="border border-black rounded-md bg-slate-100">"Go to books"
        </button>
    </a>
    }
}