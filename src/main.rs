use csv::Reader;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::str;

/// This is our service handler. It receives a Request, routes on its
/// path, and returns a Future of a Response.
async fn handle_request(req: Request<Body>) -> Result<Response<Body>, anyhow::Error> {
    match (req.method(), req.uri().path()) {
        // Serve some instructions at /
        (&Method::GET, "/") => Ok(Response::new(Body::from(
            "Try POSTing data to /find_rate such as: `curl localhost:8001/get_rate -XPOST -d '78701'`",
        ))),

        // (&Method::POST, "/find_rate_api") => {
        //     let post_body = hyper::body::to_bytes(req.into_body()).await?;
        //     let client = 
        //
        //     Ok(Response::new(Body::from(rate)))
        // }
        (&Method::POST, "/find_rate_old") => {
            let post_body = hyper::body::to_bytes(req.into_body()).await?;
            let mut rate = None; // default is 404

            let rates_data: &[u8] = include_bytes!("rates_by_zipcode.csv");
            let mut rdr = Reader::from_reader(rates_data);
            for result in rdr.records() {
                let record = result?;

                if str::from_utf8(&post_body).unwrap().eq(&record[0]) {
                    rate = Some(record[1].to_string());
                    break;
                }
            }

            Ok(rate.map_or(response404(),|r| Response::new(Body::from(r))))
        }

        // Return the 404 Not Found for other routes.
        _ => Ok(response404())
    }
}

fn response404() -> Response<Body> {
    let mut not_found = Response::default();
    *not_found.status_mut() = StatusCode::NOT_FOUND;
    not_found
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 8001));
    let make_svc = make_service_fn(|_| async move {
        Ok::<_, Infallible>(service_fn(move |req| handle_request(req)))
    });
    let server = Server::bind(&addr).serve(make_svc);
    dbg!("Server started on port 8001");
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
    Ok(())
}
