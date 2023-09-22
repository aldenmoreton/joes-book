use leptos::*;
use leptos_router::use_params_map;

use crate::{server::{get_picks, get_spread_teams, save_picks}, objects::{Event, Pick, EventContent}};


#[component]
pub fn GradeChapter(cx: Scope) -> impl IntoView {
	let params = use_params_map(cx);
	let book_id: i64 = params.with_untracked(|params| params.get("book_id").cloned()).unwrap().parse::<i64>().unwrap();
	let chapter_id: i64 = params.with_untracked(|params| params.get("chapter_id").cloned()).unwrap().parse::<i64>().unwrap();

	let new_picks: RwSignal<bool> = create_rw_signal(cx, false);
	provide_context(cx, new_picks.write_only());

	let pick_fetcher = create_resource(cx, move || new_picks.get(),
		move |_| get_picks(cx, chapter_id)
	);

	view!{cx,
		<p>{book_id}</p>
		<p>{chapter_id}</p>
		<Transition fallback=|| "Loading...">
			<div class="flex flex-col items-center justify-center">
				{move ||
					pick_fetcher.read(cx).map(|events| match events {
						Err(e) => {
							view! { cx, <pre class="error">"Server Error with pick fetcher: " {e.to_string()}</pre>}.into_view(cx)
						},
						Ok(events) => {
							view!{cx,
								<ChapterEvents initial_values=events/>
							}.into_view(cx)
						}
					})
				}
			</div>
		</Transition>
	}
}

#[component]
pub fn ChapterEvents(cx: Scope, initial_values: Vec<(String, Vec<(Event, Pick)>)>) -> impl IntoView {
	let params = use_params_map(cx);
	let book_id: i64 = params.with_untracked(|params| params.get("book_id").cloned()).unwrap().parse::<i64>().unwrap();

	let discrepancies: RwSignal<Option<i32>> = create_rw_signal(cx, None);
	provide_context(cx, discrepancies.write_only());

	let global_picks: RwSignal<Vec<ReadSignal<Pick>>> = create_rw_signal(cx, Vec::new());
	provide_context(cx, global_picks.write_only());

	let new_picks: WriteSignal<bool> = use_context(cx)
		.expect("You should be able to refresh picks");

	let pick_views = initial_values.into_iter()
		.map(move |value| {
			match value.0.as_str() {
				"SpreadGroup" => view!{cx,
					<SpreadGroupPick initial_values=value.1/>
				}.into_view(cx),
				"UserInput" => view! {cx,
					<UserInputs initial_values=value.1/>
				},
				_ => "No pick GUI for this event yet".into_view(cx)
			}
		})
		.collect_view(cx);

	let pick_submission = create_action(cx,
		move |_| async move {
			let picks = global_picks
				.get()
				.into_iter()
				.map(|pick| pick.get())
				.collect();
			save_picks(cx, picks).await
		}
	);

	view! {cx,
		{pick_views}
		{move || new_picks.update(|p| {if pick_submission.value().get().is_some() { *p = !*p }})}
		// fixed inset-x-0 bottom-0
		<div class="h-16 items-center justify-center grid">
			<div class="h-full w-32 justify-center self-center text-center content-center">
				{move || match discrepancies.get() {
					Some(discrepancies) if discrepancies == 0 => {
						match (pick_submission.pending().get(), pick_submission.value().get()) {
							(false, None) => {
								view!{cx,
									<button on:click=move |_| pick_submission.dispatch(()) class="h-full w-full bg-black text-white rounded-xl">"Submit"</button>
								}.into_view(cx)
							},
							(false, Some(Ok(()))) => {
								view!{cx,
									<a href={format!("/books/{}", book_id)}>
										<button class="border border-black rounded-md bg-green-500">
											<h1>"Picks are saved"</h1>
											<p>"Go back to book"</p>
										</button>
									</a>
									<button class="border border-black rounded-md bg-green-500" on:click=move |_| pick_submission.value().set(None)>
										<h1>"Edit Picks Again"</h1>
									</button>
								}.into_view(cx)
							},
							(false, Some(Err(e))) => {
								view!{cx,
									<p>{format!("{e}")}</p>
								}.into_view(cx)
							},
							(true, _) => {
								view!{cx,
									<p>"Loading..."</p>
								}.into_view(cx)
							}
						}
					},
					Some(d) => {
						view!{cx,
							<h1>"Picks aren't finished yet: "{d}</h1>
						}.into_view(cx)
					},
					None => {
						view!{cx,
							<a href={format!("/books/{}", book_id)}>
								<button class="border border-black rounded-md bg-green-500">
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
pub fn SpreadGroupPick(cx: Scope, initial_values: Vec<(Event, Pick)>) -> impl IntoView {
	let global_picks = use_context::<WriteSignal<Vec<ReadSignal<Pick>>>>(cx)
		.expect("You should have access to the picks");
	let global_discrepancies = use_context::<WriteSignal<Option<i32>>>(cx)
		.expect("You should have access to discrepancies counter");

	let num_of_picks = initial_values.len();
	let wager_trackers = create_rw_signal(cx, vec![0; num_of_picks]);

	let reactive_events: Vec<(Event, RwSignal<Pick>)> = initial_values
		.into_iter()
		.map(|(event, pick)| {
			if let Some(wager) = pick.wager {
				wager_trackers.update(|t| t[wager as usize - 1] += 1);
			}
			if pick.wager.is_none() { global_discrepancies.update(|d| *d = Some(d.unwrap_or(0) + 1)) }
			if pick.choice.is_none() { global_discrepancies.update(|d| *d = Some(d.unwrap_or(0) + 1)) }

			let new_pick = create_rw_signal(cx, pick);
			global_picks.update(|picks| picks.push(new_pick.read_only()));
			(event, new_pick)
		})
		.collect::<Vec<_>>();

	let spread_setter = move |pick: RwSignal<Pick>, choice: &str| {
		pick.update(|pick| {
			if let Some(_) = &pick.choice {
				global_discrepancies.update(|d| *d = Some(d.unwrap_or(0)))
			} else {
				global_discrepancies.update(|d| *d = Some(d.unwrap_or(1) - 1))
			}
			pick.choice = Some(choice.into());
		})
	};

	let wager_setter = move |pick: RwSignal<Pick>, wager| {
		pick.update(|pick| {
			let new_wager = wager_trackers.get()[wager-1];
			let new_wager_right = new_wager == 0;
			if let Some(old_wager) = pick.wager {
				let old_wager_wrong = wager_trackers.get()[old_wager as usize - 1] > 1;
				if old_wager == new_wager {
					()
				} else if old_wager_wrong && new_wager_right {
					global_discrepancies.update(|d| *d = Some(d.unwrap_or(1) - 1))
				} else if !old_wager_wrong && !new_wager_right {
					global_discrepancies.update(|d| *d = Some(d.unwrap_or(0) + 1))
				}
			} else {
				if new_wager_right {
					global_discrepancies.update(|d| *d = Some(d.unwrap_or(1) - 1))
				}
			}

			wager_trackers.update(|t| {
				if let Some(old_wager) = pick.wager {
					t[old_wager as usize - 1] -= 1;
				}
				t[wager - 1] += 1
			});
			pick.wager = Some(wager as i64)
		})
	};

	view! {cx,
		{
			reactive_events
				.into_iter()
				.enumerate()
				.map(|(i, (event, pick))| match event.contents {
					EventContent::SpreadGroup(spread) => {
						let old_pick = pick.get();
						view!{cx,
							<div class="p-3">
								<div class="max-w-sm rounded-lg overflow-hidden shadow-lg justify-center content-center bg-white">
									<h1>"Game " {i+1}</h1>
									<Await future=move |cx| get_spread_teams(cx, spread.home_id, spread.away_id) bind:spread_teams>
										{match spread_teams {
											Err(e) => format!("Could not find spread teams: {e}").into_view(cx),
											Ok((home_team, away_team)) => {
												view!{cx,
													<div class="grid grid-cols-2 grid-flow-col gap-4 p-5">
														<div class="col-span-1">
															<h1>"Home"</h1>
															<input on:click=move |_| spread_setter(pick, "Home") type="radio" id={format!("{}", &home_team.id)} name={format!("{}-{}", &home_team.id, &away_team.id)} value="home" class="hidden peer" checked={if &old_pick.choice == &Some("Home".into()) {true} else {false}}/>
															<label for={format!("{}", &home_team.id)} class="inline-grid w-full p-5 pb-0 pt-0 border border-black rounded-lg cursor-pointer hover:border-green-700 peer-checked:bg-green-500 peer-checked:border-green-600 hover:bg-green-100">
																<img src=&home_team.logo class="w-full"/>
																<h2>{&home_team.name}</h2>
																<h2 class="text-center pb-1">{format!("{:+}", spread.home_spread)}</h2>
															</label>
														</div>
														<div class="col-span-1">
															<h1>"Away"</h1>
															<input on:click=move |_| spread_setter(pick, "Away") type="radio" id={format!("{}", &away_team.id)} name={format!("{}-{}", &home_team.id, &away_team.id)} value="away" class="hidden peer" checked={if &old_pick.choice == &Some("Away".into()) {true} else {false}}/>
															<label for={format!("{}", &away_team.id)} class="inline-grid w-full p-5 pb-0 pt-0 border border-black rounded-lg cursor-pointer hover:border-green-700 peer-checked:bg-green-500 peer-checked:border-green-600 hover:bg-green-100">
																<img src=&away_team.logo class="w-full"/>
																<h2>{&away_team.name}</h2>
																<h2 class="text-center pb-1">{format!("{:+}", -1. * spread.home_spread)}</h2>
															</label>
														</div>
													</div>
												}.into_view(cx)
											}
										}}
									</Await>
									<ul>
									{
										(1..=num_of_picks)
											.map(|i| {
												view!{cx,
													<li class="inline-flex p-1 items-center">
														<input on:click=move |_| wager_setter(pick, i) type="radio" id={format!("{}-{}-{}-wager", i, spread.home_id, spread.away_id)} name={format!("{}-{}-wager", spread.home_id, spread.away_id)} value="home" class="hidden peer" checked={if old_pick.wager == Some(i as i64) {true} else {false}}/>
														<label for={format!("{}-{}-{}-wager", i, spread.home_id, spread.away_id)} class="inline-grid w-5 h-5 p-5 border border-black rounded-lg cursor-pointer hover:border-green-700 peer-checked:bg-green-500 peer-checked:border-green-600 hover:bg-green-100">
															<p class="text-center">{i}</p>
														</label>
													</li>
												}
											})
											.collect_view(cx)
									}
									</ul>
									{
										if let Some(notes) = spread.notes {
											view!{cx,
												<details class="pt-1 pb-3">
													<summary>"Notes"</summary>
													<p>
														<span style="white-space: pre-line">
															{notes}
														</span>
													</p>
												</details>
											}.into_view(cx)
										} else {
											().into_view(cx)
										}
									}
								</div>
							</div>

						}.into_view(cx)
					},
					_ => {
						view!{cx,
							<h1>"This should have been a spread pick. Something is wrong!!!!!!!"</h1>
						}.into_view(cx)
					}
				})
				.collect_view(cx)
		}
	}
}

#[component]
pub fn UserInputs(cx: Scope, initial_values: Vec<(Event, Pick)>) -> impl IntoView {
	let global_picks = use_context::<WriteSignal<Vec<ReadSignal<Pick>>>>(cx)
		.expect("You should have access to the picks");
	let global_discrepancies = use_context::<WriteSignal<Option<i32>>>(cx)
		.expect("You should have access to discrepancies counter");

	let reactive_events: Vec<(Event, RwSignal<Pick>)> = initial_values
		.into_iter()
		.map(|(event, pick)| {
			if pick.choice.is_none() { global_discrepancies.update(|d| *d = Some(d.unwrap_or(0) + 1)) }

			let new_pick = create_rw_signal(cx, pick);
			new_pick.update(|p| {
				if p.wager.is_none() {
					let wager = match event.contents.clone() {
						EventContent::UserInput(input) => input.points,
						_ => panic!("You shouldn't be in this component with an event other than UserInput")
					};
					p.wager = Some(wager)
				}
			});
			global_picks.update(|picks| picks.push(new_pick.read_only()));
			(event, new_pick)
		})
		.collect::<Vec<_>>();

	let change_answer = move |pick: RwSignal<Pick>, new_answer: String| {
		pick.update(|pick| {
			let new_length = new_answer.len();

			if new_length == 0 {
				if pick.choice.is_some() {
					global_discrepancies.update(|c| *c = Some(c.unwrap_or(0) + 1))
				}
				pick.choice = None
			} else {
				if pick.choice.is_none() {
					global_discrepancies.update(|c| *c = Some(c.unwrap_or(1) - 1))
				} else {
					global_discrepancies.update(|c| if c.is_none() { *c = Some(0) })
				}
				pick.choice = Some(new_answer)
			}
		})
	};

	view! {cx,
		{
			reactive_events
				.into_iter()
				.enumerate()
				.map(|(i, (event, pick))| match event.contents {
					EventContent::UserInput(user_input_event) => {
						view!{cx,
							<div class="max-w-sm rounded-lg overflow-hidden shadow-lg justify-center content-center bg-white">
								<h1>"Extra Point Question " {i+1}</h1>
								<p>{user_input_event.question}</p>
								<div class="justify-center p-2">
									<div class="relative h-30">
										<textarea
										class="peer resize-none h-full w-full rounded-[7px] border border-green-200 border-t-transparent bg-transparent px-3 py-2.5 font-sans text-sm font-normal text-green-700 outline outline-0 transition-all placeholder-shown:border placeholder-shown:border-green-200 placeholder-shown:border-t-green-200 focus:border-2 focus:border-t-transparent focus:outline-0 disabled:border-0 disabled:bg-green-50"
										placeholder=""
										on:input=move |ev| {change_answer(pick, event_target_value(&ev))}/>
										<label class="before:content[' '] after:content[' '] pointer-events-none absolute left-0 -top-1.5 flex h-full w-full select-none text-[11px] font-normal leading-tight text-green-400 transition-all before:pointer-events-none before:mt-[6.5px] before:mr-1 before:box-border before:block before:h-1.5 before:w-2.5 before:rounded-tl-md before:border-t before:border-l before:border-green-200 before:transition-all after:pointer-events-none after:mt-[6.5px] after:ml-1 after:box-border after:block after:h-1.5 after:w-2.5 after:flex-grow after:rounded-tr-md after:border-t after:border-r after:border-green-200 after:transition-all peer-placeholder-shown:text-sm peer-placeholder-shown:leading-[3.75] peer-placeholder-shown:text-green-500 peer-placeholder-shown:before:border-transparent peer-placeholder-shown:after:border-transparent peer-focus:text-[11px] peer-focus:leading-tight peer-focus:text-gree-500 peer-focus:before:border-t-2 peer-focus:before:border-l-2 peer-focus:after:border-t-2 peer-focus:after:border-r-2 peer-disabled:text-transparent peer-disabled:before:border-transparent peer-disabled:after:border-transparent peer-disabled:peer-placeholder-shown:text-green-500">
										{pick.get().choice.map(|c| format!("Current Answer: {c}")).unwrap_or("Please Answer".into())}
										</label>
									</div>
								</div>
							</div>
						}
					},
					_ => panic!("You shouldn't be calling this component with something other than a UserInput")
				})
				.collect_view(cx)
		}
	}
}
