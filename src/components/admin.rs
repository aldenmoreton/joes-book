use leptos::*;
use leptos_router::{ActionForm, Redirect};

use crate::{
    objects::FrontendUser,
    server::{has_permission, search_user, MakeAdmin},
};

#[component]
pub fn Admin() -> impl IntoView {
    let user = create_resource(
        || (),
        move |_| async move { has_permission("owner".into()).await.unwrap_or(false) },
    );

    view! {
        <Suspense fallback=|| "Loading user data">
            {move || match user.get() {
                Some(true) => AdminVerified().into_view(),
                Some(false) => view! { <Redirect path="/"/> }.into_view(),
                None => ().into_view(),
            }}
        </Suspense>
    }
}

#[component]
pub fn AdminVerified() -> impl IntoView {
    let (user, user_selector) = create_signal(None);

    let make_admin = create_server_action::<MakeAdmin>();

    view! {
        <h1>"You must be important"</h1>
        <UserSelect user_selector/>
        {move ||
            match user.get() {
                Some(user) => view! {
                    <h3>"You chose "{user.username}</h3>
                    <ActionForm action=make_admin>
                        <input type="hidden" name="id" value={user.id}/>
                        <input type="submit" value="Add admin rights"/>
                    </ActionForm>
                    <button on:click=move |_| {user_selector.set(None)}>"Deselect"</button>
                }.into_view(),
                None => ().into_view()
            }
        }
    }
}

#[component]
pub fn UserSelect(user_selector: WriteSignal<Option<FrontendUser>>) -> impl IntoView {
    let (query, set_query) = create_signal::<Option<String>>(None);

    let users = create_resource(
        move || query.get(),
        move |_| async move {
            if let Some(query) = query.get_untracked() {
                search_user(query).await
            } else {
                Ok(Vec::new())
            }
        },
    );

    view! {
        <div>
            <input type="text" class="bg-green-300 border border-black" on:input=move |ev|{
                let new_query = event_target_value(&ev);
                 if new_query.is_empty() {
                    set_query.set(None);
                } else {
                    set_query.set(Some(new_query));
                }
            }/>
            <Transition fallback=move || view! { <p>"Loading..."</p> }>
                {move || {
                    let users_list_items = {
                        move || {
                            users.get()
                                .map(move |users| match users {
                                    Err(e) => {
                                        view! { <pre class="error">"Server Error: " {e.to_string()}</pre>}.into_view()
                                    }
                                    Ok(users) => {
                                        if users.is_empty() {
                                            ().into_view()
                                        } else {
                                            users
                                                .into_iter()
                                                .map(move |user| {
                                                    let user_select = user.clone();
                                                    view! {
                                                        <li>
                                                            <button class="m-1 border border-black rounded-md bg-gray-50"
                                                                on:click=move |_| {
                                                                    set_query.set(None);
                                                                    user_selector.set(Some(user_select.clone()))
                                                                }>
                                                                {user.username}
                                                            </button>
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

                    view! {
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
