use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, routing::get, Form, Json, Router};
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use std::net::SocketAddr;

// ----------------------------------------------------
// Custom error type for better error handling 
// Weird af at first, i did not understand shit

#[derive(Debug)]
pub struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        ).into_response()
    }
}

// specific implementations for the error types we use
impl From<std::env::VarError> for AppError {
    fn from(err: std::env::VarError) -> Self {
        Self(anyhow::Error::from(err))
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        Self(anyhow::Error::from(err))
    }
}

type Result<T> = std::result::Result<T, AppError>;

// ----------------------------------------------------

#[tokio::main]
async fn main() -> Result<()> {
    let dotenv = dotenv::dotenv();
    let url: String = std::env::var("DATABASE_URL")?;
    let pool = SqlitePool::connect(&url).await?; // the ? prevent returning "Error"

    match dotenv {
        Ok(_) => println!(".env successfully loaded"),
        Err(_) => println!("Could not find .env")
    }

    let app: Router = Router::new()
        .route("/", get(list))
        .route("/create", get(create))
        .route("/delete/{id}", get(delete))
        .route("/update", get(update))
        .fallback(index)
        .with_state(pool)
        .layer(CorsLayer::very_permissive());

    let address: SocketAddr = SocketAddr::from(([0, 0, 0, 0], 8000));
    let listener: TcpListener = tokio::net::TcpListener::bind(address).await.unwrap();
    
    // if await.unwrap() fails, it will panic and return Err(e), and the Ok() wrapper will never be reached
    // which will cause that the Result<()> never will be returned to the main function
    // Ok(axum::serve(listener, app).await.unwrap()) 

    // handling serve errors manually
    match axum::serve(listener, app).await {
        Ok(()) => (),
        Err(err) => {
            eprintln!("Server error: {}", err);
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn index() -> String {
    "Hello, world!".to_string()
}

#[derive(Serialize, Deserialize)]
struct Todo {
    id: i64,
    title: String,
    description: String,
    completed: bool
}

#[derive(Serialize, Deserialize)]
struct NewTodo {
    id: i64,
    title: String,
    description: String,
    completed: bool
}

#[debug_handler]
async fn list(State(pool): State<SqlitePool>) -> Result<Json<Vec<Todo>>> {
    // List all todos
    let todos: Vec<Todo> = sqlx::query_as!(Todo, "SELECT id, title, description, completed FROM todos ORDER BY id").fetch_all(&pool).await?;
    Ok(Json(todos))
}

#[debug_handler]
async fn create(State(pool): State<SqlitePool>, Form(todo): Form<NewTodo>) -> Result<String> {
    sqlx::query!("INSERT INTO todos (title, description, completed) VALUES (?, ?, ?)", todo.title, todo.description, todo.completed).execute(&pool).await?;
    Ok(format!("Successfully inserted todo!"))
}

#[debug_handler]
async fn delete(State(pool): State<SqlitePool>, Path(id): Path<i64>) -> Result<String> {
    let result = sqlx::query!("DELETE FROM todos WHERE id = ?", id).execute(&pool).await?;

    if result.rows_affected() > 0 {
        Ok(format!("Successfully deleted todo!"))
    } else {
        Ok(format!("No todo found with id {}", id))
    }      
}

#[debug_handler]
async fn update(State(pool): State<SqlitePool>, Form(todo): Form<Todo>) -> Result<String> {
    sqlx::query!("UPDATE todos SET title = ?, description = ?, completed = ? WHERE id = ?", todo.title, todo.description, todo.completed, todo.id).execute(&pool).await?;
    Ok(format!("Successfully updated todo!"))
}