use leptos::*;
use leptos_router::{use_params_map, Redirect};

use crate::{
    objects::{BookRole, BookSubscription, Event, EventContent, Spread, UserInput},
    server::{get_book, get_events, get_spread_teams, get_user_inputs, save_answers},
};

#[component]
pub fn GradeChapter() -> impl IntoView {
    let params = use_params_map();
    let book_id: i64 = params
        .with_untracked(|params| params.get("book_id").cloned())
        .unwrap()
        .parse::<i64>()
        .unwrap();
    let user_subscription = create_resource(|| (), move |_| async move { get_book(book_id).await });

    view! {
        <Suspense fallback=|| "Loading user data">
            {move || user_subscription.get().map(|subscription| {
                match subscription {
                    Err(e) => format!("{e}").into_view(),
                    Ok(BookSubscription{role: BookRole::Owner, ..}) => VerifiedGradeChapter().into_view(),
                    Ok(_) => view! { <Redirect path=format!("/books/{book_id}")/> }.into_view()
                }
            })
            }
        </Suspense>
    }
}

#[component]
pub fn VerifiedGradeChapter() -> impl IntoView {
    let params = use_params_map();
    let chapter_id: i64 = params
        .with_untracked(|params| params.get("chapter_id").cloned())
        .unwrap()
        .parse::<i64>()
        .unwrap();

    let events_fetcher = create_resource(move || (), move |_| get_events(chapter_id));

    view! {
        <Transition fallback=|| "Loading...">
            <div class="flex flex-col items-center justify-center">
                {move ||
                    events_fetcher.get().map(|events| match events {
                        Err(e) => {
                            view! {<pre class="error">"Server Error with event fetcher: " {e.to_string()}</pre>}.into_view()
                        },
                        Ok(events) => {
                            view!{
                                <ChapterEvents events/>
                            }.into_view()
                        }
                    })
                }
            </div>
        </Transition>
    }
}

#[component]
pub fn ChapterEvents(events: Vec<Event>) -> impl IntoView {
    let params = use_params_map();
    let book_id: i64 = params
        .with_untracked(|params| params.get("book_id").cloned())
        .unwrap()
        .parse::<i64>()
        .unwrap();
    let chapter_id: i64 = params
        .with_untracked(|params| params.get("chapter_id").cloned())
        .unwrap()
        .parse::<i64>()
        .unwrap();

    let global_answers: RwSignal<Vec<ReadSignal<Option<(i64, Vec<String>)>>>> =
        create_rw_signal(Vec::new());
    provide_context(global_answers);

    let pick_views = events
        .into_iter()
        .map(move |event| match event.contents {
            EventContent::SpreadGroup(spread) => view! {
                <SpreadGroupGrade id=event.id spread/>
            }
            .into_view(),
            EventContent::UserInput(input) => view! {
                <UserInputGrade id=event.id question=input/>
            }
            .into_view(),
        })
        .collect_view();

    let pick_submission = create_action(move |_| async move {
        let picks: Vec<(i64, Vec<String>)> = global_answers
            .get()
            .into_iter()
            .map(|answer| answer.get())
            .flatten()
            .map(|answer| (answer.0, answer.1))
            .collect();
        save_answers(picks).await
    });

    view! {
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
                    false => view!{<button on:click=move |_| pick_submission.dispatch(()) class="w-full h-full text-white bg-black rounded-xl">"Submit"</button>},
                    true => view!{<button disabled on:click=move |_| pick_submission.dispatch(()) class="w-full h-full text-black bg-gray-400 rounded-xl">"Pending"</button>}
                }}
            </div>
        </div>
        {move || match pick_submission.value().get() {
            None => ().into_view(),
            Some(Err(e)) => format!("Error saving picks: {e}").into_view(),
            Some(Ok(_)) => view!{<Redirect path=format!("/books/{book_id}/chapters/{chapter_id}")/>}
        }}
    }
}

#[component]
pub fn SpreadGroupGrade(id: i64, spread: Spread) -> impl IntoView {
    let global_answers = use_context::<RwSignal<Vec<ReadSignal<Option<(i64, Vec<String>)>>>>>()
        .expect("You should have access to the picks");

    let spread_answer: RwSignal<Option<(i64, Vec<String>)>> = create_rw_signal(None);
    global_answers.update(|answers| answers.push(spread_answer.read_only()));

    let answer_setter = move |choice: &str| match choice {
        "Home" | "Away" | "Push" => spread_answer.set(Some((id, vec![choice.into()]))),
        _ => spread_answer.set(None),
    };

    let teams_getter = create_resource(
        || (),
        move |_| get_spread_teams(spread.home_id, spread.away_id),
    );

    view! {
        <div class="p-3">
            <div class="content-center justify-center max-w-sm overflow-hidden bg-white rounded-lg shadow-lg">
                <Suspense fallback=move || view!{<p>"Loading..."</p>}>
                    {
                        move || teams_getter.get().map(|result| {
                            match result {
                                Err(e) => format!("Could not get teams:\n{e}").into_view(),
                                Ok((home_team, away_team)) =>
                                    view!{
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
                                    }.into_view()
                            }
                        })
                    }
                </Suspense>
            </div>
        </div>

    }
}

#[component]
pub fn UserInputGrade(id: i64, question: UserInput) -> impl IntoView {
    let global_answers = use_context::<RwSignal<Vec<ReadSignal<Option<(i64, Vec<String>)>>>>>()
        .expect("You should have access to the picks");

    let input_answers: RwSignal<Option<(i64, Vec<String>)>> = create_rw_signal(None);
    global_answers.update(|answers| answers.push(input_answers.read_only()));

    let answer_setter = move |add: bool, choice: &str| {
        let pos = input_answers
            .get()
            .map(|answers| answers.1.iter().position(|x| *x == choice));
        match (add, pos) {
            (true, None) => input_answers.set(Some((id, vec![choice.into()]))),
            (true, Some(None)) => {
                input_answers.update(|answers| answers.as_mut().unwrap().1.push(choice.into()))
            }
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

    let inputs_getter = create_resource(|| (), move |_| get_user_inputs(id));
    // let question_str = create_write(question.question.clone());
    let (question_str, _) = create_signal(question.question.clone());
    view! {
        <div class="p-3">
            <div class="content-center justify-center max-w-sm overflow-hidden bg-white rounded-lg shadow-lg">
                <h1 class="mt-1">{format!("{}", question_str.get())}</h1>
                <Suspense fallback=move || view!{<p>"Loading..."</p>}>
                    {move ||
                        inputs_getter.get().map(|inputs| {
                            match inputs {
                                Err(e) => format!("Could not load user inputs: {e}").into_view(),
                                Ok(inputs) => {
                                    inputs
                                        .into_iter()
                                        .enumerate()
                                        .map(|(i, user_input)| {
                                            let (remove_input, add_input) = (user_input.clone(), user_input.clone());
                                            view!{
                                                <div class="grid grid-flow-col grid-cols-2 gap-4 p-5 border border-gray-100">
                                                    <h1>{&user_input}</h1>
                                                    <div class="col-span-1">
                                                        <input on:click=move |_| answer_setter(false, &remove_input) type="radio" id=format!("{}-{}-Remove-{}", i, &user_input, question_str.get()) name=format!("{}-{}-{}", i, user_input, question_str.get()) value="wrong" class="hidden peer" checked/>
                                                        <label for=format!("{}-{}-Remove-{}", i, user_input, question_str.get()) class="inline-grid w-full p-5 pt-0 pb-0 border border-black rounded-lg cursor-pointer hover:border-red-700 peer-checked:bg-red-500 peer-checked:border-red-600 hover:bg-red-100">
                                                            <h2>"Wrong"</h2>
                                                        </label>
                                                    </div>
                                                    <div class="col-span-1">
                                                        <input on:click=move |_| answer_setter(true, &add_input) type="radio" id=format!("{}-{}-Add-{}", i, user_input, question_str.get()) name=format!("{}-{}-{}", i, user_input, question_str.get()) value="right" class="hidden peer"/>
                                                        <label for=format!("{}-{}-Add-{}", i, user_input, question_str.get()) class="inline-grid w-full p-5 pt-0 pb-0 border border-black rounded-lg cursor-pointer hover:border-green-700 peer-checked:bg-green-500 peer-checked:border-green-600 hover:bg-green-100">
                                                            <h2>"Right"</h2>
                                                        </label>
                                                    </div>
                                                </div>
                                            }
                                        })
                                        .collect_view()
                                }
                            }
                        })
                    }
                </Suspense>
            </div>
        </div>
    }
}
