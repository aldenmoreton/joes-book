use leptos::*;
// use leptos::{component, IntoView, Scope, view};


#[component]
pub fn PickSix(cx: Scope) -> impl IntoView {
	let number_of_picks: i32 = 6;
	let elements = (1..=number_of_picks)
		.map(|n| {
			view! { cx,
				<PickSixElement id=n/>
			}
		})
		.collect_view(cx);

	view! { cx,
		<div class="border">
			<h1>"Pick Six"</h1>
			{elements}
		</div>
	}
}

#[component]
fn PickSixElement(cx: Scope, id: i32) -> impl IntoView {
	view! { cx,
		<h3>{id}</h3>
	}
}
