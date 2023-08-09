use leptos::*;
use leptos_router::{MultiActionForm, ActionForm};
use serde::{Serialize, Deserialize};
use cfg_if::cfg_if;

use crate::auth::User;
use crate::components::TeamSelect;
use crate::components::team_search::Team;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Todo {
    id: i64,
    user: Option<User>,
    title: String,
    created_at: String,
    completed: bool,
}

cfg_if! {
	if #[cfg(feature = "ssr")] {
        use sqlx::PgPool;
		use crate::components::pool;
        use crate::auth::get_user;
        use sqlx::types::chrono::NaiveDateTime;

		#[derive(sqlx::FromRow, Clone)]
		pub struct SqlTodo {
			id: i64,
			user_id: i64,
			title: String,
			created_at: NaiveDateTime,
			completed: bool,
		}

		impl SqlTodo {
			pub async fn into_todo(self, pool: &PgPool) -> Todo {
				Todo {
					id: self.id,
					user: User::get(self.user_id, pool).await,
					title: self.title,
					created_at: self.created_at.to_string(),
					completed: self.completed,
				}
			}
		}
	}
}

#[server(GetTodos, "/secure")]
pub async fn get_todos(cx: Scope) -> Result<Vec<Todo>, ServerFnError> {
    use futures::TryStreamExt;

    let pool = pool(cx)?;

    let mut todos = Vec::new();
    let mut rows =
        sqlx::query_as::<_, SqlTodo>("SELECT * FROM todos").fetch(&pool);

    while let Some(row) = rows
        .try_next()
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?
    {
        todos.push(row);
    }

    // why can't we just have async closures?
    // let mut rows: Vec<Todo> = rows.iter().map(|t| async { t }).collect();

    let mut converted_todos = Vec::with_capacity(todos.len());

    for t in todos {
        let todo = t.into_todo(&pool).await;
        converted_todos.push(todo);
    }

    let todos: Vec<Todo> = converted_todos;

    Ok(todos)
}

#[server(AddTodo, "/secure")]
pub async fn add_todo(cx: Scope, title: String) -> Result<(), ServerFnError> {
    let user = get_user(cx).await?;
    let pool = pool(cx)?;

    let id = match user {
        Some(user) => user.id,
        None => -1,
    };

    // fake API delay
    // std::thread::sleep(std::time::Duration::from_millis(1250));

    match sqlx::query(
        "INSERT INTO todos (title, user_id, completed) VALUES ($1, $2, false)",
    )
    .bind(title)
    .bind(id)
    .execute(&pool)
    .await
    {
        Ok(_row) => Ok(()),
        Err(e) => Err(ServerFnError::ServerError(e.to_string())),
    }
}

#[server(DeleteTodo, "/secure")]
pub async fn delete_todo(cx: Scope, id: i64) -> Result<(), ServerFnError> {
    let pool = pool(cx)?;

    sqlx::query("DELETE FROM todos WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map(|_| ())
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

#[component]
pub fn Todos(cx: Scope) -> impl IntoView {
    let add_todo = create_server_multi_action::<AddTodo>(cx);
    let delete_todo = create_server_action::<DeleteTodo>(cx);
    let submissions = add_todo.submissions();

    // list of todos is loaded from the server in reaction to changes
    let todos = create_resource(
        cx,
        move || (add_todo.version().get(), delete_todo.version().get()),
        move |_| get_todos(cx),
    );

    let (selected_team, team_selector) = create_signal::<Option<Team>>(cx, None);

    view! {
        cx,
        <TeamSelect team_selector/>
        {move || match selected_team.get() {
            Some(team) => format!("Selected team is {}", team.name),
            None => "".into()
        }}
        <div>
            <MultiActionForm action=add_todo>
                <label>
                    "Add a Todo"
                    <input type="text" name="title"/>
                </label>
                <input type="submit" value="Add"/>
            </MultiActionForm>
            <Transition fallback=move || view! {cx, <p>"Loading..."</p> }>
                {move || {
                    let existing_todos = {
                        move || {
                            todos.read(cx)
                                .map(move |todos| match todos {
                                    Err(e) => {
                                        view! { cx, <pre class="error">"Server Error: " {e.to_string()}</pre>}.into_view(cx)
                                    }
                                    Ok(todos) => {
                                        if todos.is_empty() {
                                            view! { cx, <p>"No tasks were found."</p> }.into_view(cx)
                                        } else {
                                            todos
                                                .into_iter()
                                                .map(move |todo| {
                                                    view! {
                                                        cx,
                                                        <li>
                                                            {todo.title}
                                                            ": Created at "
                                                            {todo.created_at}
                                                            " by "
                                                            {
                                                                todo.user.unwrap_or_default().username
                                                            }
                                                            <ActionForm action=delete_todo>
                                                                <input type="hidden" name="id" value={todo.id}/>
                                                                <input type="submit" value="X"/>
                                                            </ActionForm>
                                                        </li>
                                                    }
                                                })
                                                .collect_view(cx)
                                        }
                                    }
                                })
                                .unwrap_or_default()
                        }
                    };

                    let pending_todos = move || {
                        submissions
                        .get()
                        .into_iter()
                        .filter(|submission| submission.pending().get())
                        .map(|submission| {
                            view! {
                                cx,
                                <li class="pending">{move || submission.input.get().map(|data| data.title) }</li>
                            }
                        })
                        .collect_view(cx)
                    };

                    view! {
                        cx,
                        <ul>
                            {existing_todos}
                            {pending_todos}
                        </ul>
                    }
                }
            }
            </Transition>
        </div>
    }
}
