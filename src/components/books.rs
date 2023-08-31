use leptos::*;
use leptos_router::{ActionForm, Redirect};

use crate::server::{AddBook, get_books};

#[component]
pub fn Books(
    cx: Scope
) -> impl IntoView {

	let add_book = create_server_action::<AddBook>(cx);

	let book_subscriptions = create_resource(cx,
		move || { add_book.version().get() },
		move |_| { get_books(cx) }
	);

	view! {
        cx,
        <div>
			<ActionForm action=add_book>
				<label>
					"Add Book"
					<input type="text" name="name"/>
				</label>
				<input type="submit" value="Create"/>
			</ActionForm>
			{
				move ||
				if add_book.pending().get() {
					view!{cx, <p>"Loading"</p>}.into_view(cx)
				} else {().into_view(cx)}
			}
			{
				move || if let Some(Ok(new_id)) = add_book.value().get() {
					view!{cx, <Redirect path=new_id.to_string()/>}
				} else {
					().into_view(cx)
				}
			}
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
													<li>
														<a href={format!("/books/{}", book_subscription.book_id)}>{book_subscription.name}</a>
														<p>{Into::<String>::into(book_subscription.role)}</p>
													</li>
												}).collect_view(cx)
										}
									}
								})
							}
						};
						view! {cx,
							<ul>
								{user_books}
							</ul>
						}
					}
				}
			</Transition>
        </div>
    }
}