use leptos::*;
use leptos_router::use_params_map;

use crate::{server::get_picks, objects::{Event, Pick}};


#[component]
pub fn Chapter(cx: Scope) -> impl IntoView {
	let params = use_params_map(cx);
	let book_id: i64 = params.with_untracked(|params| params.get("book_id").cloned()).unwrap().parse::<i64>().unwrap();
	let chapter_id: i64 = params.with_untracked(|params| params.get("chapter_id").cloned()).unwrap().parse::<i64>().unwrap();

	let pick_fetcher = create_resource(cx, || (),
		move |_| get_picks(cx, chapter_id)
	);

	view!{cx,
		<p>{book_id}</p>
		<p>{chapter_id}</p>
		<h1>"Chapter Page!"</h1>
		<Transition fallback=|| "Loading...">
			<div class="flex flex-col items-center justify-center border border-green-500">
				{move ||
					{move ||
						pick_fetcher.read(cx).map(|events| match events {
							Err(e) => {
								view! { cx, <pre class="error">"Server Error with pick fetcher: " {e.to_string()}</pre>}.into_view(cx)
							},
							Ok(events) => {
								view!{cx,
									<p>"From Server"</p>
									<p>{format!("{:?}", events)}</p>
									<ChapterEvents _inital_values=events/>
								}.into_view(cx)
							}
						})
					}
				}
			</div>
		</Transition>
	}
}

#[component]
pub fn ChapterEvents(cx: Scope, _inital_values: Vec<(Event, Pick)>) -> impl IntoView {
	// let pick_signals = inital_values
	// 	.iter()
		// .;



	view! {cx,
		<h1>"Chapter Events"</h1>
		{

		}
	}
}

