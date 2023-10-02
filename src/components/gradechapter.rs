use leptos::*;
use leptos_router::{use_params_map, Redirect};

use crate::{server::{get_spread_teams, get_book, get_events, save_answers, get_user_inputs}, objects::{Event, EventContent, BookSubscription, BookRole, Spread, UserInput}};

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

	let global_answers: RwSignal<Vec<ReadSignal<Option<(i64, Vec<String>)>>>> = create_rw_signal(cx, Vec::new());
	provide_context(cx, global_answers);

	let pick_views = events.into_iter()
		.map(move |event| {
			match event.contents {
				EventContent::SpreadGroup(spread) => view!{cx,
					<SpreadGroupGrade id=event.id spread/>
				}.into_view(cx),
				EventContent::UserInput(input) => view!{cx,
					<UserInputGrade id=event.id question=input/>
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
		{move ||
			global_answers
				.get()
				.into_iter()
				.map(|answer| answer.get())
				.flatten()
				.map(|answer| format!("{}-{:?}<br/>", answer.0, answer.1))
				.collect::<Vec<_>>()
		}
		<div class="grid items-center justify-center h-16">
			<div class="content-center self-center justify-center w-32 h-full text-center">
				{move || match pick_submission.pending().get() {
					false => view!{cx, <button on:click=move |_| pick_submission.dispatch(()) class="w-full h-full text-white bg-black rounded-xl">"Submit"</button>},
					true => view!{cx, <button disabled on:click=move |_| pick_submission.dispatch(()) class="w-full h-full text-black bg-gray-400 rounded-xl">"Pending"</button>}
				}}
			</div>
		</div>
		{move || match pick_submission.value().get() {
			None => ().into_view(cx),
			Some(Err(e)) => format!("Error saving picks: {e}").into_view(cx),
			Some(Ok(_)) => view!{cx, <Redirect path=format!("/books/{book_id}")/>}
		}}
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
										<input on:click=move |_| answer_setter("Push") type="radio" id={format!("{}-push", &away_team.id)} name={format!("{}-{}", &home_team.id, &away_team.id)} value="away" class="hidden peer"/>
										<label for={format!("{}-push", &away_team.id)} class="inline-grid w-1/2 pt-1 pb-1 mb-3 border border-black rounded-lg cursor-pointer hover:border-green-700 peer-checked:bg-green-500 peer-checked:border-green-600 hover:bg-green-100">
											<h2>"Push"</h2>
										</label>
									}.into_view(cx)
							}
						})
					}
				</Suspense>
			</div>
		</div>

	}
}

#[component]
pub fn UserInputGrade(cx: Scope, id: i64, question: UserInput) -> impl IntoView {
	let global_answers = use_context::<RwSignal<Vec<ReadSignal<Option<(i64, Vec<String>)>>>>>(cx)
		.expect("You should have access to the picks");

	let input_answers: RwSignal<Option<(i64, Vec<String>)>> = create_rw_signal(cx, None);
	global_answers.update(|answers| answers.push(input_answers.read_only()));

	let answer_setter = move |add: bool, choice: &str| {
		let pos = input_answers.get().map(|answers| answers.1.iter().position(|x| *x == choice));
		match (add, pos) {
			(true, None) => input_answers.set(Some((id, vec![choice.into()]))),
			(true, Some(None)) => input_answers.update(|answers| answers.as_mut().unwrap().1.push(choice.into())),
			(true, Some(Some(_))) => (),
			(false, None) => (),
			(false, Some(None)) => (),
			(false, Some(Some(i))) => {
				input_answers.update(|answers| {
					answers.as_mut().unwrap().1.remove(i);
					if answers.as_ref().unwrap().1.len() == 0 {
						*answers = None;
					};
				});
			}
		}
	};

	let inputs_getter = create_resource(cx,
		|| (),
		move |_| get_user_inputs(cx, id)
	);
	let question_str = question.question.clone();
	view!{cx,
		<div class="p-3">
			<div class="content-center justify-center max-w-sm overflow-hidden bg-white rounded-lg shadow-lg">
				<h1 class="mt-1">{question_str}</h1>
				<Suspense fallback=move || view!{cx, <p>"Loading..."</p>}>
					{move ||
						inputs_getter.read(cx).map(|inputs| {
							match inputs {
								Err(e) => format!("Could not load user inputs: {e}").into_view(cx),
								Ok(inputs) => {
									inputs
										.into_iter()
										.map(|user_input| {
											let (remove_input, add_input) = (user_input.clone(), user_input.clone());
											view!{cx,
												<div class="grid grid-flow-col grid-cols-2 gap-4 p-5 border border-gray-100">
													<h1>{&user_input}</h1>
													<div class="col-span-1">
														<input on:click=move |_| answer_setter(false, &remove_input) type="radio" id=format!("{}-Remove", &user_input) name=format!("{}", &user_input) value="wrong" class="hidden peer"/>
														<label for=format!("{}-Remove", user_input) class="inline-grid w-full p-5 pt-0 pb-0 border border-black rounded-lg cursor-pointer hover:border-red-700 peer-checked:bg-red-500 peer-checked:border-red-600 hover:bg-red-100">
															<h2>"Wrong"</h2>
														</label>
													</div>
													<div class="col-span-1">
														<input on:click=move |_| answer_setter(true, &add_input) type="radio" id=format!("{}-Add", &user_input) name=format!("{}", &user_input) value="right" class="hidden peer"/>
														<label for=format!("{}-Add", user_input) class="inline-grid w-full p-5 pt-0 pb-0 border border-black rounded-lg cursor-pointer hover:border-green-700 peer-checked:bg-green-500 peer-checked:border-green-600 hover:bg-green-100">
															<h2>"Right"</h2>
														</label>
													</div>
												</div>
											}
										})
										.collect_view(cx)
								}
							}
						})
					}
				</Suspense>
			</div>
		</div>
	}
}
