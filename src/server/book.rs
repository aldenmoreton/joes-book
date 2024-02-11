use leptos::*;

use crate::objects::BookSubscription;

use cfg_if::cfg_if;
cfg_if! {
    if #[cfg(feature = "ssr")] {
        use crate::{
            server::{
                auth,
                pool,
                has_permission
            },
            objects::BookRole
        };
    }
}

#[server(GetBook, "/secure", "Url", "get_book")]
pub async fn get_book(book_id: i64) -> Result<BookSubscription, ServerFnError> {
    let user = auth()?.current_user.unwrap();
    let pool = pool()?;

    let result = sqlx::query_as::<_, BookSubscription>(
        r#"	SELECT b.id, b.name, s.role, s.user_id
			FROM books AS b
			INNER JOIN subscriptions AS s ON s.book_id=b.id
			WHERE s.user_id = $1 AND b.id = $2
		"#,
    )
    .bind(user.id)
    .bind(book_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(result)
}

#[server(GetBooks, "/secure", "Url", "get_books")]
pub async fn get_books() -> Result<Vec<BookSubscription>, ServerFnError> {
    let user = auth()?.current_user.unwrap();
    let pool = pool()?;

    let result = sqlx::query_as::<_, BookSubscription>(
        r#"	SELECT b.id, b.name, s.role, s.user_id
			FROM books AS b
			INNER JOIN subscriptions AS s ON s.book_id=b.id
			WHERE s.user_id = $1
		"#,
    )
    .bind(user.id)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(result)
}

#[server(AddBook, "/secure", "Url", "add_book")]
pub async fn add_book(name: String) -> Result<i64, ServerFnError> {
    if !has_permission("admin".into()).await? {
        return Err(ServerFnError::Request(
            "Not permitted to create books".into(),
        ));
    }

    let user = auth()?.current_user.unwrap();
    let pool = pool()?;

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

#[server(GetBookTable, "/secure", "Url", "get_book_table")]
pub async fn get_book_table(book_id: i64) -> Result<String, ServerFnError> {
    let book_subscription = get_book(book_id).await?;
    match book_subscription.role {
        BookRole::Unauthorized => {
            return Err(ServerFnError::Request("You aren't in this book".into()))
        }
        _ => (),
    }

    let pool = pool()?;
    let user_points: Vec<(_, _, _)> = sqlx::query!(
        r#"
		SELECT u.id AS id, u.username AS username, CAST(COALESCE(p.total, 0) AS INTEGER) AS book_total
		FROM (
			SELECT users.id, users.username
			FROM chapters
			INNER JOIN subscriptions ON subscriptions.book_id = chapters.book_id
			INNER JOIN users ON users.id = subscriptions.user_id
			WHERE chapters.book_id = $1
			GROUP BY users.id, users.username
		) AS u
		LEFT JOIN (
			SELECT user_id, SUM(picks.wager) AS total
			FROM picks
			WHERE picks.book_id = $1 AND picks.correct
			GROUP BY user_id
		) AS p
		ON u.id = p.user_id
		ORDER BY book_total DESC, username"#,
        book_id
    )
    .fetch_all(&pool)
    .await?
    .into_iter()
    .map(|row| (row.id, row.username, row.book_total.unwrap_or(0)))
    .collect();

    Ok(view! {
        <table class="bg-white rounded-md">
            <tr>
                <th>"Rank"</th>
                <th>"Member"</th>
                <th>"Points"</th>
            </tr>
            {
                user_points
                    .into_iter()
                    .enumerate()
                    .map(|(i, (_, username, total))| view!{
                        <tr>
                            <td><p>{i + 1}</p></td>
                            <td><p>{username}</p></td>
                            <td><p>{total}</p></td>
                        </tr>
                    })
                    .collect_view()
            }
        </table>
    }
    .into_view()
    .render_to_string()
    .to_string())
}
