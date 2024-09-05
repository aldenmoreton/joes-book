use leptos::*;
use leptos_router::{ActionForm, Redirect};

use crate::server::{get_books, has_permission, AddBook};

#[component]
pub fn Books() -> impl IntoView {
    let add_book = create_server_action::<AddBook>();

    let book_subscriptions =
        create_resource(move || add_book.version().get(), move |_| get_books());

    let owner_perm = create_resource(
        || (),
        move |_| async move { has_permission("admin".into()).await.unwrap_or(false) },
    );

    view! {
        <div>
            <Suspense fallback=|| ()>
                {move ||
                    owner_perm.get().map(|owner_perm|
                        if owner_perm {
                            view!{
                                <ActionForm action=add_book>
                                    <label>
                                    "Add Book"
                                    <input type="text" name="name"/>
                                    </label>
                                    <input type="submit" value="Create"/>
                                </ActionForm>
                            }
                        } else {
                            ().into_view()
                        }
                    )
                }
            </Suspense>
            {
                move ||
                if add_book.pending().get() {
                    view!{<p>"Loading"</p>}.into_view()
                } else {().into_view()}
            }
            {
                move || if let Some(Ok(new_id)) = add_book.value().get() {
                    view!{<Redirect path=format!("/books/{new_id}")/>}
                } else {
                    ().into_view()
                }
            }
            <div class="flex flex-col items-center justify-center">
            <Transition fallback=move || view! {<p>"Loading..."</p> }>
                {move || {
                        let user_books = {
                            move || {
                                book_subscriptions.get().map(move |book_subscriptions| match book_subscriptions {
                                    Err(e) => {
                                        view! {<pre class="error">"Server Error: " {e.to_string()}</pre>}.into_view()
                                    },
                                    Ok(book_subscriptions) => {
                                        if book_subscriptions.is_empty() {
                                            view! {<p>"No books yet"</p>}.into_view()
                                        } else {
                                            book_subscriptions
                                                .into_iter()
                                                .map(move |book_subscription| view! {
                                                    <li class="p-3 w-60 h-30">
                                                        <a href={format!("/books/{}", book_subscription.book_id)}>
                                                            <div class="content-center justify-center max-w-sm overflow-hidden bg-white rounded-lg shadow-lg">
                                                                <h1>{book_subscription.name}</h1>
                                                                <p>{Into::<String>::into(book_subscription.role)}</p>
                                                            </div>
                                                        </a>
                                                    </li>
                                                }).collect_view()
                                        }
                                    }
                                })
                            }
                        };
                        view! {
                            <ul class="items-center self-center justify-center">
                                {user_books}
                            </ul>
                        }
                    }
                }
            </Transition>
            </div>
        </div>
    }
}
