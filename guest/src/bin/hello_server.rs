use wstd::http::body::IncomingBody;
use wstd::http::server::{Finished, Responder};
use wstd::http::{Request, Response, StatusCode};

#[wstd::http_server]
async fn main(_request: Request<IncomingBody>, responder: Responder) -> Finished {
    println!("Hello, world!");
    wstd::task::sleep(wstd::time::Duration::from_micros(10)).await;
    println!("That was a nice nap");
    responder
        .respond(
            Response::builder()
                .status(StatusCode::OK)
                .header("Hello", "world")
                .body(wstd::io::empty())
                .unwrap(),
        )
        .await
}
