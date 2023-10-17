use leptos::*;
// use leptos::{component, IntoView, Scope, view};


#[component]
pub fn PickSix() -> impl IntoView {
	let number_of_picks: i32 = 6;
	let elements = (1..=number_of_picks)
		.map(|n| {
			view!{
				<PickSixElement id=n/>
			}
		})
		.collect_view();

	view!{
		<div class="border">
			<h1>"Pick Six"</h1>
			{elements}
		</div>
	}
}

#[component]
fn PickSixElement(id: i32) -> impl IntoView {
	view!{
		<h3>{id}</h3>
	}
}
