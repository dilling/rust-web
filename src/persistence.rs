#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(unused_imports)]

//!
//! PERSISTENCE
//! -----------
//!
//! Every web application needs to store data. There are, of course, many Rust
//! crates for interacting with NoSQL databases and AWS services like DynamoDB.
//! There are even some ORM-like solutions for Rust that aim to emulate the
//! ORM solutions from the Java world. However, most web applications will rely
//! on relational databases for persistence because of their ubiquity,
//! flexibility, performance, and ACID guarantees.
//!
//! Rust has many solutions for interacting with relational databases. One of
//! the most common that does not try to hide SQL from the user, and which is
//! fully compatible with Tokio, is the `sqlx` crate.
//!
//! In this section, you will learn the basics of using the `sqlx` crate to
//! interact with a PostgreSQL database.
//!
//! To get started:
//!
//! 1. Run `cargo install sqlx-cli` to install the SQLx CLI.
//!
//! 2. Set the environment variable
//! `DATABASE_URL=postgres://<user>:<password>@<address>:<port>/<database>`.
//! For example, `DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres`.
//!
//! 3. Run `sqlx database create` to create the database.
//!
//! 4. Run `sqlx migrate run` to run the migrations in the `migrations` folder.
//!

use axum::{async_trait, extract::{Path, State}, routing::{delete, get, post, put}, Json, Router};
use serde::de;
use sqlx::{pool, postgres::PgPoolOptions, types::time::PrimitiveDateTime, Pool, Postgres};

///
/// EXERCISE 1
///
/// Experiment with the `sqlx::query!` macro. If you have configured your
/// DATABASE_URL correctly (with a running Postgres), then you should be able
/// to get live feedback from the macro.
///
/// At the same time, try the `sqlx::query::<Postgres>` function, which is NOT a macro.
/// What can you say about the difference between the two?
///
/// Note that calling either `query` does not actually execute the query. For that, you
/// need to supply a database pool, which you can do so with the `fetch` family of
/// methods.
///
async fn query_playground() {
    let _ = sqlx::query!("SELECT 1 + 1 AS sum");

    let _ = sqlx::query::<Postgres>("SELECT 1 + 1 AS sum");
}

///
/// EXERCISE 2
///
/// Use the `sqlx::query!` macro to select the result of `1 + 1` from the database,
/// being sure to name the column `sum` using SQL's `AS` keyword.
///
/// Then modify the test to reference a row, which you can obtain by using the
/// `fetch_one` method on the query result, and awaiting and unwrapping it.
///
#[tokio::test]
async fn select_one_plus_one() {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let sum: i32 = sqlx::query!("SELECT 1 + 1 AS sum")
        .fetch_one(&pool).await.unwrap().sum.unwrap();

    assert_eq!(sum, 2);
}

///
/// EXERCISE 3
///
/// In this example, we are going to show the strength of sqlx by
/// doing a select star query.
///
/// Use the `sqlx::query!` macro to select all columns from the `todos` table.
/// Use a `fetch_all`, and iterate over them, printing out each row.
///
/// What do you notice about the type of the row?
///
#[tokio::test]
async fn select_star() {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let todos = sqlx::query!("SELECT * from todos")
        .fetch_all(&pool).await.unwrap();

    for todo in todos {
        println!("{:?}", todo);
    }

    assert!(true);
}

///
/// EXERCISE 4
///
/// The `query!` macro supports parameterized queries, which you can create using the
/// placeholder syntax '$1', '$2', etc. You then supply these parameters after the
/// main query.
///
/// Use the `query!` macro to insert a row into the `todo` table, keeping
/// in mind every todo has a title, description, and a boolean indicating
/// whether it is done.
///
/// Using the `RETURNING` keyword, return the id of the inserted row,
/// and assert it is greater than zero.
///
#[tokio::test]
async fn insert_todo() {
    let _pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let _title = "Learn SQLx";
    let _description = "I should really learn SQLx for my Axum web app";
    let _done = false;

    let query = sqlx::query!(
        "INSERT INTO todos (title, description, done) VALUES ($1, $2, $3) RETURNING id",
        _title,
        _description,
        _done
    );

    let id = query.fetch_one(&_pool).await.unwrap().id;

    assert!(id > 0);
}

///
/// EXERCISE 5
///
/// Use the `query!` macro to update a row in the `todo` table.
///
/// You may want to use `execute` to execute the query, rather than one
/// of the fetch methods.
///
#[tokio::test]
async fn update_todo_test() {
    let _pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let _id = 1;
    let _done = true;

    assert!(true);
}

///
/// EXERCISE 6
///
/// Use the `query!` macro to delete a row in the `todo` table.
///
/// You may want to use `execute` to execute the query, rather than one
/// of the fetch methods.
///
#[tokio::test]
async fn delete_todo_test() {
    let _pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let _id = 1;

    assert!(true);
}

///
/// EXERCISE 7
///
/// You do not have to rely on SQLx generating anonymous structs for you.
/// With the `sqlx::query_as!` macro, you can specify the type of the row
/// yourself.
///
/// In this exercise, introduce a struct called `Todo` that models the `todos`
/// table, and use the `sqlx::query_as!` macro to select all columns from the
/// `todos` table.
///
#[tokio::test]
async fn select_star_as() {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let query = sqlx::query_as!(
        Todo,
        "SELECT * from todos"
    );

    let todos = query.fetch_all(&pool).await.unwrap();

    for todo in todos {
        println!("{:?}", todo);
    }
    assert!(true);
}

#[derive(Debug)]
struct Todo {
    id: i64,
    title: String,
    description: String,
    done: bool,
    created_at: PrimitiveDateTime,
}
impl Todo {
    pub fn to_dto(&self) -> TodoDTO {
        TodoDTO {
            id: self.id,
            title: self.title.clone(),
            description: self.description.clone(),
            done: self.done,
            created_at: self.created_at.to_string(),
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct TodoDTO {
    id: i64,
    title: String,
    description: String,
    done: bool,
    created_at: String,
}

///
/// GRADUATION PROJECT
///
/// In this project, you will build a simple CRUD API for a todo list,
/// which uses sqlx for persistence.
///
pub async fn run_todo_app() {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let todo_state = TodoState { repo: TodoRepoPostgres { pool } };

    let todo_routes: Router<TodoState<TodoRepoPostgres>> = Router::new()
        .route("/", get(get_todos))
        .route("/:id", get(get_todo))
        .route("/", post(create_todo))
        .route("/:id", put(update_todo))
        .route("/:id", delete(delete_todo));

    let app = Router::new()
        .nest("/todo/", todo_routes)
        .with_state(todo_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

#[derive(Clone)]
struct TodoState<R: TodoRepo> {
    repo: R
}

#[async_trait]
trait TodoRepo: Send + Sync {
    async fn get_todos(&self) -> Vec<Todo>;
    async fn get_todo(&self, id: i64) -> Option<Todo>;
    async fn create_todo(&self, title: &str, description: &str) -> i64;
    async fn update_todo(
        &self,
        id: i64,
        title: Option<&str>,
        description: Option<&str>,
        done: Option<bool>,
    ) -> Option<i64>;
    async fn delete_todo(&self, id: i64) -> i64;

}

#[derive(Clone)]
struct TodoRepoPostgres {
    pool: Pool<Postgres>,
}

#[async_trait]
impl TodoRepo for TodoRepoPostgres {
    async fn get_todos(&self) -> Vec<Todo> {
        let query = sqlx::query_as!(Todo, "SELECT * from todos");
        query.fetch_all(&self.pool).await.unwrap()
    }
    async fn get_todo(&self, id: i64) -> Option<Todo> {
        let query = sqlx::query_as!(Todo, "SELECT * from todos where id = $1", id);
        query.fetch_optional(&self.pool).await.unwrap()
    }
    async fn create_todo(&self, title: &str, description: &str) -> i64 {
        let query = sqlx::query!(
            "INSERT INTO todos (title, description, done) VALUES ($1, $2, $3) RETURNING id",
            title,
            description,
            false
        );
        query.fetch_one(&self.pool).await.unwrap().id
    }
    async fn update_todo(
        &self,
        id: i64,
        title: Option<&str>,
        description: Option<&str>,
        done: Option<bool>,
    ) -> Option<i64> {
        let query = sqlx::query!(
            "UPDATE todos SET title = COALESCE($1, title), description = COALESCE($2, description), done = COALESCE($3, done) where id = $4 RETURNING id",
            title,
            description,
            done,
            id
        );
    
        query.fetch_optional(&self.pool).await.unwrap().map(|row| row.id)
    }
    async fn delete_todo(&self, id: i64) -> i64 {
        let query = sqlx::query!(
            "DELETE FROM todos where id = $1 RETURNING id",
            id
        );
    
        query.fetch_one(&self.pool).await.unwrap().id
    }
}

async fn get_todos<R: TodoRepo>(
    State(TodoState{ repo }): State<TodoState<R>>,
) -> Json<Vec<TodoDTO>> {
    let todos =  repo.get_todos().await;
    Json(todos.into_iter().map(|todo| todo.to_dto()).collect())
}

async fn get_todo<R: TodoRepo>(
    Path(id): Path<i64>,
    State(TodoState{ repo }): State<TodoState<R>>,
) -> Json<Option<TodoDTO>> {
    let maybe_todo = repo.get_todo(id).await;
    Json(maybe_todo.map(|todo| todo.to_dto()))
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct CreateTodo {
    title: String,
    description: String,
}

async fn create_todo<R: TodoRepo>(
    State(TodoState{ repo }): State<TodoState<R>>,
    body: Json<CreateTodo>
) -> Json<i64> {
    let id = repo.create_todo(&body.title, &body.description).await;
    Json(id)
}

#[derive(Debug, serde::Deserialize)]
struct UpdateTodo {
    title: Option<String>,
    description: Option<String>,
    done: Option<bool>,
}

async fn update_todo<R: TodoRepo>(
    Path(id): Path<i64>,
    State(TodoState{ repo }): State<TodoState<R>>,
    Json(UpdateTodo{ title, description, done }): Json<UpdateTodo>
) -> Json<Option<i64>> {
    let id = repo.update_todo(id, title.as_deref(), description.as_deref(), done).await;
    Json(id)
}

async fn delete_todo<R: TodoRepo>(
    Path(id): Path<i64>,
    State(TodoState{ repo }): State<TodoState<R>>,
) -> Json<i64> {
    let deleted_id = repo.delete_todo(id).await;
    Json(deleted_id)
}