use leptos::*;
use leptos_router::*;

use crate::{
	components::admin::UserSelect,
	server::{
		get_book,
		get_subsciption,
		AddUser,
		RemoveUser,
		PromoteAdmin,
		DemoteAdmin,
		DeleteBook,
		get_chapters
	},
	objects::{
		BookRole,
		BookSubscription,
		FrontendUser
	}
};

#[component]
pub fn Book(
    cx: Scope
) -> impl IntoView {
	let params = use_params_map(cx);
	let book_id:i64 = params.with_untracked(|params| params.get("book_id").cloned()).unwrap().parse::<i64>().unwrap();

	let book = create_resource(
		cx,
		|| (),
		move |_| async move {
			get_book(cx, book_id).await
		}
	);

	view!{cx,
		<Suspense fallback=|| "Loading">
			{move || match book.read(cx) {
				Some(Ok(BookSubscription{role: BookRole::Admin, ..})) |
				Some(Ok(BookSubscription{role: BookRole::Owner, ..})) => view!{cx,
					<VerifiedView/>
					<AdminView book_id/>
				}.into_view(cx),
				Some(Ok(BookSubscription{role: BookRole::Participant, ..})) => view!{cx,
					<VerifiedView/>
				}.into_view(cx),
				Some(Ok(BookSubscription{role: BookRole::Unauthorized, ..})) |
				Some(Err(_)) => view!{cx,
					<Redirect path="/books"/>
				}.into_view(cx),
				None => ().into_view(cx)
				// _ => Redirect(cx, RedirectProps{path: "/books", options: None}).into_view(cx)
			}}
		</Suspense>
	}
}

#[component]
pub fn VerifiedView(cx: Scope) -> impl IntoView {
	let params = use_params_map(cx);
	let book_id:i64 = params.with_untracked(|params| params.get("book_id").cloned()).unwrap().parse::<i64>().unwrap();

	let chapters = create_resource(cx, || (),
		move |_| get_chapters(cx, book_id)
	);

	view!{cx,
		<Await future=move |cx| get_book(cx, book_id) bind:book_data>
			{
				match book_data {
					Ok(book) => {
						let book_name = book.name.clone();
						view!{cx, <h1>{book_name}</h1>}.into_view(cx)
					},
					_ => ().into_view(cx)
				}
			}
		</Await>
		<div class="flex flex-col items-center justify-center border border-green-500">
		<Transition fallback=|| "Loading...">
			<ul class="border border-black items-center self-center justify-center">
			{move ||
				{
					move || {
						chapters.read(cx).map(move |chapters| match chapters {
							Err(e) => {
								view! { cx, <pre class="error">"Server Error: " {e.to_string()}</pre>}.into_view(cx)
							},
							Ok(chapters) => {
								if chapters.is_empty() {
									view! {cx, <p>"No chapters yet"</p>}.into_view(cx)
								} else {
									chapters
										.into_iter()
										.map(move |chapter| {
											let gods_time: String = {
												let utc = chrono::DateTime::parse_from_rfc3339(&chapter.closing_time).unwrap();
												let local = utc + chrono::Duration::hours(-6);
												format!("{}", local.format("%B %d, %Y %H:%M%p"))
											};
											view!{cx,
												<li class="p-3 h-30 w-60">
													<a href=format!("/books/{book_id}/chapters/{}", chapter.chapter_id)>
														<div class="max-w-sm rounded-lg overflow-hidden shadow-lg justify-center content-center bg-white">
														// <p>{format!("{:?}", chapter)}</p>
															<p>{chapter.title}</p>
															<p>"Deadline:"<br/>{gods_time}</p>
														</div>
													</a>
												</li>
											}
										})
										.collect_view(cx)
								}
							}
						})
					}
				}
			}
			</ul>
		</Transition>
		</div>
	}
}

#[component]
pub fn AdminView(cx: Scope, book_id: i64) -> impl IntoView {
	let (user, user_selector) = create_signal(cx, None);

	let delete_book = create_server_action::<DeleteBook>(cx);
	let user_subscription = create_resource(
		cx,
		move || user.get(),
		move |_| async move {
			let user: Option<FrontendUser> = user.get_untracked();
			match user {
				Some(user) => get_subsciption(cx, user.id, book_id).await,
				_ => Err(ServerFnError::Request("No user".into()))
			}
		}
	);

	view! {cx,
		<div class="border">
			<h2>"Book owner options"</h2>
			<A href="new"><button>"Add Pick Event"</button></A>
			<ActionForm action=delete_book>
				<input type="hidden" name="id" value={book_id}/>
				<input type="submit" value="Delete Book"/>
			</ActionForm>
			<UserSelect user_selector/>
			<Suspense fallback=move || view! {cx, <p>"Loading..."</p> }>
				{move ||
					{
						match user_subscription.read(cx){
							Some(Ok(user_account)) => view!{cx, <UserOptions user=user.get().unwrap() user_subscription=user_account _user_selector=user_selector/> }.into_view(cx),
							_ => { ().into_view(cx) },
						}
					}
				}
			</Suspense>
		</div>
	}
}

#[component]
pub fn UserOptions(cx: Scope, user: FrontendUser, user_subscription: BookSubscription, _user_selector: WriteSignal<Option<FrontendUser>>) -> impl IntoView {
	let add_user = create_server_action::<AddUser>(cx);
	let remove_user = create_server_action::<RemoveUser>(cx);

	let promote_admin = create_server_action::<PromoteAdmin>(cx);
	let demote_admin = create_server_action::<DemoteAdmin>(cx);

	let user_options = match user_subscription.role {
		BookRole::Unauthorized => view!{cx,
			<ActionForm action=add_user>
				<input type="hidden" name="user_id" value={user.id}/>
				<input type="hidden" name="book_id" value={user_subscription.book_id}/>
				<input type="submit" value={format!("Add {} to {}", user.username, user_subscription.name)}/>
			</ActionForm>
		},
		BookRole::Participant => {
			let promoter = user.clone();
			let promote_sub = user_subscription.clone();
			view! {cx,
				<ActionForm action=remove_user>
					<input type="hidden" name="user_id" value={user.id}/>
					<input type="hidden" name="book_id" value={user_subscription.book_id}/>
					<input type="submit" value={format!("Remove {} from {}", user.username, user_subscription.name)}/>
				</ActionForm>
				<ActionForm action=promote_admin>
					<input type="hidden" name="user_id" value={promoter.id}/>
					<input type="hidden" name="book_id" value={promote_sub.book_id}/>
					<input type="submit" value={format!("Promote {} to Admin for {}", promoter.username, promote_sub.name)}/>
				</ActionForm>
			}.into_view(cx)
		},
		BookRole::Owner => view! {cx,
			<p>"Welcome home! (this is your book)"</p>
		}.into_view(cx),
		BookRole::Admin => view!{cx,
			<ActionForm action=demote_admin>
				<input type="hidden" name="user_id" value={user.id}/>
				<input type="hidden" name="book_id" value={user_subscription.book_id}/>
				<input type="submit" value={format!("Demote {} to Participant for {}", user.username, user_subscription.name)}/>
			</ActionForm>
		}
	};

	view! {cx,
		<>
			{user_options}
		</>
		// {
		// 	move || {
		// 		add_user.version().get();
		// 		remove_user.version().get();

		// 		user_selector.set(None);
		// 	}
		// }
	}
}
