mod errors;
mod handlers;
mod models;
mod router;
mod state;

use crate::router::create_router;
use crate::state::create_application_state;

const LOCAL_ADDRESS: &str = "0.0.0.0:3000";

#[tokio::main]
async fn main() {
    let application_state = create_application_state();

    let router = create_router(application_state);

    let listener = match tokio::net::TcpListener::bind(LOCAL_ADDRESS).await {
        Ok(ok) => ok,
        Err(err) => {
            println!("Error binding to port 3000: {err}");
            return;
        }
    };

    println!("Server started and listening on: {LOCAL_ADDRESS}");

    if let Err(e) = axum::serve(listener, router).await {
        println!("Couldn't start Axum: {e}");
    }
}
