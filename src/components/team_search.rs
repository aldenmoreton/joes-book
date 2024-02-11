use leptos::*;

use crate::{objects::Team, server::search_team};

#[component]
pub fn TeamSelect(team_selector: WriteSignal<Option<Team>>) -> impl IntoView {
    let (query, set_query) = create_signal("-1".to_string());

    let teams = create_resource(move || query.get(), move |_| search_team(query.get()));

    view! {
        <div>
            <label for="default-search" class="mb-2 text-sm font-medium text-black sr-only dark:text-black">Search</label>
            <div class="relative">
                <div class="absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none">
                    <svg class="w-4 h-4 text-black dark:text-black" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 20 20">
                        <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="m19 19-4-4m0-7A7 7 0 1 1 1 8a7 7 0 0 1 14 0Z"/>
                    </svg>
                </div>
                <input type="search" id="default-search" class="block w-full p-4 pl-10 text-sm text-black border border-gray-300 rounded-lg bg-green-50 focus:ring-blue-500 focus:border-blue-500 dark:bg-green-50 dark:border-gray-600 dark:placeholder-gray-400 dark:text-black dark:focus:ring-blue-500 dark:focus:border-blue-500" placeholder="Team Search"
                    on:input=move |ev|{
                        let new_query = event_target_value(&ev);
                        if new_query.len() >= 3 {
                            set_query.set(new_query);
                        } else if new_query.len() == 0 {
                            set_query.set("-1".into());
                        }
                    }/>
            </div>
            <Transition fallback=move || view! { <p>"Loading..."</p> }>
                {move || {
                    let teams_list_items = {
                        move || {
                            teams.get()
                                .map(move |teams| match teams {
                                    Err(e) => {
                                        view! {<pre class="error">"Server Error: " {e.to_string()}</pre>}.into_view()
                                    }
                                    Ok(teams) => {
                                        if teams.is_empty() {
                                            view! { <p></p> }.into_view()
                                        } else {
                                            teams
                                                .into_iter()
                                                .map(move |team| {
                                                    let team_select = team.clone();
                                                    view!{
                                                        <li>
                                                            <img src=team.logo width="25" height="25"/>
                                                            <button on:click=move |_| {set_query.set("-1".into()); team_selector.set(Some(team_select.clone()))}>{team.name}</button>
                                                        </li>
                                                    }
                                                })
                                                .collect_view()
                                        }
                                    }
                                }
                            ).unwrap_or_default()
                        }
                    };

                    view!{
                        <ul>
                            {teams_list_items}
                        </ul>
                    }
                }
            }
            </Transition>
        </div>
    }
}
