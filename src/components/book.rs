use leptos::*;
use leptos_router::*;

use crate::components::books::Book;

#[server(GetBook, "/secure")]
pub async fn get_book(cx: Scope, id: i64) -> Result<Book, ServerFnError> {
	use crate::components::{auth, pool};

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
	use crate::components::{auth, pool};
	use crate::components::books::BookRoles;

	let user = auth(cx)?.current_user.unwrap();
	let pool = pool(cx)?;

	sqlx::query(
		r#"	DELETE FROM books
			WHERE id IN (
				SELECT book_id
				FROM subscriptions
				WHERE user_id = $1 AND book_id = $3 AND role = $2
			)"#
	)
		.bind(user.id)
		.bind(id)
		.bind(Into::<String>::into(BookRoles::Owner))
		.execute(&pool)
		.await?;

	Ok(())
}

#[component]
pub fn Book(
    cx: Scope
) -> impl IntoView {
	let params = use_params_map(cx);

	let delete_book = create_server_action::<DeleteBook>(cx);

	let id:i64 = params.with(|params| params.get("id").cloned()).unwrap().parse::<i64>().unwrap();

	view!{cx,
		// <Transition fallback=move || view! {cx, <p>"Loading..."</p> }>
		// 	{move ||
		// 		book =
		// 	}
		// </Transition>
		// {book.read(cx).unwrap().unwrap().name}
		// <p>{id}</p>

		<ActionForm action=delete_book>
			<input type="hidden" name="id" value={id}/>
			<input type="submit" value="Delete Book"/>
		</ActionForm>
	}

}