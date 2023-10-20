use leptos::*;
use leptos_router::{use_params_map, Redirect};

use crate::{server::{get_picks, get_spread_teams, save_picks, is_open, set_open, get_book}, objects::{Event, Pick, EventContent, BookRole, BookSubscription}};


#[component]
pub fn Chapter() -> impl IntoView {
	let params = use_params_map();
	let book_id: i64 = params.with_untracked(|params| params.get("book_id").cloned()).unwrap().parse::<i64>().unwrap();
	let chapter_id: i64 = params.with_untracked(|params| params.get("chapter_id").cloned()).unwrap().parse::<i64>().unwrap();

	let new_picks: RwSignal<bool> = create_rw_signal(false);
	provide_context(new_picks.write_only());

	let status_fetcher = create_resource(|| (),
		move |_| is_open(chapter_id)
	);

	let pick_fetcher = create_resource(move || new_picks.get(),
		move |_| get_picks(chapter_id)
	);

	let chapter_closer = create_action(move |()| set_open(chapter_id, false));

	view!{
		<Await future=move || get_book(book_id) let:subscription>
			{match subscription {
				Ok(BookSubscription{role: BookRole::Owner, ..}) => view!{<button class="bg-green-200 border border-black rounded-md" on:click=move |_| chapter_closer.dispatch(())>"Close Chapter"</button>}.into_view(),
				_ => ().into_view()
			}}
		</Await>
		<Transition fallback=|| "Loading...">
			{move ||
				status_fetcher.get().map(|status| match status {
					Err(e) => {
						view! {<pre class="error">"Server Error with status fetcher: " {e.to_string()}</pre>}.into_view()
					},
					Ok(true) => view!{
						{move ||
							pick_fetcher.get().map(|events| match events {
								Err(e) => {
									view! {<pre class="error">"Server Error with pick fetcher: " {e.to_string()}</pre>}.into_view()
								},
								Ok(events) => {
									view!{
										<ChapterEvents initial_values=events/>
									}.into_view()
								}
							})
						}
					}.into_view(),
					Ok(false) => view!{<Redirect path=format!("/books/{}/chapters/{}/table", book_id, chapter_id)/>}.into_view()
				})
			}
		</Transition>
	}
}

#[component]
pub fn ChapterEvents(initial_values: Vec<(String, Vec<(Event, Pick)>)>) -> impl IntoView {
	let params = use_params_map();
	let book_id: i64 = params.with_untracked(|params| params.get("book_id").cloned()).unwrap().parse::<i64>().unwrap();

	let discrepancies: RwSignal<Option<i32>> = create_rw_signal(None);
	provide_context(discrepancies.write_only());

	let global_picks: RwSignal<Vec<ReadSignal<Pick>>> = create_rw_signal(Vec::new());
	provide_context(global_picks.write_only());

	let new_picks: WriteSignal<bool> = use_context()
		.expect("You should be able to refresh picks");

	let pick_views = initial_values.into_iter()
		.map(move |value| {
			match value.0.as_str() {
				"SpreadGroup" => view!{
					<SpreadGroupPick initial_values=value.1/>
				}.into_view(),
				"UserInput" => view! {
					<UserInputs initial_values=value.1/>
				},
				_ => "No pick GUI for this event yet".into_view()
			}
		})
		.collect_view();

	let pick_submission = create_action(
		move |_| async move {
			let picks = global_picks
				.get()
				.into_iter()
				.map(|pick| pick.get())
				.collect();
			save_picks(picks).await
		}
	);

	view! {
		<div class="flex flex-col items-center justify-center">
			{pick_views}
			{move || new_picks.update(|p| {if pick_submission.value().get().is_some() { *p = !*p }})}
			<div class="grid items-center justify-center h-16">
				<div class="content-center self-center justify-center w-32 h-full text-center">
					{move || match discrepancies.get() {
						Some(discrepancies) if discrepancies == 0 => {
							match (pick_submission.pending().get(), pick_submission.value().get()) {
								(false, None) => {
									view!{
										<button on:click=move |_| pick_submission.dispatch(()) class="w-full h-full text-white bg-black rounded-xl">"Submit"</button>
									}.into_view()
								},
								(false, Some(Ok(()))) => {
									view!{
										<a href={format!("/books/{}", book_id)}>
											<button class="bg-green-500 border border-black rounded-md">
												<h1>"Picks are saved"</h1>
												<p>"Go back to book"</p>
											</button>
										</a>
										<button class="bg-green-500 border border-black rounded-md" on:click=move |_| pick_submission.value().set(None)>
											<h1>"Edit Picks Again"</h1>
										</button>
									}.into_view()
								},
								(false, Some(Err(e))) => {
									view!{
										<p>{format!("{e}")}</p>
									}.into_view()
								},
								(true, _) => {
									view!{
										<p>"Loading..."</p>
									}.into_view()
								}
							}
						},
						Some(d) => {
							view!{
								<h1>"Picks aren't finished yet: "{d}</h1>
							}.into_view()
						},
						None => {
							view!{
								<a href={format!("/books/{}", book_id)}>
									<button class="bg-green-500 border border-black rounded-md">
										<h1>"Current Picks are Saved"</h1>
										<p>"Go back to book"</p>
									</button>
								</a>
							}.into_view()
						}
					}}
				</div>
			</div>
		</div>
	}
}

#[component]
pub fn SpreadGroupPick(initial_values: Vec<(Event, Pick)>) -> impl IntoView {
	let global_picks = use_context::<WriteSignal<Vec<ReadSignal<Pick>>>>()
		.expect("You should have access to the picks");
	let global_discrepancies = use_context::<WriteSignal<Option<i32>>>()
		.expect("You should have access to discrepancies counter");

	let num_of_picks = initial_values.len();
	let wager_trackers = create_rw_signal(vec![0; num_of_picks]);

	let reactive_events: Vec<(Event, RwSignal<Pick>)> = initial_values
		.into_iter()
		.map(|(event, pick)| {
			if let Some(wager) = pick.wager {
				wager_trackers.update(|t| t[wager as usize - 1] += 1);
			}
			if pick.wager.is_none() { global_discrepancies.update(|d| *d = Some(d.unwrap_or(0) + 1)) }
			if pick.choice.is_none() { global_discrepancies.update(|d| *d = Some(d.unwrap_or(0) + 1)) }

			let new_pick = create_rw_signal(pick);
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
			} else if new_wager_right {
				global_discrepancies.update(|d| *d = Some(d.unwrap_or(1) - 1))
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

	view! {
		{
			reactive_events
				.into_iter()
				.enumerate()
				.map(|(i, (event, pick))| match event.contents {
					EventContent::SpreadGroup(spread) => {
						let old_pick = pick.get_untracked();
						view!{
							<div class="p-3">
								<div class="content-center justify-center max-w-sm overflow-hidden bg-white rounded-lg shadow-lg">
									<h1>"Game " {i+1}</h1>
									<Await future=move || get_spread_teams(spread.home_id, spread.away_id) let:spread_teams>
										{match spread_teams {
											Err(e) => format!("Could not find spread teams: {e}").into_view(),
											Ok((home_team, away_team)) => {
												view!{
													<div class="grid grid-flow-col grid-cols-2 gap-4 p-5">
														<div class="col-span-1">
															<h1>"Home"</h1>
															<input on:click=move |_| spread_setter(pick, "Home") type="radio" id={format!("{}", &home_team.id)} name={format!("{}-{}", &home_team.id, &away_team.id)} value="home" class="hidden peer" checked={if &old_pick.choice == &Some("Home".into()) {true} else {false}}/>
															<label for={format!("{}", &home_team.id)} class="inline-grid w-full p-5 pt-0 pb-0 border border-black rounded-lg cursor-pointer hover:border-green-700 peer-checked:bg-green-500 peer-checked:border-green-600 hover:bg-green-100">
																<img src=&home_team.logo class="w-full"/>
																<h2>{&home_team.name}</h2>
																<h2 class="pb-1 text-center">{format!("{:+}", spread.home_spread)}</h2>
															</label>
														</div>
														<div class="col-span-1">
															<h1>"Away"</h1>
															<input on:click=move |_| spread_setter(pick, "Away") type="radio" id={format!("{}", &away_team.id)} name={format!("{}-{}", &home_team.id, &away_team.id)} value="away" class="hidden peer" checked={if &old_pick.choice == &Some("Away".into()) {true} else {false}}/>
															<label for={format!("{}", &away_team.id)} class="inline-grid w-full p-5 pt-0 pb-0 border border-black rounded-lg cursor-pointer hover:border-green-700 peer-checked:bg-green-500 peer-checked:border-green-600 hover:bg-green-100">
																<img src=&away_team.logo class="w-full"/>
																<h2>{&away_team.name}</h2>
																<h2 class="pb-1 text-center">{format!("{:+}", -1. * spread.home_spread)}</h2>
															</label>
														</div>
													</div>
												}.into_view()
											}
										}}
									</Await>
									<ul>
									{
										(1..=num_of_picks)
											.map(|i| {
												view!{
													<li class="inline-flex items-center p-1">
														<input on:click=move |_| wager_setter(pick, i) type="radio" id={format!("{}-{}-{}-wager", i, spread.home_id, spread.away_id)} name={format!("{}-{}-wager", spread.home_id, spread.away_id)} value="home" class="hidden peer" checked={if old_pick.wager == Some(i as i64) {true} else {false}}/>
														<label for={format!("{}-{}-{}-wager", i, spread.home_id, spread.away_id)} class="inline-grid w-5 h-5 p-5 border border-black rounded-lg cursor-pointer hover:border-green-700 peer-checked:bg-green-500 peer-checked:border-green-600 hover:bg-green-100">
															<p class="text-center">{i}</p>
														</label>
													</li>
												}
											})
											.collect_view()
									}
									</ul>
									{
										if let Some(notes) = spread.notes {
											view!{
												<details class="pt-1 pb-3">
													<summary>"Notes"</summary>
													<p>
														<span style="white-space: pre-line">
															{notes}
														</span>
													</p>
												</details>
											}.into_view()
										} else {
											().into_view()
										}
									}
								</div>
							</div>

						}.into_view()
					},
					_ => {
						view!{
							<h1>"This should have been a spread pick. Something is wrong!!!!!!!"</h1>
						}.into_view()
					}
				})
				.collect_view()
		}
	}
}

#[component]
pub fn UserInputs(initial_values: Vec<(Event, Pick)>) -> impl IntoView {
	let global_picks = use_context::<WriteSignal<Vec<ReadSignal<Pick>>>>()
		.expect("You should have access to the picks");
	let global_discrepancies = use_context::<WriteSignal<Option<i32>>>()
		.expect("You should have access to discrepancies counter");

	let reactive_events: Vec<(Event, RwSignal<Pick>)> = initial_values
		.into_iter()
		.map(|(event, pick)| {
			if pick.choice.is_none() { global_discrepancies.update(|d| *d = Some(d.unwrap_or(0) + 1)) }

			let new_pick = create_rw_signal(pick);
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

	view! {
		{
			reactive_events
				.into_iter()
				.enumerate()
				.map(|(i, (event, pick))| match event.contents {
					EventContent::UserInput(user_input_event) => {
						view!{
							<div class="p-3">
								<div class="content-center justify-center max-w-sm overflow-hidden bg-white rounded-lg shadow-lg">
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
							</div>
						}
					},
					_ => panic!("You shouldn't be calling this component with something other than a UserInput")
				})
				.collect_view()
		}
	}
}
