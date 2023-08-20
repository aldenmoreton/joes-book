use leptos::*;

use crate::{
    server::search_team,
    objects::Team
};

#[component]
pub fn TeamSelect(
    cx: Scope,
	team_selector: WriteSignal<Option<Team>>
) -> impl IntoView {
	let (query, set_query) = create_signal(cx, "-1".to_string());

	let teams = create_resource(
		cx,
		move || query.get(),
		move |_| { search_team(cx, query.get()) }
	);

    view! {
        cx,
        <div>
			<input type="text" on:input=move |ev|{
				let new_query = event_target_value(&ev);
				if new_query.len() >= 3 {
					set_query.set(new_query);
				} else if new_query.len() == 0 {
					set_query.set("-1".into());
				}
			}/>
			<Transition fallback=move || view! {cx, <p>"Loading..."</p> }>
                {move || {
                    let teams_list_items = {
                        move || {
                            teams.read(cx)
                                .map(move |teams| match teams {
                                    Err(e) => {
                                        view! { cx, <pre class="error">"Server Error: " {e.to_string()}</pre>}.into_view(cx)
                                    }
                                    Ok(teams) => {
                                        if teams.is_empty() {
                                            view! { cx, <p></p> }.into_view(cx)
                                        } else {
                                            teams
                                                .into_iter()
                                                .map(move |team| {
													let team_select = team.clone();
                                                    view! {
                                                        cx,
                                                        <li>
															<img src=team.logo width="25" height="25"/>
															<button on:click=move |_| {set_query.set("-1".into()); team_selector.set(Some(team_select.clone()))}>{team.name}</button>
                                                        </li>
                                                    }
                                                })
                                                .collect_view(cx)
                                        }
                                    }
                                }
                            ).unwrap_or_default()
                        }
                    };

                    view! {
                        cx,
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