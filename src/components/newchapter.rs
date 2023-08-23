use leptos::*;
use leptos_router::{use_params_map, Redirect};

use crate::{
	server::get_book,
	objects::{BookSubscription, BookRole, EventBuilder, Team, SpreadBuilder}, components::TeamSelect
};

#[component]
pub fn NewChapter(cx: Scope) -> impl IntoView {
	let params = use_params_map(cx);
	let book_id:i64 = params.with(|params| params.get("id").cloned()).unwrap().parse::<i64>().unwrap();
	let user_subscription = create_resource(
		cx,
		|| (),
		move |_| async move {
			get_book(cx, book_id).await
		}
	);

	view!{cx,
		<Suspense fallback=|| "Loading user data">
			{move || match user_subscription.read(cx) {
				Some(Ok(BookSubscription{role: BookRole::Owner, ..})) => VerifiedNewChapter(cx).into_view(cx),
				None => ().into_view(cx),
				_ => view! { cx, <Redirect path={format!("/books/{book_id}")}/> }.into_view(cx),
			}}
		</Suspense>
	}
}

#[component]
pub fn VerifiedNewChapter(cx: Scope) -> impl IntoView {
	let params = use_params_map(cx);
	let book_id:i64 = params.with(|params| params.get("id").cloned()).unwrap().parse::<i64>().unwrap();

	let (events, set_events) = create_signal::<Vec<(i64, RwSignal<EventBuilder>)>>(cx, Vec::new());

	let add_event = move |event: &str| {
		let new_event = match event {
			"SpreadGroup" => {
				let new_spread = create_rw_signal(cx, SpreadBuilder::default());
				let new_tuple = (0, new_spread);
				let new_group = create_rw_signal(cx, vec![new_tuple]);
				create_rw_signal(cx, EventBuilder::SpreadGroup(new_group))
			},
			_ => panic!()
		};

		set_events.update(move |events| {
			let next_id = if let Some(last_event) = events.last() {
				last_event.0 + 1
			} else { 0 };
			events.push((next_id, new_event))
		})
	};

	view! {cx,
		<div class="justify-center grid auto-cols-auto">
			<For each={move || events.get()} key={|event| event.0}
				view=move |cx, (_, event)| {
					let new_group = move || match event.get() {
						EventBuilder::SpreadGroup(spreads) => view!{cx,
							<NewSpreadGroup spreads/>
						}
					};

					view! {cx,
						<div class="border border-black">
							{new_group}
						</div>
					}
				}/>
			<button on:click=move |_| add_event("SpreadGroup") class="bg-transparent hover:bg-blue-500 text-blue-700 font-semibold hover:text-white py-2 px-4 border border-blue-500 hover:border-transparent rounded">"Add Spread Group"</button>
		</div>
	}
}

#[component]
pub fn NewSpreadGroup(cx: Scope, spreads: RwSignal<Vec<(i64, RwSignal<SpreadBuilder>)>>) -> impl IntoView {
	let add_spread = move |_| {
		spreads.update(|spreads| {
				let new_id = if let Some(spread) = spreads.last() {
					spread.0 + 1
				} else {
					0
				};
				let new_spread = (new_id, create_rw_signal(cx, SpreadBuilder::default()));
				spreads.push(new_spread)
			}
		);
	};

	view!{cx,
		<h1 class="text-center">"Spread Group"</h1>
		<div class="border border-black grid auto-cols-auto">
			<For each={move || spreads.get().into_iter().enumerate()} key={|spread| spread.0}
				view=move |cx, (i, (_, spread))| {
					view!{cx,
						<h2>"Spread " {i + 1}</h2>
						<NewSpread spread/>
					}
				}/>
			<button on:click=add_spread class="bg-transparent hover:bg-blue-500 text-blue-700 font-semibold hover:text-white py-2 px-4 border border-blue-500 hover:border-transparent rounded">"Add Spread"</button>
		</div>
	}
}

#[component]
pub fn NewSpread(cx: Scope, spread: RwSignal<SpreadBuilder>) -> impl IntoView {
	let home_team = create_rw_signal::<Option<Team>>(cx, None);
	let away_team = create_rw_signal::<Option<Team>>(cx, None);

	let home_view = {move ||
		match home_team.get() {
			Some(team) => {
				spread.update(|s| { s.home_id = Some(team.id) });
				view!{cx,
					<button class="hover:bg-red-700 " on:click=move |_| home_team.set(None)>
						<div class="col-span-1">
							<h3 class="text-center">Home</h3>
							<img src=team.logo class="object-contain w-full"/>
							<h3 class="text-center">{team.name}</h3>
						</div>
					</button>
				}.into_view(cx)
			},
			None => view!{cx,
				<div class="col-span-1">
					<h3 class="text-center">Home</h3>
					<TeamSelect team_selector=home_team.write_only()/>
				</div>
			}.into_view(cx)
		}
	};

	let away_view = {move ||
		match away_team.get() {
			Some(team) => {
				spread.update(|s| { s.home_id = Some(team.id) });
				view!{cx,
					<button class="hover:bg-red-700" on:click=move |_| away_team.set(None)>
						<div class="col-span-1">
							<h3 class="text-center">Away</h3>
							<img src=team.logo class="object-contain w-full"/>
							<h3 class="text-center">{team.name}</h3>
						</div>
					</button>
				}.into_view(cx)
			},
			None => view!{cx,
				<div class="col-span-1">
					<h3 class="text-center">Home</h3>
					<TeamSelect team_selector=away_team.write_only()/>
				</div>
			}.into_view(cx)
		}
	};

	view! {cx,
		<div class="max-w-sm rounded overflow-hidden shadow-lg justify-center">
			<div class="grid grid-cols-2 grid-flow-col gap-4">
				{home_view}
				{away_view}
			</div>
			<div class="w-72 justify-center">
				<div class="relative h-10 w-full min-w-[200px]">
					<input
					class="peer h-full w-full rounded-[7px] border border-blue-gray-200 border-t-transparent bg-transparent px-3 py-2.5 font-sans text-sm font-normal text-blue-gray-700 outline outline-0 transition-all placeholder-shown:border placeholder-shown:border-blue-gray-200 placeholder-shown:border-t-blue-gray-200 focus:border-2 focus:border-t-transparent focus:outline-0 disabled:border-0 disabled:bg-blue-gray-50"
					placeholder=""
					/>
					<label class="before:content[' '] after:content[' '] pointer-events-none absolute left-0 -top-1.5 flex h-full w-full select-none text-[11px] font-normal leading-tight text-blue-gray-400 transition-all before:pointer-events-none before:mt-[6.5px] before:mr-1 before:box-border before:block before:h-1.5 before:w-2.5 before:rounded-tl-md before:border-t before:border-l before:border-blue-gray-200 before:transition-all after:pointer-events-none after:mt-[6.5px] after:ml-1 after:box-border after:block after:h-1.5 after:w-2.5 after:flex-grow after:rounded-tr-md after:border-t after:border-r after:border-blue-gray-200 after:transition-all peer-placeholder-shown:text-sm peer-placeholder-shown:leading-[3.75] peer-placeholder-shown:text-blue-gray-500 peer-placeholder-shown:before:border-transparent peer-placeholder-shown:after:border-transparent peer-focus:text-[11px] peer-focus:leading-tight peer-focus:text-gree-500 peer-focus:before:border-t-2 peer-focus:before:border-l-2 peer-focus:after:border-t-2 peer-focus:after:border-r-2 peer-disabled:text-transparent peer-disabled:before:border-transparent peer-disabled:after:border-transparent peer-disabled:peer-placeholder-shown:text-blue-gray-500">
					"Home Spread"
					</label>
				</div>
			</div>
			<div class="w-72 justify-center">
				<div class="relative h-10 w-full min-w-[200px]">
					<input
					class="peer h-full w-full rounded-[7px] border border-blue-gray-200 border-t-transparent bg-transparent px-3 py-2.5 font-sans text-sm font-normal text-blue-gray-700 outline outline-0 transition-all placeholder-shown:border placeholder-shown:border-blue-gray-200 placeholder-shown:border-t-blue-gray-200 focus:border-2 focus:border-t-transparent focus:outline-0 disabled:border-0 disabled:bg-blue-gray-50"
					placeholder=""
					/>
					<label class="before:content[' '] after:content[' '] pointer-events-none absolute left-0 -top-1.5 flex h-full w-full select-none text-[11px] font-normal leading-tight text-blue-gray-400 transition-all before:pointer-events-none before:mt-[6.5px] before:mr-1 before:box-border before:block before:h-1.5 before:w-2.5 before:rounded-tl-md before:border-t before:border-l before:border-blue-gray-200 before:transition-all after:pointer-events-none after:mt-[6.5px] after:ml-1 after:box-border after:block after:h-1.5 after:w-2.5 after:flex-grow after:rounded-tr-md after:border-t after:border-r after:border-blue-gray-200 after:transition-all peer-placeholder-shown:text-sm peer-placeholder-shown:leading-[3.75] peer-placeholder-shown:text-blue-gray-500 peer-placeholder-shown:before:border-transparent peer-placeholder-shown:after:border-transparent peer-focus:text-[11px] peer-focus:leading-tight peer-focus:text-gree-500 peer-focus:before:border-t-2 peer-focus:before:border-l-2 peer-focus:after:border-t-2 peer-focus:after:border-r-2 peer-disabled:text-transparent peer-disabled:before:border-transparent peer-disabled:after:border-transparent peer-disabled:peer-placeholder-shown:text-blue-gray-500">
					"Notes (Optional)"
					</label>
				</div>
			</div>
		</div>
	}
}
