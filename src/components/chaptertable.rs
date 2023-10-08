use leptos::*;
use leptos_router::use_params_map;

use crate::{server::{get_chapter_table, get_book}, objects::{BookSubscription, BookRole}};

#[component]
pub fn ChapterTable(cx: Scope) -> impl IntoView {
	let params = use_params_map(cx);
	let book_id: i64 = params.with_untracked(|params| params.get("book_id").cloned()).unwrap().parse::<i64>().unwrap();
	let chapter_id: i64 = params.with_untracked(|params| params.get("chapter_id").cloned()).unwrap().parse::<i64>().unwrap();

	let table_fetcher = create_resource(cx, || (),
		move |_| get_chapter_table(cx, chapter_id)
	);

	view!{cx,
		<a href=format!("/books/{book_id}") class="bg-green-300 border border-black rounded-md">"Back to Book"</a>
		<br/>
		<Await future=move |_| get_book(cx, book_id) bind:subscription>
			{match subscription {
				Ok(BookSubscription{role: BookRole::Owner, ..}) => view!{cx, <a href=format!("/books/{book_id}/chapters/{chapter_id}/grade") class="bg-green-200 border border-black rounded-md">"Grade Picks"</a>}.into_view(cx),
				_ => ().into_view(cx)
			}}
		</Await>
		<Transition fallback=move || "Loaading".into_view(cx)>
			{move ||
				table_fetcher
					.read(cx)
					.map(|table|
						table
							.map(|table|
								view!{cx, <div inner_html=table/>}
							)
					)
			}
		</Transition>
	}
}