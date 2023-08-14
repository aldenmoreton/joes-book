use leptos::*;
use leptos_router::{Redirect, ActionForm};

use crate::auth::{has_permission, FrontendUser};

#[component]
pub fn Admin(cx: Scope) -> impl IntoView {
	let user = create_resource(
		cx,
		|| (),
		move |_| async move {
			has_permission(cx, "owner".into()).await.unwrap_or(false)
		}
	);

	view!{cx,
		<Suspense fallback=|| "Loading user data">
			{move || match user.read(cx) {
				Some(true) => AdminVerified(cx).into_view(cx),
				Some(false) => view! { cx, <Redirect path="/"/> }.into_view(cx),
				None => ().into_view(cx),
			}}
		</Suspense>
	}
}

#[server(MakeAdmin, "/secure")]
pub async fn make_admin(cx:Scope, id: i64) -> Result<bool, ServerFnError> {
	use crate::components::pool;
	let pool = pool(cx)?;

	let result = sqlx::query(
		r#"	SELECT user_id
			FROM user_permissions
			WHERE user_id=$1 AND token=$2"#,
	)
		.bind(id)
		.bind("admin")
		.fetch_optional(&pool)
		.await?;

	if result.is_some() { return Ok(false) }

	sqlx::query(
		r#"	INSERT INTO user_permissions (user_id, token)
			VALUES ($1, $2)"#
	)
		.bind(id)
		.bind("admin")
		.execute(&pool)
		.await?;

	Ok(true)
}

#[component]
pub fn AdminVerified(cx: Scope) -> impl IntoView {
	let (user, user_selector) = create_signal(cx, None);

	let make_admin = create_server_action::<MakeAdmin>(cx);

	view! {cx,
		<h1>"You must be important"</h1>
		<UserSelect user_selector/>
		{move ||
			match user.get() {
				Some(user) => view! {cx,
					<h3>"You chose "{user.username}</h3>
					<ActionForm action=make_admin>
						<input type="hidden" name="id" value={user.id}/>
						<input type="submit" value="Add admin rights"/>
					</ActionForm>
					<button on:click=move |_| {user_selector.set(None)}>"Deselect"</button>
				}.into_view(cx),
				None => ().into_view(cx)
			}
		}
	}
}

#[server(SearchUser, "/secure")]
pub async fn search_user(cx: Scope, username: String) -> Result<Vec<FrontendUser>, ServerFnError> {
	use crate::components::pool;
	let pool = pool(cx)?;

	let result = sqlx::query_as::<_, FrontendUser>(
		r#"	SELECT id, username
			FROM users
			WHERE LOWER(username) LIKE '%' || LOWER($1) || '%'
			ORDER BY username LIMIT 5"#
	)
        .bind(username)
        .fetch_all(&pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

	Ok(result)
}

#[component]
pub fn UserSelect(
    cx: Scope,
	user_selector: WriteSignal<Option<FrontendUser>>
) -> impl IntoView {
	let (query, set_query) = create_signal(cx, "".to_string());

	let users = create_resource(
		cx,
		move || query.get(),
		move |_| { search_user(cx, query.get()) }
	);

    view! {
        cx,
        <div>
			<input type="text" on:input=move |ev|{
				let new_query = event_target_value(&ev);
				 if new_query.len() == 0 {
					set_query.set("".into());
				} else {
					set_query.set(new_query);
				}
			}/>
			<Transition fallback=move || view! {cx, <p>"Loading..."</p> }>
                {move || {
                    let users_list_items = {
                        move || {
                            users.read(cx)
                                .map(move |users| match users {
                                    Err(e) => {
                                        view! { cx, <pre class="error">"Server Error: " {e.to_string()}</pre>}.into_view(cx)
                                    }
                                    Ok(users) => {
                                        if users.is_empty() {
                                            view! { cx, <p></p> }.into_view(cx)
                                        } else {
                                            users
                                                .into_iter()
                                                .map(move |user| {
													let user_select = user.clone();
                                                    view! {
                                                        cx,
                                                        <li>
															<button
																on:click=move |_| {
																	set_query.set("-1".into());
																	user_selector.set(Some(user_select.clone()))
																}>
																{user.username}
															</button>
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
                            {users_list_items}
                        </ul>
                    }
                }
            }
            </Transition>
        </div>
    }
}
