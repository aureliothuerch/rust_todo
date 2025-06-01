use axum::{Router, routing::get};
use axum_error::Result;
use std::{net::SocketAddr};


#[tokio::main]
async fn main() -> Result<()> {
    let dotenv = dotenv::dotenv();
    let url = std::env::var("DATABASE_URL")?;

    match dotenv {
        Ok(dotenv) => println!(".env successfully loaded"),
        Err(dotenv) => println!("Could not find .env")
    }

    let app = Router::new().route("/", get(index)).fallback(index);

    let address: SocketAddr = SocketAddr::from(([0, 0, 0, 0], 8000));
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    
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
