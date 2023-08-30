use serde_json::json;
use worker::*;

mod utils;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or_else(|| "unknown region".into())
    );
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    let mut headers = Headers::new();
    headers.append("Access-Control-Allow-Headers", "*")?;
    headers.append("Access-Control-Allow-Methods", "POST,GET,OPTIONS")?;
    headers.append("Access-Control-Allow-Origin", "*")?;

    if req.method() == Method::Options {
        let response = Response::ok("OK")?;
        return Ok(response.with_headers(headers));
    }

    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    // Optionally, use the Router to handle matching endpoints, use ":name" placeholders, or "*name"
    // catch-alls to match on specific patterns. Alternatively, use `Router::with_data(D)` to
    // provide arbitrary data that will be accessible in each route via the `ctx.data()` method.
    let router = Router::new();

    // Add as many routes as your Worker needs! Each route will get a `Request` for handling HTTP
    // functionality and a `RouteContext` which you can use to  and get route parameters and
    // Environment bindings like KV Stores, Durable Objects, Secrets, and Variables.
    router
        .get("/", |_, _| Response::ok("Hello from Workers!"))
        .post_async("/upload", |mut req, ctx| async move {
            let form = req.form_data().await?;
            if let Some(entry) = form.get("file") {
                match entry {
                    FormEntry::File(file) => {
                        let bytes = file.bytes().await?;
                        let kv = ctx.kv("IMAGES")?;
                        kv.put("test", bytes)?.execute().await?;

												let mut headers = Headers::new();
												headers.append("Access-Control-Allow-Headers", "*")?;
                        headers.append("Access-Control-Allow-Methods", "POST")?;
                        headers.append("Access-Control-Allow-Origin", "*")?;

                        let response = Response::ok("Got a file, thanks!")?;
                        return Ok(response.with_headers(headers));
                    }
                    FormEntry::Field(_) => return Response::error("Bad Request", 400),
                }
            }

            Response::error("Bad Request", 400)
        })
        .get("/worker-version", |_, ctx| {
            let version = ctx.var("WORKERS_RS_VERSION")?.to_string();
            Response::ok(version)
        })
        .run(req, env)
        .await
}
