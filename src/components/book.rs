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
		DeleteBook
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

	let book_id:i64 = params.with(|params| params.get("id").cloned()).unwrap().parse::<i64>().unwrap();

	let book = create_resource(
		cx,
		|| (),
		move |_| async move {
			get_book(cx, book_id).await.unwrap()
		}
	);

	view!{cx,
		<p>"Common knowlege"</p>
		<Suspense fallback=|| "Loading">
			{move || match book.read(cx) {
				Some(BookSubscription{role: BookRole::Admin, ..}) |
				Some(BookSubscription{role: BookRole::Owner, ..}) => AdminView(cx, AdminViewProps{book_id}).into_view(cx),
				_ => ().into_view(cx)
			}}
		</Suspense>
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
			let user: Option<FrontendUser> = user.get();
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
