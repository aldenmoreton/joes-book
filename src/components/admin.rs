use leptos::*;
use leptos_router::{Redirect, ActionForm};

use crate::{
	server::{
		has_permission,
		search_user,
		MakeAdmin
	},
	objects::FrontendUser
};

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

#[component]
pub fn UserSelect(
    cx: Scope,
	user_selector: WriteSignal<Option<FrontendUser>>
) -> impl IntoView {
	let (query, set_query) = create_signal::<Option<String>>(cx, None);

	let users = create_resource(
		cx,
		move || query.get(),
		move |_| async move {
			if let Some(query) = query.get_untracked() {
				search_user(cx, query).await
			} else {
				Ok(Vec::new())
			}
		}
	);

    view! {
        cx,
        <div>
			<input type="text" class="border border-black bg-green-300" on:input=move |ev|{
				let new_query = event_target_value(&ev);
				 if new_query.len() == 0 {
					set_query.set(None);
				} else {
					set_query.set(Some(new_query));
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
                                            ().into_view(cx)
                                        } else {
                                            users
                                                .into_iter()
                                                .map(move |user| {
													let user_select = user.clone();
                                                    view! {
                                                        cx,
                                                        <li>
															<button class="border border-black bg-gray-50 rounded-md m-1"
																on:click=move |_| {
																	set_query.set(None);
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
