use leptos::*;
use leptos_router::use_params_map;

use crate::{server::{get_book, set_open}, objects::{BookSubscription, BookRole}};

async fn foo() -> Option<String> {
	Some("Bar".to_string())
}

#[component]
pub fn ChapterTable() -> impl IntoView {
	let params = use_params_map();
	let book_id: i64 = params.with_untracked(|params| params.get("book_id").cloned()).unwrap().parse::<i64>().unwrap();
	let chapter_id: i64 = params.with_untracked(|params| params.get("chapter_id").cloned()).unwrap().parse::<i64>().unwrap();

	let table_fetcher = create_resource(|| (),
		move |_| foo()
		// move |_| get_chapter_table(chapter_id)
	);

	let chapter_opener = create_action(move |_| set_open(chapter_id, true));

	view!{
		<a href=format!("/books/{book_id}") class="bg-green-300 border border-black rounded-md">"Back to Book"</a>
		<br/>
		<Await future=move || get_book(book_id) let:subscription>
			{match subscription {
				Ok(BookSubscription{role: BookRole::Owner, ..}) => view!{
					<a href=format!("/books/{book_id}/chapters/{chapter_id}/grade") class="bg-green-200 border border-black rounded-md">"Grade Picks"</a>
					<button class="bg-green-200 border border-black rounded-md" on:click=move |_| chapter_opener.dispatch(())>"Open Chapter"</button>
				}.into_view(),
				_ => ().into_view()
			}}
		</Await>
		<Transition fallback=move || "Loading".into_view()>
			<div>
				{move ||
					table_fetcher
						.get()
						.map(|table|
							table
								.map(|table|
									view!{<div inner_html=table/>}
								)
						)
				}
			</div>
		</Transition>
	}
}