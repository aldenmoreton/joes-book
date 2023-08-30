use leptos::*;
use leptos_router::{use_params_map, Redirect};

use crate::{server::{get_picks, get_spread_teams, save_picks}, objects::{Event, Pick, EventContent}};


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
									<ChapterEvents initial_values=events/>
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
pub fn ChapterEvents(cx: Scope, initial_values: Vec<(String, Vec<(Event, Pick)>)>) -> impl IntoView {
	let params = use_params_map(cx);
	let book_id: i64 = params.with_untracked(|params| params.get("book_id").cloned()).unwrap().parse::<i64>().unwrap();

	let discrepancies: RwSignal<Option<i32>> = create_rw_signal(cx, None);
	provide_context(cx, discrepancies.write_only());

	let global_picks: RwSignal<Vec<ReadSignal<Pick>>> = create_rw_signal(cx, Vec::new());
	provide_context(cx, global_picks.write_only());

	let pick_views = initial_values.into_iter()
		.map(move |value| {
			match value.0.as_str() {
				"SpreadGroup" => view!{cx,
					<div class="border border-black">
						<h1>"SpreadGroup"</h1>
						<SpreadGroupPick initial_values=value.1/>
					</div>
				}.into_view(cx),
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
		<h1>"Chapter Events"</h1>
		{pick_views}
		<div class="fixed inset-x-0 bottom-0 h-16 items-center justify-center grid">
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
									<Redirect path={format!("/books/{}", book_id)}/>
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
						// if !pick_submission.pending().get() {
						// 	view!{cx,
						// 		<button on:click=move |_| pick_submission.dispatch(()) class="h-full w-full bg-black text-white rounded-xl">"Submit"</button>
						// 	}.into_view(cx)
						// } else if let Some(Ok(_)) = pick_submission.value().get() {
						// 	view!{cx,
						// 		<Redirect path={format!("/books/{}", book_id)}/>
						// 	}.into_view(cx)
						// } else if let Some(Err(e)) = pick_submission.value().get() {
						// 	view!{cx,
						// 		<h1>{e}</h1>
						// 	}.into_view(cx)
						// } else {
						// 	view!{cx,
						// 		<p>"Loading..."</p>
						// 	}.into_view(cx)
						// }
					},
					Some(d) => {
						view!{cx,
							<h1>"Picks aren't finished yet: "{d}</h1>
						}.into_view(cx)
					},
					None => {
						().into_view(cx)
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
				log!("Some wager");
				wager_trackers.update(|t| t[wager as usize - 1] += 1);
			}
			if pick.wager.is_none() { global_discrepancies.update(|d| *d = Some(d.unwrap_or(0) + 1)) }
			// else {  }
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
			log!("Update wager for pick: {pick:?}");
			let new_wager_right = wager_trackers.get()[wager-1] == 0;
			if let Some(old_wager) = pick.wager {
				let old_wager_wrong = wager_trackers.get()[old_wager as usize - 1] > 1;
				if old_wager_wrong && new_wager_right {
					global_discrepancies.update(|d| *d = Some(d.unwrap_or(1) - 1))
				} else if !old_wager_wrong && !new_wager_right {
					global_discrepancies.update(|d| *d = Some(d.unwrap_or(0) + 1))
				}
			} else {
				if new_wager_right {
					global_discrepancies.update(|d| *d = Some(d.unwrap_or(1) - 1))
				}
			}

			// if no_old_wager && new_wager_right {
			// 	global_discrepancies.update(|d| *d = Some(d.unwrap_or(1) - 1))
			// } else if  { global_discrepancies.update(|d| *d = Some(d.unwrap_or(1) - 1)) }

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
		<h1>"Tracker: " {move || wager_trackers.get()}</h1>
		{
			reactive_events
				.into_iter()
				.enumerate()
				.map(|(i, (event, pick))| match event.contents {
					EventContent::SpreadGroup{spread} => {
						let old_pick = pick.get();
						view!{cx,
							<div class="p-3">
								<div class="max-w-sm rounded-lg overflow-hidden shadow-lg justify-center content-center bg-white">
									// <h1>{format!("{:?}", spread)}</h1>
									// <h1>{move || format!("{:?}", pick)}</h1>
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
												log!("{i:?}");
												view!{cx,
													<li class="inline-flex p-1 items-center">
														<input on:click=move |_| wager_setter(pick, i) type="radio" id={format!("{}-{}-{}-wager", i, spread.home_id, spread.away_id)} name={format!("{}-{}-wager", spread.home_id, spread.away_id)} value="home" class="hidden peer" checked={if old_pick.wager == Some(i as i64) {true} else {false}}/>
														// {
														// 	match wager_trackers.get()[i-1] {
														// 		0 => view!{cx,
														// 			<label for={format!("{}-{}-{}-wager", i, spread.home_id, spread.away_id)} class="inline-grid w-5 h-5 p-5 border border-black rounded-lg cursor-pointer hover:border-green-700 peer-checked:bg-green-500 peer-checked:border-green-600 hover:bg-green-100">
														// 				<p class="text-center">{i}</p>
														// 			</label>
														// 		},
														// 		1 => view!{cx,
														// 			<label for={format!("{}-{}-{}-wager", i, spread.home_id, spread.away_id)} class="inline-grid w-5 h-5 p-5 border border-black bg-green-700 rounded-lg cursor-pointer hover:border-green-700 peer-checked:bg-green-500 peer-checked:border-green-600 hover:bg-green-100">
														// 				<p class="text-center">{i}</p>
														// 			</label>
														// 		},
														// 		2.. => view!{cx,
														// 			<label for={format!("{}-{}-{}-wager", i, spread.home_id, spread.away_id)} class="inline-grid w-5 h-5 p-5 border border-black bg-red-500 rounded-lg cursor-pointer hover:border-red-700 peer-checked:bg-red-500 peer-checked:border-red-600 hover:bg-red-100">
														// 				<p class="text-center">{i}</p>
														// 			</label>
														// 		},
														// 		i32::MIN..=-1 => panic!("You can't wager a negative number")
														// 	}
														// }
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
													<p>{notes}</p>
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
					EventContent::UserInput(_input) => {
						view!{cx,
							<h1>"Not implemented for User Input yet"</h1>
						}.into_view(cx)
					}
				})
				.collect_view(cx)
		}
	}
}

