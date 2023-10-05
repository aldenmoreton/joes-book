use leptos::*;
use leptos_router::{ActionForm, Redirect};

use crate::server::{AddBook, get_books, has_permission};

#[component]
pub fn Books(
    cx: Scope
) -> impl IntoView {

	let add_book = create_server_action::<AddBook>(cx);

	let book_subscriptions = create_resource(cx,
		move || { add_book.version().get() },
		move |_| { get_books(cx) }
	);

	let owner_perm = create_resource(
		cx,
		|| (),
		move |_| async move {
			has_permission(cx, "admin".into()).await.unwrap_or(false)
		}
	);

	view! {
        cx,
        <div>
			<Suspense fallback=|| ()>
				{move ||
					owner_perm.read(cx).map(|owner_perm|
						if owner_perm {
							view!{cx,
								<ActionForm action=add_book>
									<label>
									"Add Book"
									<input type="text" name="name"/>
									</label>
									<input type="submit" value="Create"/>
								</ActionForm>
							}
						} else {
							().into_view(cx)
						}
					)
				}
			</Suspense>
			{
				move ||
				if add_book.pending().get() {
					view!{cx, <p>"Loading"</p>}.into_view(cx)
				} else {().into_view(cx)}
			}
			{
				move || if let Some(Ok(new_id)) = add_book.value().get() {
					view!{cx, <Redirect path=format!("/books/{new_id}")/>}
				} else {
					().into_view(cx)
				}
			}
			<div class="flex flex-col items-center justify-center">
			<Transition fallback=move || view! {cx, <p>"Loading..."</p> }>
				{move || {
						let user_books = {
							move || {
								book_subscriptions.read(cx).map(move |book_subscriptions| match book_subscriptions {
									Err(e) => {
										view! { cx, <pre class="error">"Server Error: " {e.to_string()}</pre>}.into_view(cx)
									},
									Ok(book_subscriptions) => {
										if book_subscriptions.is_empty() {
											view! {cx, <p>"No books yet"</p>}.into_view(cx)
										} else {
											book_subscriptions
												.into_iter()
												.map(move |book_subscription| view! {cx,
													<li class="p-3 w-60 h-30">
														<a href={format!("/books/{}", book_subscription.book_id)}>
															<div class="content-center justify-center max-w-sm overflow-hidden bg-white rounded-lg shadow-lg">
																<h1>{book_subscription.name}</h1>
																<p>{Into::<String>::into(book_subscription.role)}</p>
															</div>
														</a>
													</li>
												}).collect_view(cx)
										}
									}
								})
							}
						};
						view! {cx,
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