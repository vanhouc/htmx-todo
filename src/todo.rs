use askama::Template;
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
    routing::get,
    Form, Router,
};
use axum_htmx::HxRequest;
use serde::Deserialize;

use crate::{Error, SharedState};

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/", get(todos).post(create_todo))
        .route(
            "/:id",
            get(todos)
                .post(edit_todo)
                .put(edit_todo)
                .delete(delete_todo),
        )
        .route("/:id/done", get(done_todo))
        .route("/:id/undo", get(undo_todo))
        .route("/:id/delete", get(delete_todo))
}

// handlers

async fn todos(
    HxRequest(hx): HxRequest,
    id: Option<Path<i64>>,
    State(state): State<SharedState>,
) -> Result<impl IntoResponse, crate::Error> {
    // unwrap the path inside the option
    let id = id.map(|id| id.0);
    // fetch todo's from db and convert to viewmodel
    let mut todos: Vec<Todo> = db::list_todo(&state)
        .await?
        .into_iter()
        .map(Todo::from)
        .collect();
    // if the path contains an id set that todo to edit mode
    if let Some(id) = id {
        if let Some(edit_todo) = todos.iter_mut().find(|todo| todo.id == id) {
            edit_todo.edit = true;
        }
    }
    // if this is an hx request only the list needs to render as it is the only dynamic element on the page
    let response = if hx {
        TodoListFragment { todos }.into_response()
    } else {
        TodosPage { todos }.into_response()
    };
    Ok(response)
}

async fn delete_todo(
    Path(id): Path<i64>,
    State(state): State<SharedState>,
) -> Result<impl IntoResponse, Error> {
    db::delete_todo(id, &state).await?;
    Ok(Redirect::to("/todos").into_response())
}

async fn create_todo(
    State(state): State<SharedState>,
    Form(form): Form<NewTodoForm>,
) -> Result<impl IntoResponse, Error> {
    db::create_todo(form.description, &state).await?;
    Ok(Redirect::to("/todos").into_response())
}

async fn edit_todo(
    Path(id): Path<i64>,
    State(state): State<SharedState>,
    Form(form): Form<EditTodoForm>,
) -> Result<impl IntoResponse, Error> {
    db::update_todo(id, form.description, &state).await?;
    Ok(Redirect::to("/todos").into_response())
}

async fn done_todo(
    Path(id): Path<i64>,
    State(state): State<SharedState>,
) -> Result<impl IntoResponse, Error> {
    db::done_todo(id, true, &state).await?;
    Ok(Redirect::to("/todos").into_response())
}

async fn undo_todo(
    Path(id): Path<i64>,
    State(state): State<SharedState>,
) -> Result<impl IntoResponse, Error> {
    db::done_todo(id, false, &state).await?;
    Ok(Redirect::to("/todos").into_response())
}

// pages

#[derive(Template)]
#[template(path = "todos.html")]
struct TodosPage {
    todos: Vec<Todo>,
}

// fragments

#[derive(Template)]
#[template(path = "todos.html", block = "list")]
struct TodoListFragment {
    todos: Vec<Todo>,
}

// forms

#[derive(Deserialize, Debug)]
struct NewTodoForm {
    description: String,
}

#[derive(Deserialize, Debug)]
struct EditTodoForm {
    description: String,
}

// view models

struct Todo {
    id: i64,
    description: String,
    done: bool,
    edit: bool,
}

impl From<db::Todo> for Todo {
    fn from(value: db::Todo) -> Self {
        Todo {
            id: value.id,
            description: value.description,
            done: value.done,
            edit: false,
        }
    }
}

// database entities

mod db {
    use sqlx::PgPool;

    use crate::Error;

    pub struct Todo {
        pub id: i64,
        pub description: String,
        pub done: bool,
    }

    pub async fn create_todo(description: String, pool: &PgPool) -> Result<(), Error> {
        sqlx::query!(
            r#"
        INSERT INTO todos (description)
        VALUES ($1)
        "#,
            description,
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn list_todo(pool: &PgPool) -> Result<Vec<Todo>, Error> {
        let todos = sqlx::query_as!(
            Todo,
            r#"
        SELECT id, description, done
        FROM todos
        ORDER BY id
        "#
        )
        .fetch_all(pool)
        .await?;
        Ok(todos)
    }

    pub async fn update_todo(id: i64, description: String, pool: &PgPool) -> Result<(), Error> {
        sqlx::query!(
            r#"
        Update todos
        SET description = $1
        WHERE id = $2
        "#,
            description,
            id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn done_todo(id: i64, done: bool, pool: &PgPool) -> Result<(), Error> {
        sqlx::query!(
            r#"
        Update todos
        SET done = $1
        WHERE id = $2
        "#,
            done,
            id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn delete_todo(id: i64, pool: &PgPool) -> Result<(), Error> {
        sqlx::query!(
            r#"
            DELETE FROM todos
            WHERE id = $1
            "#,
            id
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}
