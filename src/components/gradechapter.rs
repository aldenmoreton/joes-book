use leptos::*;
use leptos_router::{use_params_map, Redirect};

use crate::{server::{get_spread_teams, get_book, get_events, save_answers}, objects::{Event, EventContent, BookSubscription, BookRole, Spread}};

#[component]
pub fn GradeChapter(cx: Scope) -> impl IntoView {
	let params = use_params_map(cx);
	let book_id:i64 = params.with_untracked(|params| params.get("book_id").cloned()).unwrap().parse::<i64>().unwrap();
	let user_subscription = create_resource(
		cx,
		|| (),
		move |_| async move {
			get_book(cx, book_id).await
		}
	);

	view!{cx,
		<Suspense fallback=|| "Loading user data">
			{move || user_subscription.read(cx).map(|subscription| {
				match subscription {
					Err(e) => format!("{e}").into_view(cx),
					Ok(BookSubscription{role: BookRole::Owner, ..}) => VerifiedGradeChapter(cx).into_view(cx),
					Ok(_) => view! { cx, <Redirect path=format!("/books/{book_id}")/> }.into_view(cx)
				}
			})
			}
		</Suspense>
	}
}

#[component]
pub fn VerifiedGradeChapter(cx: Scope) -> impl IntoView {
	let params = use_params_map(cx);
	let chapter_id: i64 = params.with_untracked(|params| params.get("chapter_id").cloned()).unwrap().parse::<i64>().unwrap();

	let events_fetcher = create_resource(cx, move || (),
		move |_| get_events(cx, chapter_id)
	);

	view!{cx,
		<Transition fallback=|| "Loading...">
			<div class="flex flex-col items-center justify-center">
				{move ||
					events_fetcher.read(cx).map(|events| match events {
						Err(e) => {
							view! { cx, <pre class="error">"Server Error with event fetcher: " {e.to_string()}</pre>}.into_view(cx)
						},
						Ok(events) => {
							view!{cx,
								<ChapterEvents events/>
							}.into_view(cx)
						}
					})
				}
			</div>
		</Transition>
	}
}

#[component]
pub fn ChapterEvents(cx: Scope, events: Vec<Event>) -> impl IntoView {
	let params = use_params_map(cx);
	let book_id: i64 = params.with_untracked(|params| params.get("book_id").cloned()).unwrap().parse::<i64>().unwrap();

	let discrepancies: RwSignal<bool> = create_rw_signal(cx, false);
	provide_context(cx, discrepancies.write_only());

	let global_answers: RwSignal<Vec<ReadSignal<Option<(i64, Vec<String>)>>>> = create_rw_signal(cx, Vec::new());
	provide_context(cx, global_answers);

	let pick_views = events.into_iter()
		.map(move |event| {
			match event.contents {
				EventContent::SpreadGroup(spread) => view!{cx,
					<SpreadGroupGrade id=event.id spread/>
				}.into_view(cx),
				EventContent::UserInput(input) => view!{cx,
					<p>"Not implemented for User Input yet"</p>
					{format!("{input:?}")}
				}.into_view(cx)
			}
		})
		.collect_view(cx);

	let pick_submission = create_action(cx,
		move |_| async move {
			let picks: Vec<(i64, Vec<String>)> = global_answers
				.get()
				.into_iter()
				.map(|answer| answer.get())
				.flatten()
				.map(|answer| (answer.0, answer.1))
				.collect();
			save_answers(cx, picks).await
		}
	);

	view! {cx,
		{pick_views}
		<br/>
		{global_answers}
		<div class="grid items-center justify-center h-16">
			<div class="content-center self-center justify-center w-32 h-full text-center">
				{move || match discrepancies.get() {
					true => {
						view!{cx,
							<button on:click=move |_| pick_submission.dispatch(()) class="w-full h-full text-white bg-black rounded-xl">"Submit"</button>
						}.into_view(cx)
					},
					false => {
						view!{cx,
							<a href={format!("/books/{}", book_id)}>
								<button class="bg-green-500 border border-black rounded-md">
									<h1>"Current Picks are Saved"</h1>
									<p>"Go back to book"</p>
								</button>
							</a>
						}.into_view(cx)
					}
				}}
			</div>
		</div>
	}
}

#[component]
pub fn SpreadGroupGrade(cx: Scope, id: i64, spread: Spread) -> impl IntoView {
	let global_answers = use_context::<RwSignal<Vec<ReadSignal<Option<(i64, Vec<String>)>>>>>(cx)
		.expect("You should have access to the picks");

	let spread_answer: RwSignal<Option<(i64, Vec<String>)>> = create_rw_signal(cx, None);
	global_answers.update(|answers| answers.push(spread_answer.read_only()));

	let answer_setter = move |choice: &str| {
		match choice {
			"Home" | "Away" | "Push" => spread_answer.set(Some((id, vec![choice.into()]))),
			_ => spread_answer.set(None)
		}
	};

	let teams_getter = create_resource(cx,
		|| (),
		move |_| get_spread_teams(cx, spread.home_id, spread.away_id)
	);

	view!{cx,
		<div class="p-3">
			<div class="content-center justify-center max-w-sm overflow-hidden bg-white rounded-lg shadow-lg">
				<Suspense fallback=move || view!{cx, <p>"Loading..."</p>}>
					{
						move || teams_getter.read(cx).map(|result| {
							match result {
								Err(e) => format!("Could not get teams:\n{e}").into_view(cx),
								Ok((home_team, away_team)) =>
									view!{cx,
										<div class="grid grid-flow-col grid-cols-2 gap-4 p-5">
											<div class="col-span-1">
												<h1>"Home"</h1>
												<input on:click=move |_| answer_setter("Home") type="radio" id={format!("{}", &home_team.id)} name={format!("{}-{}", &home_team.id, &away_team.id)} value="home" class="hidden peer"/>
												<label for={format!("{}", &home_team.id)} class="inline-grid w-full p-5 pt-0 pb-0 border border-black rounded-lg cursor-pointer hover:border-green-700 peer-checked:bg-green-500 peer-checked:border-green-600 hover:bg-green-100">
													<img src=&home_team.logo class="w-full"/>
													<h2>{&home_team.name}</h2>
													<h2 class="pb-1 text-center">{format!("{:+}", spread.home_spread)}</h2>
												</label>
											</div>
											<div class="col-span-1">
												<h1>"Away"</h1>
												<input on:click=move |_| answer_setter("Away") type="radio" id={format!("{}", &away_team.id)} name={format!("{}-{}", &home_team.id, &away_team.id)} value="away" class="hidden peer"/>
												<label for={format!("{}", &away_team.id)} class="inline-grid w-full p-5 pt-0 pb-0 border border-black rounded-lg cursor-pointer hover:border-green-700 peer-checked:bg-green-500 peer-checked:border-green-600 hover:bg-green-100">
													<img src=&away_team.logo class="w-full"/>
													<h2>{&away_team.name}</h2>
													<h2 class="pb-1 text-center">{format!("{:+}", -1. * spread.home_spread)}</h2>
												</label>
											</div>
										</div>
									}.into_view(cx)
							}
						})
					}
				</Suspense>
				// <Await future=move |cx| get_spread_teams(cx, spread.home_id, spread.away_id) bind:spread_teams>
				// 	{match spread_teams {
				// 		Err(e) => format!("Could not find spread teams: {e}").into_view(cx),
				// 		Ok((home_team, away_team)) => {
				// 			view!{cx,
				// 				<div class="grid grid-flow-col grid-cols-2 gap-4 p-5">
				// 					<div class="col-span-1">
				// 						<h1>"Home"</h1>
				// 						<input on:click=move |_| spread_setter(pick, "Home") type="radio" id={format!("{}", &home_team.id)} name={format!("{}-{}", &home_team.id, &away_team.id)} value="home" class="hidden peer" checked={if &old_pick.choice == &Some("Home".into()) {true} else {false}}/>
				// 						<label for={format!("{}", &home_team.id)} class="inline-grid w-full p-5 pt-0 pb-0 border border-black rounded-lg cursor-pointer hover:border-green-700 peer-checked:bg-green-500 peer-checked:border-green-600 hover:bg-green-100">
				// 							<img src=&home_team.logo class="w-full"/>
				// 							<h2>{&home_team.name}</h2>
				// 							<h2 class="pb-1 text-center">{format!("{:+}", spread.home_spread)}</h2>
				// 						</label>
				// 					</div>
				// 					<div class="col-span-1">
				// 						<h1>"Away"</h1>
				// 						<input on:click=move |_| spread_setter(pick, "Away") type="radio" id={format!("{}", &away_team.id)} name={format!("{}-{}", &home_team.id, &away_team.id)} value="away" class="hidden peer" checked={if &old_pick.choice == &Some("Away".into()) {true} else {false}}/>
				// 						<label for={format!("{}", &away_team.id)} class="inline-grid w-full p-5 pt-0 pb-0 border border-black rounded-lg cursor-pointer hover:border-green-700 peer-checked:bg-green-500 peer-checked:border-green-600 hover:bg-green-100">
				// 							<img src=&away_team.logo class="w-full"/>
				// 							<h2>{&away_team.name}</h2>
				// 							<h2 class="pb-1 text-center">{format!("{:+}", -1. * spread.home_spread)}</h2>
				// 						</label>
				// 					</div>
				// 				</div>
				// 			}.into_view(cx)
				// 		}
				// 	}}
				// </Await>
			</div>
		</div>

	}
}

// #[component]
// pub fn UserInputs(cx: Scope, initial_values: Vec<(Event, Pick)>) -> impl IntoView {
// 	let global_picks = use_context::<WriteSignal<Vec<ReadSignal<Pick>>>>(cx)
// 		.expect("You should have access to the picks");
// 	let global_discrepancies = use_context::<WriteSignal<Option<i32>>>(cx)
// 		.expect("You should have access to discrepancies counter");

// 	let reactive_events: Vec<(Event, RwSignal<Pick>)> = initial_values
// 		.into_iter()
// 		.map(|(event, pick)| {
// 			if pick.choice.is_none() { global_discrepancies.update(|d| *d = Some(d.unwrap_or(0) + 1)) }

// 			let new_pick = create_rw_signal(cx, pick);
// 			new_pick.update(|p| {
// 				if p.wager.is_none() {
// 					let wager = match event.contents.clone() {
// 						EventContent::UserInput(input) => input.points,
// 						_ => panic!("You shouldn't be in this component with an event other than UserInput")
// 					};
// 					p.wager = Some(wager)
// 				}
// 			});
// 			global_picks.update(|picks| picks.push(new_pick.read_only()));
// 			(event, new_pick)
// 		})
// 		.collect::<Vec<_>>();

// 	let change_answer = move |pick: RwSignal<Pick>, new_answer: String| {
// 		pick.update(|pick| {
// 			let new_length = new_answer.len();

// 			if new_length == 0 {
// 				if pick.choice.is_some() {
// 					global_discrepancies.update(|c| *c = Some(c.unwrap_or(0) + 1))
// 				}
// 				pick.choice = None
// 			} else {
// 				if pick.choice.is_none() {
// 					global_discrepancies.update(|c| *c = Some(c.unwrap_or(1) - 1))
// 				} else {
// 					global_discrepancies.update(|c| if c.is_none() { *c = Some(0) })
// 				}
// 				pick.choice = Some(new_answer)
// 			}
// 		})
// 	};

// 	view! {cx,
// 		{
// 			reactive_events
// 				.into_iter()
// 				.enumerate()
// 				.map(|(i, (event, pick))| match event.contents {
// 					EventContent::UserInput(user_input_event) => {
// 						view!{cx,
// 							<div class="content-center justify-center max-w-sm overflow-hidden bg-white rounded-lg shadow-lg">
// 								<h1>"Extra Point Question " {i+1}</h1>
// 								<p>{user_input_event.question}</p>
// 								<div class="justify-center p-2">
// 									<div class="relative h-30">
// 										<textarea
// 										class="peer resize-none h-full w-full rounded-[7px] border border-green-200 border-t-transparent bg-transparent px-3 py-2.5 font-sans text-sm font-normal text-green-700 outline outline-0 transition-all placeholder-shown:border placeholder-shown:border-green-200 placeholder-shown:border-t-green-200 focus:border-2 focus:border-t-transparent focus:outline-0 disabled:border-0 disabled:bg-green-50"
// 										placeholder=""
// 										on:input=move |ev| {change_answer(pick, event_target_value(&ev))}/>
// 										<label class="before:content[' '] after:content[' '] pointer-events-none absolute left-0 -top-1.5 flex h-full w-full select-none text-[11px] font-normal leading-tight text-green-400 transition-all before:pointer-events-none before:mt-[6.5px] before:mr-1 before:box-border before:block before:h-1.5 before:w-2.5 before:rounded-tl-md before:border-t before:border-l before:border-green-200 before:transition-all after:pointer-events-none after:mt-[6.5px] after:ml-1 after:box-border after:block after:h-1.5 after:w-2.5 after:flex-grow after:rounded-tr-md after:border-t after:border-r after:border-green-200 after:transition-all peer-placeholder-shown:text-sm peer-placeholder-shown:leading-[3.75] peer-placeholder-shown:text-green-500 peer-placeholder-shown:before:border-transparent peer-placeholder-shown:after:border-transparent peer-focus:text-[11px] peer-focus:leading-tight peer-focus:text-gree-500 peer-focus:before:border-t-2 peer-focus:before:border-l-2 peer-focus:after:border-t-2 peer-focus:after:border-r-2 peer-disabled:text-transparent peer-disabled:before:border-transparent peer-disabled:after:border-transparent peer-disabled:peer-placeholder-shown:text-green-500">
// 										{pick.get().choice.map(|c| format!("Current Answer: {c}")).unwrap_or("Please Answer".into())}
// 										</label>
// 									</div>
// 								</div>
// 							</div>
// 						}
// 					},
// 					_ => panic!("You shouldn't be calling this component with something other than a UserInput")
// 				})
// 				.collect_view(cx)
// 		}
// 	}
// }
