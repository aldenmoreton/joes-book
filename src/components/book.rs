use leptos::*;
use leptos_router::*;

use crate::{components::{books::{BookSubscription, BookRole}, admin::UserSelect}, auth::FrontendUser};

#[server(GetBook, "/secure")]
pub async fn get_book(cx: Scope, book_id: i64) -> Result<BookSubscription, ServerFnError> {
	use crate::components::pool;
	use crate::auth::auth;

	let user = auth(cx)?.current_user.unwrap();
	let pool = pool(cx)?;

    let result = sqlx::query_as::<_, BookSubscription>(
		r#"	SELECT b.id, b.name, s.role, s.user_id
			FROM books AS b
			INNER JOIN subscriptions AS s ON s.book_id=b.id
			WHERE s.user_id = $1 AND b.id = $2
		"#
	)
        .bind(user.id)
		.bind(book_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

	Ok(result)
}

#[server(DeleteBook, "/secure")]
pub async fn delete_book(cx: Scope, id: i64) -> Result<(), ServerFnError> {
	use crate::components::pool;
	use crate::auth::auth;
	use crate::components::books::BookRole;

	let user = auth(cx)?.current_user.unwrap();
	let pool = pool(cx)?;

	match get_book(cx, id).await? {
		BookSubscription { role: BookRole::Owner, .. } => (),
		_ => return Err(ServerFnError::Request("You can't just delete someone else's book! Rude!!!".into()))
	}

	let deleted_book = sqlx::query!(
		r#"	DELETE FROM books
			WHERE id IN (
				SELECT s.book_id
				FROM subscriptions AS s
				WHERE s.user_id = $1 AND s.book_id = $2 AND s.role = $3
			)
			RETURNING id"#,
			user.id,
			id,
			Into::<String>::into(BookRole::Owner)
	)
		.fetch_one(&pool)
		.await
		.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

	log!("{:?}", deleted_book.id);

	sqlx::query!(
		r#"	DELETE FROM subscriptions
			WHERE book_id IN (
				SELECT s.book_id
				FROM subscriptions AS s
				WHERE s.book_id = $1
			)
		"#,
		id
	)
		.execute(&pool)
		.await?;

	leptos_axum::redirect(cx, "/books");

	Ok(())
}

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
		<Suspense fallback=|| "Loading">
			{move || match book.read(cx) {
				Some(BookSubscription{role: BookRole::Admin, ..}) |
				Some(BookSubscription{role: BookRole::Owner, ..}) => AdminView(cx, AdminViewProps{book_id}).into_view(cx),
				_ => ().into_view(cx)
			}}
		</Suspense>

		<p>"Common knowlege"</p>
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
		<ActionForm action=delete_book>
			<input type="hidden" name="id" value={book_id}/>
			<input type="submit" value="Delete Book"/>
		</ActionForm>
		<UserSelect user_selector/>
		<Suspense fallback=move || view! {cx, <p>"Loading..."</p> }>
			{move ||
				{
					match user_subscription.read(cx){
						Some(Ok(user_account)) => view!{cx, <UserOptions user=user.get().unwrap() user_subscription=user_account user_selector/> }.into_view(cx),
						_ => { ().into_view(cx) },
					}
				}
			}
		</Suspense>
	}
}

#[component]
pub fn UserOptions(cx: Scope, user: FrontendUser, user_subscription: BookSubscription, user_selector: WriteSignal<Option<FrontendUser>>) -> impl IntoView {
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

#[server(AddUser, "/secure")]
pub async fn add_user(cx: Scope, user_id: i64, book_id: i64) -> Result<bool, ServerFnError> {
	use crate::components::pool;

	let owner = get_book(cx, book_id).await?;
	match owner {
		BookSubscription{role: BookRole::Owner, ..} => (),
		_ => return Err(ServerFnError::Request("I love your enthusiasm, but you can't add people to a book you don't own".into()))
	}

	let pool = pool(cx)?;

	let result = sqlx::query(
		r#"SELECT user_id
			FROM subscriptions
			WHERE user_id=$1 AND book_id=$2"#
	)
		.bind(user_id)
		.bind(book_id)
		.fetch_optional(&pool)
		.await?;

	if result.is_some() { return Ok(false) }

	sqlx::query(
		r#"	INSERT INTO subscriptions (user_id, book_id, role)
			VALUES ($1, $2, $3)"#
	)
		.bind(user_id)
		.bind(book_id)
		.bind(Into::<String>::into(BookRole::Participant))
		.execute(&pool)
		.await?;

	Ok(true)
}

#[server(RemoveUser, "/secure")]
pub async fn remove_user(cx: Scope, user_id: i64, book_id: i64) -> Result<bool, ServerFnError> {
	use crate::components::pool;

	let owner = get_book(cx, book_id).await?;
	match owner {
		BookSubscription{role: BookRole::Owner, ..} => (),
		_ => return Err(ServerFnError::Request("You can't remove people from a book that isn't yours... that's antisocial".into()))
	}

	let pool = pool(cx)?;

	let result = sqlx::query(
		r#"SELECT user_id
			FROM subscriptions
			WHERE user_id=$1 AND book_id=$2"#
	)
		.bind(user_id)
		.bind(book_id)
		.fetch_optional(&pool)
		.await?;

	if result.is_none() { return Ok(false) }

	sqlx::query(
		r#"	DELETE FROM subscriptions
			WHERE user_id = $1 AND book_id = $2"#
	)
		.bind(user_id)
		.bind(book_id)
		.bind(Into::<String>::into(BookRole::Participant))
		.execute(&pool)
		.await?;

	Ok(true)
}

#[server(PromoteAdmin, "/secure")]
pub async fn promote_admin(cx: Scope, user_id: i64, book_id: i64) -> Result<bool, ServerFnError> {
	use crate::components::pool;

	let owner = get_book(cx, book_id).await?;
	match owner {
		BookSubscription{role: BookRole::Owner, ..} => (),
		_ => return Err(ServerFnError::Request("That's flattering, but you can't promte people in a book that isn't yours.".into()))
	}

	let subscription = get_subsciption(cx, user_id, book_id).await?;
	match subscription.role {
		BookRole::Admin => return Ok(false),
		BookRole::Owner => return Err(ServerFnError::Request("You are already kind of an admin (you're the owner)".into())),
		BookRole::Unauthorized => { add_user(cx, user_id, book_id).await?; },
		BookRole::Participant => ()
	}

	let pool = pool(cx)?;

	sqlx::query(
		r#"	UPDATE subscriptions
			SET role =  $1
			WHERE user_id = $2 AND book_id = $3"#
	)
		.bind(Into::<String>::into(BookRole::Admin))
		.bind(user_id)
		.bind(book_id)
		.execute(&pool)
		.await?;

	Ok(true)
}

#[server(DemoteAdmin, "/secure")]
pub async fn demote_admin(cx: Scope, user_id: i64, book_id: i64) -> Result<bool, ServerFnError> {
	use crate::components::pool;

	let owner = get_book(cx, book_id).await?;
	match owner {
		BookSubscription{role: BookRole::Owner, ..} => (),
		_ => return Err(ServerFnError::Request("You can't demote people in a book that isn't yours... that's antisocial".into()))
	}

	let pool = pool(cx)?;

	sqlx::query(
		r#"	UPDATE subscriptions
			SET role =  $1
			WHERE user_id = $2 AND book_id = $3"#
	)
		.bind(Into::<String>::into(BookRole::Participant))
		.bind(user_id)
		.bind(book_id)
		.execute(&pool)
		.await?;

	Ok(true)
}

#[server(GetSubscription, "/secure")]
pub async fn get_subsciption(cx: Scope, user_id: i64, book_id: i64) -> Result<BookSubscription, ServerFnError> {
	use crate::components::pool;

	let pool = pool(cx)?;

	let admin_book = get_book(cx, book_id).await?;
	match admin_book {
		BookSubscription{role: BookRole::Owner, ..} => (),
		_ => return Err(ServerFnError::Request("I can't just give out info willy nilly".into()))
	};

	let result = sqlx::query_as::<_, BookSubscription>(
		r#"	SELECT b.id, b.name, s.role, s.user_id
				FROM books AS b
				INNER JOIN subscriptions AS s ON s.book_id=b.id
				WHERE s.user_id = $1 AND b.id = $2
			"#
	)
		.bind(user_id)
		.bind(book_id)
		.fetch_optional(&pool)
		.await?
		.unwrap_or(
			BookSubscription {
				book_id,
				user_id,
				name: admin_book.name,
				role: BookRole::Unauthorized
			}
		);



	Ok(result)
}
