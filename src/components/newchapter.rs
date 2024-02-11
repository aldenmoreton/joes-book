use leptos::*;
use leptos_router::{use_params_map, Redirect};

use crate::{
    components::{DateTimePickerTZ, TeamSelect},
    objects::{
        BookRole, BookSubscription, EventBuilder, EventContent, SpreadBuilder, Team,
        UserInputBuilder,
    },
    server::{add_chapter, get_book},
};

#[component]
pub fn NewChapter() -> impl IntoView {
    let params = use_params_map();
    let book_id: i64 = params
        .with_untracked(|params| params.get("book_id").cloned())
        .unwrap()
        .parse::<i64>()
        .unwrap();
    let user_subscription = create_resource(|| (), move |_| async move { get_book(book_id).await });

    view! {
        <Suspense fallback=|| "Loading user data">
            {move || match user_subscription.get() {
                Some(Ok(BookSubscription{role: BookRole::Owner, ..})) => VerifiedNewChapter().into_view(),
                None => ().into_view(),
                _ => view! { <Redirect path="/books/"/> }.into_view(),
            }}
        </Suspense>
    }
}

#[component]
pub fn VerifiedNewChapter() -> impl IntoView {
    let params = use_params_map();
    let book_id: i64 = params
        .with_untracked(|params| params.get("book_id").cloned())
        .unwrap()
        .parse::<i64>()
        .unwrap();

    let title = create_rw_signal(String::new());
    let events = create_rw_signal::<Vec<(i64, EventBuilder)>>(Vec::new());

    let initial_datetime = {
        let current = chrono::Utc::now();
        let naive = (current + chrono::Duration::days(1)).date_naive();
        let date = naive.format("%Y-%m-%d").to_string();
        let datetime = format!("{date}T11:00");
        datetime
    };

    let date_time_rfc3339 = create_rw_signal(format!("{initial_datetime}:00-06:00"));

    let untracked_changes = create_rw_signal(1);
    provide_context(untracked_changes.write_only());

    let add_event = move |event: &str| {
        let new_event = match event {
            "SpreadGroup" => {
                let new_spread = create_rw_signal(SpreadBuilder::default());
                let new_tuple = (0, new_spread);
                let new_group = create_rw_signal(vec![new_tuple]);
                EventBuilder::SpreadGroup(new_group)
            }
            "UserInput" => {
                let new_input = create_rw_signal(UserInputBuilder::new());
                EventBuilder::UserInput(new_input)
            }
            _ => panic!(),
        };

        events.update(move |events| {
            let next_id = if let Some(last_event) = events.last() {
                last_event.0 + 1
            } else {
                0
            };
            events.push((next_id, new_event))
        });
    };
    add_event("SpreadGroup");

    let submit = create_action(move |_| async move {
        let built_events: Result<Vec<Vec<EventContent>>, String> = events
            .get()
            .into_iter()
            .map(|(_, event)| event.build())
            .collect();

        let built_events = match built_events {
            Ok(events) => events,
            Err(e) => return async { Err(ServerFnError::Serialization(e)) }.await,
        };

        add_chapter(
            book_id,
            title.get(),
            date_time_rfc3339.get(),
            built_events.into_iter().flatten().collect(),
        )
        .await
    });

    let change_title = move |ev: leptos::ev::Event| {
        title.update(|t| {
            let new_title = event_target_value(&ev);

            let old_empty = t.len() == 0;
            let new_empty = new_title.len() == 0;

            if old_empty && !new_empty {
                untracked_changes.update(|changes| *changes -= 1)
            } else if new_empty && !old_empty {
                untracked_changes.update(|changes| *changes += 1)
            }

            *t = new_title
        })
    };

    view! {
        <div class="flex flex-col items-center justify-center border border-green-500">
        // <h1>"Untracked Changes: "{move || format!("{:?}", untracked_changes.get())}</h1>
            <h1>{move || untracked_changes.get()}</h1>
            <input type="text" class="border border-black" placeholder="Chapter Name" on:input=change_title/>
            <DateTimePickerTZ picker=date_time_rfc3339.write_only() initial_datetime/>
            <For
                each={move || events.get()}
                key={|event| event.0}
                let:event
            >
                {
                    let event_view = move || match event.1 {
                        EventBuilder::SpreadGroup(spreads) => view!{
                            <NewSpreadGroup spreads/>
                        },
                        EventBuilder::UserInput(input) => view!{
                            <NewUserInput input/>
                        }
                    };

                    view! {
                        <div class="grid items-center">
                            {event_view}
                        </div>
                    }
                }
            </For>
                <button on:click=move |_| add_event("UserInput") class="px-4 py-2 font-semibold text-green-700 bg-transparent border border-green-500 rounded hover:bg-green-500 hover:text-white hover:border-transparent">"Add Extra Point"</button>
        </div>
        <div class="p-3">
            {move ||
                if untracked_changes.get() > 0 {
                    view!{
                        <button class="px-4 py-2 font-semibold text-black border border-black rounded cursor-not-allowed bg-transparen w-30">"Incomplete"</button>
                    }.into_view()
                } else if submit.pending().get() {
                    view!{
                        <button class="px-4 py-2 font-semibold text-black border border-black rounded cursor-not-allowed bg-transparen w-30">"Creating..."</button>
                    }.into_view()
                } else if let Some(Ok(_new_chapter_id)) = submit.value().get() {
                    view!{
                        <Redirect path={format!("/books/{}", book_id)}/>
                    }.into_view()
                } else if let Some(Err(_)) = submit.value().get() {
                    ().into_view()
                } else {
                    view!{
                        <button on:click=move |ev|{ submit.dispatch(ev) } class="px-4 py-2 font-semibold text-white bg-black border border-black rounded w-30">"Submit"</button>
                    }.into_view()
                }
            }
        </div>
    }
}

#[component]
pub fn NewUserInput(input: RwSignal<UserInputBuilder>) -> impl IntoView {
    let total_invalid_count =
        use_context::<WriteSignal<i32>>().expect("Where's the invalid state counter?");
    total_invalid_count.update(|c| *c += 1);

    input.update(|input| input.points = Some(1));

    let change_question = move |event| {
        input.update(|input| {
            let new_question = event_target_value(&event);
            let new_length = new_question.len();

            if new_length == 0 {
                if input.question.is_some() {
                    total_invalid_count.update(|c| *c += 1)
                }
                input.question = None
            } else {
                if input.question.is_none() {
                    total_invalid_count.update(|c| *c -= 1)
                }
                input.question = Some(new_question)
            }
        })
    };

    view! {
        <div class="p-3">
            <div class="content-center justify-center max-w-sm overflow-hidden bg-white rounded-lg shadow-lg">
                <h1>"Extra Point"</h1>
                <div class="justify-center p-2">
                    <div class="relative h-30">
                        <textarea
                        class="peer resize-none h-full w-full rounded-[7px] border border-green-200 border-t-transparent bg-transparent px-3 py-2.5 font-sans text-sm font-normal text-green-700 outline outline-0 transition-all placeholder-shown:border placeholder-shown:border-green-200 placeholder-shown:border-t-green-200 focus:border-2 focus:border-t-transparent focus:outline-0 disabled:border-0 disabled:bg-green-50"
                        placeholder=""
                        on:input=change_question/>
                        <label class="before:content[' '] after:content[' '] pointer-events-none absolute left-0 -top-1.5 flex h-full w-full select-none text-[11px] font-normal leading-tight text-green-400 transition-all before:pointer-events-none before:mt-[6.5px] before:mr-1 before:box-border before:block before:h-1.5 before:w-2.5 before:rounded-tl-md before:border-t before:border-l before:border-green-200 before:transition-all after:pointer-events-none after:mt-[6.5px] after:ml-1 after:box-border after:block after:h-1.5 after:w-2.5 after:flex-grow after:rounded-tr-md after:border-t after:border-r after:border-green-200 after:transition-all peer-placeholder-shown:text-sm peer-placeholder-shown:leading-[3.75] peer-placeholder-shown:text-green-500 peer-placeholder-shown:before:border-transparent peer-placeholder-shown:after:border-transparent peer-focus:text-[11px] peer-focus:leading-tight peer-focus:text-gree-500 peer-focus:before:border-t-2 peer-focus:before:border-l-2 peer-focus:after:border-t-2 peer-focus:after:border-r-2 peer-disabled:text-transparent peer-disabled:before:border-transparent peer-disabled:after:border-transparent peer-disabled:peer-placeholder-shown:text-green-500">
                        "Question"
                        </label>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn NewSpreadGroup(spreads: RwSignal<Vec<(i64, RwSignal<SpreadBuilder>)>>) -> impl IntoView {
    let add_spread = move |_| {
        spreads.update(|spreads| {
            let new_id = if let Some(spread) = spreads.last() {
                spread.0 + 1
            } else {
                0
            };
            let new_spread = (new_id, create_rw_signal(SpreadBuilder::default()));
            spreads.push(new_spread)
        });
    };

    view! {
        <div class="flex flex-col items-center justify-center">
        // <h1 class="text-center">"Spread Group"</h1>
            <For
                each={move || spreads.get().into_iter().enumerate()}
                key={|spread| spread.0}
                let:spread
            >
                <h2>"Spread " {spread.0 + 1}</h2>
                <NewSpread spread=spread.1.1/>
            </For>
            <div class="p-3">
                <button on:click=add_spread class="px-4 py-2 font-semibold text-green-700 bg-transparent border border-green-500 rounded hover:bg-green-500 hover:text-white hover:border-transparent">"Add Spread"</button>
            </div>
        </div>
    }
}

#[component]
pub fn NewSpread(spread: RwSignal<SpreadBuilder>) -> impl IntoView {
    let total_invalid_count =
        use_context::<WriteSignal<i32>>().expect("Where's the invalid state counter?");
    total_invalid_count.update(|c| *c += 1);
    let local_invalid_count = create_rw_signal(1);

    let home_team = create_rw_signal::<Option<Team>>(None);
    let away_team = create_rw_signal::<Option<Team>>(None);

    let home_view = {
        move || match home_team.get() {
            Some(new_home_team) => {
                spread.update(|s| {
                    s.home_id = Some(new_home_team.id);
                    if s.home_id != s.away_id {
                        local_invalid_count.update(|c| *c -= 1);
                        total_invalid_count.update(|c| *c -= 1);
                    }
                });

                view! {
                    <button class="hover:bg-red-700 " on:click=move |_| home_team.set(None)>
                        <div class="col-span-1">
                            <h3 class="text-center">Home</h3>
                            <img src=new_home_team.logo class="object-contain w-full"/>
                            <h3 class="text-center">{new_home_team.name}</h3>
                        </div>
                    </button>
                }
                .into_view()
            }
            None => {
                spread.update(|s| {
                    if s.away_id.is_none() || s.home_id != s.away_id {
                        local_invalid_count.update(|c| *c += 1);
                        total_invalid_count.update(|c| *c += 1);
                    }
                    s.home_id = None
                });

                view! {
                    <div class="col-span-1">
                        <h3 class="text-center">Home</h3>
                        <TeamSelect team_selector=home_team.write_only()/>
                    </div>
                }
                .into_view()
            }
        }
    };

    let away_view = {
        move || match away_team.get() {
            Some(team) => {
                spread.update(|s| {
                    s.away_id = Some(team.id);
                    if s.home_id != s.away_id {
                        local_invalid_count.update(|c| *c -= 1);
                        total_invalid_count.update(|c| *c -= 1);
                    }
                });

                view! {
                    <button class="hover:bg-red-700" on:click=move |_| away_team.set(None)>
                        <div class="col-span-1">
                            <h3 class="text-center">Away</h3>
                            <img src=team.logo class="object-contain w-full"/>
                            <h3 class="text-center">{team.name}</h3>
                        </div>
                    </button>
                }
                .into_view()
            }
            None => {
                spread.update(|s| {
                    if s.home_id.is_none() || s.home_id != s.away_id {
                        local_invalid_count.update(|c| *c += 1);
                        total_invalid_count.update(|c| *c += 1);
                    }
                    s.away_id = None;
                });

                view! {
                    <div class="col-span-1">
                        <h3 class="text-center">Away</h3>
                        <TeamSelect team_selector=away_team.write_only()/>
                    </div>
                }
                .into_view()
            }
        }
    };

    let change_spread = move |ev| {
        let new_spread: Result<f64, _> = event_target_value(&ev).parse();

        spread.update(|s| match new_spread {
            Ok(new_val) => {
                if s.home_spread.is_none() {
                    local_invalid_count.update(|c| *c -= 1);
                    total_invalid_count.update(|c| *c -= 1);
                }
                s.home_spread = Some(new_val);
            }
            Err(_) => {
                if s.home_spread.is_some() {
                    local_invalid_count.update(|c| *c += 1);
                    total_invalid_count.update(|c| *c += 1);
                }
                s.home_spread = None;
            }
        })
    };

    let change_notes = move |ev| {
        let new_notes = event_target_value(&ev);

        spread.update(|s| {
            if new_notes.is_empty() {
                s.notes = None
            } else {
                s.notes = Some(new_notes);
            }
        })
    };

    view! {
        <div class="content-center justify-center max-w-sm overflow-hidden bg-white rounded-lg shadow-lg">
            <h3 class="text-right">{move || local_invalid_count.get()}</h3>
            <div class="grid grid-flow-col grid-cols-2 gap-4 p-5">
                {home_view}
                {away_view}
            </div>
            <div class="justify-center p-2">
                <div class="relative w-full h-10">
                    <input
                    class="peer h-full w-full rounded-[7px] border border-green-200 border-t-transparent bg-transparent px-3 py-2.5 font-sans text-sm font-normal text-green-700 outline outline-0 transition-all placeholder-shown:border placeholder-shown:border-green-200 placeholder-shown:border-t-green-200 focus:border-2 focus:border-t-transparent focus:outline-0 disabled:border-0 disabled:bg-green-50"
                    placeholder=""
                    on:input=change_spread/>
                    <label class="before:content[' '] after:content[' '] pointer-events-none absolute left-0 -top-1.5 flex h-full w-full select-none text-[11px] font-normal leading-tight text-green-400 transition-all before:pointer-events-none before:mt-[6.5px] before:mr-1 before:box-border before:block before:h-1.5 before:w-2.5 before:rounded-tl-md before:border-t before:border-l before:border-green-200 before:transition-all after:pointer-events-none after:mt-[6.5px] after:ml-1 after:box-border after:block after:h-1.5 after:w-2.5 after:flex-grow after:rounded-tr-md after:border-t after:border-r after:border-green-200 after:transition-all peer-placeholder-shown:text-sm peer-placeholder-shown:leading-[3.75] peer-placeholder-shown:text-green-500 peer-placeholder-shown:before:border-transparent peer-placeholder-shown:after:border-transparent peer-focus:text-[11px] peer-focus:leading-tight peer-focus:text-gree-500 peer-focus:before:border-t-2 peer-focus:before:border-l-2 peer-focus:after:border-t-2 peer-focus:after:border-r-2 peer-disabled:text-transparent peer-disabled:before:border-transparent peer-disabled:after:border-transparent peer-disabled:peer-placeholder-shown:text-green-500">
                    "Home Spread"
                    </label>
                </div>
            </div>
            <div class="justify-center p-2">
                <div class="relative h-30">
                    <textarea
                    class="peer resize-none h-full w-full rounded-[7px] border border-green-200 border-t-transparent bg-transparent px-3 py-2.5 font-sans text-sm font-normal text-green-700 outline outline-0 transition-all placeholder-shown:border placeholder-shown:border-green-200 placeholder-shown:border-t-green-200 focus:border-2 focus:border-t-transparent focus:outline-0 disabled:border-0 disabled:bg-green-50"
                    placeholder=""
                    on:input=change_notes/>
                    <label class="before:content[' '] after:content[' '] pointer-events-none absolute left-0 -top-1.5 flex h-full w-full select-none text-[11px] font-normal leading-tight text-green-400 transition-all before:pointer-events-none before:mt-[6.5px] before:mr-1 before:box-border before:block before:h-1.5 before:w-2.5 before:rounded-tl-md before:border-t before:border-l before:border-green-200 before:transition-all after:pointer-events-none after:mt-[6.5px] after:ml-1 after:box-border after:block after:h-1.5 after:w-2.5 after:flex-grow after:rounded-tr-md after:border-t after:border-r after:border-green-200 after:transition-all peer-placeholder-shown:text-sm peer-placeholder-shown:leading-[3.75] peer-placeholder-shown:text-green-500 peer-placeholder-shown:before:border-transparent peer-placeholder-shown:after:border-transparent peer-focus:text-[11px] peer-focus:leading-tight peer-focus:text-gree-500 peer-focus:before:border-t-2 peer-focus:before:border-l-2 peer-focus:after:border-t-2 peer-focus:after:border-r-2 peer-disabled:text-transparent peer-disabled:before:border-transparent peer-disabled:after:border-transparent peer-disabled:peer-placeholder-shown:text-green-500">
                    "Notes (Optional)"
                    </label>
                </div>
            </div>
        </div>
    }
}
