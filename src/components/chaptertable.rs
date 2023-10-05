use leptos::*;
use leptos_router::use_params_map;

use crate::server::get_chapter_table;

#[component]
pub fn ChapterTable(cx: Scope) -> impl IntoView {
	let params = use_params_map(cx);
	let chapter_id: i64 = params.with_untracked(|params| params.get("chapter_id").cloned()).unwrap().parse::<i64>().unwrap();

	let table_fetcher = create_resource(cx, || (),
		move |_| get_chapter_table(cx, chapter_id)
	);

	view!{cx,
		"Chapter table!"
		<Transition fallback=move || "Loaading".into_view(cx)>
			{
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