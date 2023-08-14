use leptos::*;
use leptos_router::*;

use crate::components::books::{Book, BookRole};

#[server(GetBook, "/secure")]
pub async fn get_book(cx: Scope, id: i64) -> Result<Book, ServerFnError> {
	use crate::components::pool;
	use crate::auth::auth;

	let user = auth(cx)?.current_user.unwrap();
	let pool = pool(cx)?;

    let result = sqlx::query_as::<_, Book>(
		r#"	SELECT b.id, b.name, s.role
			FROM books AS b
			INNER JOIN subscriptions AS s ON s.book_id=b.id
			WHERE s.user_id = $1 AND b.id = $2
		"#
	)
        .bind(user.id)
		.bind(id)
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
		Book { role: BookRole::Owner, .. } => (),
		_ => return Err(ServerFnError::Request("You can't just delete someone elses book! Rude!!!".into()))
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
				Some(Book{role: BookRole::Admin, ..}) |
				Some(Book{role: BookRole::Owner, ..}) => AdminView(cx, AdminViewProps{id: book_id}).into_view(cx),
				_ => ().into_view(cx)
			}}
		</Suspense>

		<p>"Common knowlege"</p>
	}
}

#[component]
pub fn AdminView(cx: Scope, id: i64) -> impl IntoView {
	let delete_book = create_server_action::<DeleteBook>(cx);

	view! {cx,
		<ActionForm action=delete_book>
			<input type="hidden" name="id" value={id}/>
			<input type="submit" value="Delete Book"/>
		</ActionForm>
	}
}