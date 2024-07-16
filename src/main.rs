use std::{
    fmt::Debug,
    sync::{Arc, RwLock},
};

use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::get,
    Form, Router,
};
use axum_htmx::{AutoVaryLayer, HxRequest};
use serde::Deserialize;
use thiserror::Error;
use tower_http::services::ServeDir;
use tower_livereload::LiveReloadLayer;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let state = SharedState::default();
    let app = Router::new()
        .route("/todos", get(todos).post(create_todo))
        .route(
            "/todos/:id",
            get(todos)
                .post(edit_todo)
                .put(edit_todo)
                .delete(delete_todo),
        )
        .route("/todos/:id/delete", get(delete_todo))
        .fallback(|| async { Redirect::permanent("/todos") })
        .layer(AutoVaryLayer)
        .nest_service("/assets", ServeDir::new("assets"))
        .layer(LiveReloadLayer::new())
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

type SharedState = Arc<RwLock<AppState>>;

#[derive(Default)]
struct AppState {
    todos: Vec<String>,
}

async fn todos(
    HxRequest(hx): HxRequest,
    id: Option<Path<usize>>,
    State(state): State<SharedState>,
) -> Result<impl IntoResponse, Error> {
    let id = id.map(|id| id.0);
    let todos = &state.read().or(Err(Error::SharedState))?.todos;
    let response = if hx {
        TodoListFragment {
            todos: &todos[..],
            active: id,
        }
        .into_response()
    } else {
        TodosTemplate {
            todos: &todos[..],
            active: id,
        }
        .into_response()
    };
    Ok(response)
}

async fn delete_todo(
    HxRequest(hx): HxRequest,
    Path(id): Path<usize>,
    State(state): State<SharedState>,
) -> Result<impl IntoResponse, Error> {
    let mut state = state.write().map_err(|_| Error::Poison)?;
    if state.todos.get(id).is_some() {
        state.todos.remove(id);
    } else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }
    let response = if hx {
        TodoListFragment {
            todos: &state.todos[..],
            active: None,
        }
        .into_response()
    } else {
        Redirect::to("/todos").into_response()
    };
    Ok(response)
}

async fn create_todo(
    HxRequest(hx): HxRequest,
    State(state): State<SharedState>,
    Form(form): Form<TodoForm>,
) -> Result<impl IntoResponse, Error> {
    let mut state = state.write().map_err(|_| Error::Poison)?;
    state.todos.push(form.todo);
    let response = if hx {
        TodoListFragment {
            todos: &state.todos[..],
            active: None,
        }
        .into_response()
    } else {
        Redirect::to("/todos").into_response()
    };
    Ok(response)
}

async fn edit_todo(
    Path(id): Path<usize>,
    State(state): State<SharedState>,
    Form(form): Form<TodoForm>,
) -> Result<impl IntoResponse, Error> {
    let mut state = state.write().map_err(|_| Error::Poison)?;
    match state.todos.get_mut(id) {
        Some(existing) => *existing = form.todo,
        None => state.todos.push(form.todo),
    }
    Ok(Redirect::to("/todos"))
}

#[derive(Deserialize, Debug)]
struct TodoForm {
    todo: String,
}

// Templates

// Pages

#[derive(Template)]
#[template(path = "todos.html")]
struct TodosTemplate<'a> {
    todos: &'a [String],
    active: Option<usize>,
}

// Fragments

#[derive(Template)]
#[template(path = "todos.html", block = "list")]
struct TodoListFragment<'a> {
    todos: &'a [String],
    active: Option<usize>,
}

#[derive(Error, Debug)]
enum Error {
    #[error("an error occurred while accessing shared state")]
    SharedState,
    #[error("shared state lock was poisoned")]
    Poison,
    #[error(transparent)]
    Askama(#[from] askama::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "oops something went wrong",
        )
            .into_response()
    }
}
