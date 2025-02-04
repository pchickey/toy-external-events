use wstd::http::body::IncomingBody;
use wstd::http::server::{Finished, Responder};
use wstd::http::{Request, Response, StatusCode};

#[wstd::http_server]
async fn main(_request: Request<IncomingBody>, responder: Responder) -> Finished {
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
