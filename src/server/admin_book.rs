use cfg_if::cfg_if;
use leptos::*;

use crate::objects::BookSubscription;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use crate::server::{
            auth::auth,
            pool,
            get_book
        };
        use crate::objects::BookRole;
    }
}

#[server(DeleteBook, "/secure", "Url", "delete_book")]
pub async fn delete_book(id: i64) -> Result<(), ServerFnError> {
    let user = auth()?.current_user.unwrap();
    let pool = pool()?;

    match get_book(id).await? {
        BookSubscription {
            role: BookRole::Owner,
            ..
        } => (),
        _ => {
            return Err(ServerFnError::Request(
                "You can't just delete someone else's book! Rude!!!".into(),
            ))
        }
    }

    let _deleted_book = sqlx::query!(
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
    .map_err(|e| ServerFnError::new(e.to_string()))?;

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

    leptos_axum::redirect("/books");

    Ok(())
}

#[server(AddUser, "/secure", "Url", "add_user")]
pub async fn add_user(user_id: i64, book_id: i64) -> Result<bool, ServerFnError> {
    let owner = get_book(book_id).await?;
    match owner {
        BookSubscription {
            role: BookRole::Owner,
            ..
        } => (),
        _ => {
            return Err(ServerFnError::Request(
                "I love your enthusiasm, but you can't add people to a book you don't own".into(),
            ))
        }
    }

    let pool = pool()?;

    let result = sqlx::query(
        r#"SELECT user_id
			FROM subscriptions
			WHERE user_id=$1 AND book_id=$2"#,
    )
    .bind(user_id)
    .bind(book_id)
    .fetch_optional(&pool)
    .await?;

    if result.is_some() {
        return Ok(false);
    }

    sqlx::query(
        r#"	INSERT INTO subscriptions (user_id, book_id, role)
			VALUES ($1, $2, $3)"#,
    )
    .bind(user_id)
    .bind(book_id)
    .bind(Into::<String>::into(BookRole::Participant))
    .execute(&pool)
    .await?;

    Ok(true)
}

#[server(RemoveUser, "/secure", "Url", "remove_user")]
pub async fn remove_user(user_id: i64, book_id: i64) -> Result<bool, ServerFnError> {
    let owner = get_book(book_id).await?;
    match owner {
        BookSubscription {
            role: BookRole::Owner,
            ..
        } => (),
        _ => {
            return Err(ServerFnError::Request(
                "You can't remove people from a book that isn't yours... that's antisocial".into(),
            ))
        }
    }

    let pool = pool()?;

    let result = sqlx::query(
        r#"SELECT user_id
			FROM subscriptions
			WHERE user_id=$1 AND book_id=$2"#,
    )
    .bind(user_id)
    .bind(book_id)
    .fetch_optional(&pool)
    .await?;

    if result.is_none() {
        return Ok(false);
    }

    sqlx::query(
        r#"	DELETE FROM subscriptions
			WHERE user_id = $1 AND book_id = $2"#,
    )
    .bind(user_id)
    .bind(book_id)
    .bind(Into::<String>::into(BookRole::Participant))
    .execute(&pool)
    .await?;

    Ok(true)
}

#[server(PromoteAdmin, "/secure", "Url", "promote_admin")]
pub async fn promote_admin(user_id: i64, book_id: i64) -> Result<bool, ServerFnError> {
    let owner = get_book(book_id).await?;
    match owner {
        BookSubscription {
            role: BookRole::Owner,
            ..
        } => (),
        _ => {
            return Err(ServerFnError::Request(
                "That's flattering, but you can't promte people in a book that isn't yours.".into(),
            ))
        }
    }

    let subscription = get_subsciption(user_id, book_id).await?;
    match subscription.role {
        BookRole::Admin => return Ok(false),
        BookRole::Owner => {
            return Err(ServerFnError::Request(
                "You are already kind of an admin (you're the owner)".into(),
            ))
        }
        BookRole::Unauthorized => {
            add_user(user_id, book_id).await?;
        }
        BookRole::Participant => (),
    }

    let pool = pool()?;

    sqlx::query(
        r#"	UPDATE subscriptions
			SET role =  $1
			WHERE user_id = $2 AND book_id = $3"#,
    )
    .bind(Into::<String>::into(BookRole::Admin))
    .bind(user_id)
    .bind(book_id)
    .execute(&pool)
    .await?;

    Ok(true)
}

#[server(DemoteAdmin, "/secure", "Url", "demote_admin")]
pub async fn demote_admin(user_id: i64, book_id: i64) -> Result<bool, ServerFnError> {
    let owner = get_book(book_id).await?;
    match owner {
        BookSubscription {
            role: BookRole::Owner,
            ..
        } => (),
        _ => {
            return Err(ServerFnError::Request(
                "You can't demote people in a book that isn't yours... that's antisocial".into(),
            ))
        }
    }

    let pool = pool()?;

    sqlx::query(
        r#"	UPDATE subscriptions
			SET role =  $1
			WHERE user_id = $2 AND book_id = $3"#,
    )
    .bind(Into::<String>::into(BookRole::Participant))
    .bind(user_id)
    .bind(book_id)
    .execute(&pool)
    .await?;

    Ok(true)
}

#[server(GetSubscription, "/secure", "Url", "get_subsciption")]
pub async fn get_subsciption(
    user_id: i64,
    book_id: i64,
) -> Result<BookSubscription, ServerFnError> {
    let pool = pool()?;

    let admin_book = get_book(book_id).await?;
    match admin_book {
        BookSubscription {
            role: BookRole::Owner,
            ..
        } => (),
        _ => {
            return Err(ServerFnError::Request(
                "I can't just give out info willy nilly".into(),
            ))
        }
    };

    let result = sqlx::query_as::<_, BookSubscription>(
        r#"	SELECT b.id, b.name, s.role, s.user_id
				FROM books AS b
				INNER JOIN subscriptions AS s ON s.book_id=b.id
				WHERE s.user_id = $1 AND b.id = $2
			"#,
    )
    .bind(user_id)
    .bind(book_id)
    .fetch_optional(&pool)
    .await?
    .unwrap_or(BookSubscription {
        book_id,
        user_id,
        name: admin_book.name,
        role: BookRole::Unauthorized,
    });

    Ok(result)
}
