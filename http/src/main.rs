mod pool;

use std::convert::Infallible;
use std::net::SocketAddr;

use anyhow::Result;
use hyper::server::conn::Http;
use hyper::service::service_fn;
use hyper::{Body, Request, Response};
use tokio::net::TcpListener;

use crate::pool::Pool;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let mut pool = Pool::new(4);

    pool.spawn_many(3, |stream| async move {
        if let Err(http_err) = Http::new()
            .serve_connection(stream, service_fn(hello))
            .await
        {
            eprintln!("Error while serving HTTP connection: {http_err}");
        }
    });

    let addr: SocketAddr = ([127, 0, 0, 1], 8080).into();
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        pool.send(stream)?;
    }
}

async fn hello(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("Hello World!")))
}
