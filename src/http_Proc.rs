use axum::{
    routing::{get, post},
    Router,
    response::{Html, IntoResponse},
    http::StatusCode,
    Form
};
use std::net::SocketAddr;
use tokio::net::TcpListener;



pub async fn spawn() -> anyhow::Result<()> {
    let app = Router::new()
        .route("/test", get(hello_handler))
        .fallback(fallback_handler);

    let PORT = 3000;    

    let addr = SocketAddr::from(([0, 0, 0, 0], PORT));
    println!(          "*********************************************************");
    println!(          "*********************************************************");
    println!(          "***  ***************************************    ****  ***");
    println!(          "***  *******                           *****  ** ***  ***");
    println!(          "***  *******STARTUP SERVER AT PORT {}*****  *** **  ***", PORT);
    println!(          "***  *******                           *****  **** *  ***");
    println!(          "***  ***************************************  *****   ***");
    println!(          "***       **********************************  ******  ***");
    println!(          "*********************************************************");
    println!(" Опрос доступен на русском и фарси");

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    print!("WORKING!!!!");
    Ok(())
}




async fn hello_handler() -> &'static str {
    "hello world"
}


async fn fallback_handler() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Страница не найдена")
}
