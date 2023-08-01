use leptos::*;
use cfg_if::cfg_if;
use serde::{Serialize, Deserialize};

cfg_if! {
	if #[cfg(feature = "ssr")] {
		use crate::components::pool;
	}
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Team {
	pub id: i64,
	pub name: String,
	logo: String
}

#[server(SearchTeam, "/secure")]
pub async fn search_team(cx: Scope, name: String) -> Result<Vec<Team>, ServerFnError> {
    let pool = pool(cx)?;

    let result = sqlx::query_as::<_, Team>("SELECT * FROM teams WHERE LOWER(name) LIKE '%' || LOWER($1) || '%' ORDER BY name LIMIT 5")
        .bind(name)
        .fetch_all(&pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

	Ok(result)
}

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
			{move || log!("{}", query.get())}
			<Transition fallback=move || view! {cx, <p>"Loading..."</p> }>
                {move || {
                    let teams_li = {
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
                                })
                                .unwrap_or_default()
                        }
                    };

                    view! {
                        cx,
                        <ul>
                            {teams_li}
                        </ul>
                    }
                }
            }
            </Transition>
        </div>
    }
}