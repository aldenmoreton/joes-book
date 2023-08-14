use leptos::*;
use leptos_router::{ActionForm, Redirect};
use serde::{ Serialize, Deserialize };

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BookRole {
	Owner,
	Admin,
	Participant,
	Unauthorized
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Book {
	pub id: i64,
	pub name: String,
	#[cfg_attr(feature = "ssr", sqlx(try_from = "String"))]
	pub role: BookRole
}

impl From<String> for BookRole {
	fn from(value: String) -> Self {
		match value.as_str() {
			"owner" => Self::Owner,
			"admin" => Self::Admin,
			"participant" => Self::Participant,
			_ => Self::Unauthorized
		}
	}
}

impl Into<String> for BookRole {
	fn into(self) -> String {
		match self {
			Self::Owner => "owner",
			Self::Admin => "admin",
			Self::Participant => "participant",
			Self::Unauthorized => "unauthorized"
		}.into()
	}
}

#[server(GetBooks, "/secure")]
pub async fn get_books(cx: Scope) -> Result<Vec<Book>, ServerFnError> {
	use crate::components::pool;
	use crate::auth::auth;

	let user = auth(cx)?.current_user.unwrap();
	let pool = pool(cx)?;

    let result = sqlx::query_as::<_, Book>(
		r#"	SELECT b.id, b.name, s.role
			FROM books AS b
			INNER JOIN subscriptions AS s ON s.book_id=b.id
			WHERE s.user_id = $1
		"#
	)
        .bind(user.id)
        .fetch_all(&pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

	Ok(result)
}

#[server(AddBook, "/secure")]
pub async fn add_book(cx: Scope, name: String) -> Result<i64, ServerFnError> {
	use crate::components::pool;
	use crate::auth::auth;
	use crate::auth::has_permission;

	if !has_permission(cx, "admin".into()).await? { return Err(ServerFnError::Request("Not permitted to create books".into())) }

	let user = auth(cx)?.current_user.unwrap();
	let pool = pool(cx)?;

	let result = sqlx::query!(
		r#"	WITH inserted_book AS (
				INSERT INTO books (name) VALUES ($1) RETURNING id
			)
			INSERT INTO subscriptions (book_id, user_id, role)
			SELECT id, $2, $3 FROM inserted_book
			RETURNING book_id"#,
			name,
			user.id,
			Into::<String>::into(BookRole::Owner)
	)
		.fetch_one(&pool)
		.await?;

	Ok(result.book_id)
}

#[component]
pub fn Books(
    cx: Scope
) -> impl IntoView {

	let add_book = create_server_action::<AddBook>(cx);

	let books = create_resource(cx,
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
								books.read(cx).map(move |books| match books {
									Err(e) => {
										view! { cx, <pre class="error">"Server Error: " {e.to_string()}</pre>}.into_view(cx)
									},
									Ok(books) => {
										if books.is_empty() {
											view! {cx, <p>"No books yet"</p>}.into_view(cx)
										} else {
											books
												.into_iter()
												.map(move |book| view! {cx,
													<li>
														<a href={format!("/books/{}", book.id)}>{book.name}</a>
														<p>{Into::<String>::into(book.role)}</p>
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