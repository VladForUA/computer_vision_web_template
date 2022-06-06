use actix::{Actor, StreamHandler};
use actix_web_actors::ws;
use actix_web::body::BoxBody;
use actix_web::dev::ServiceResponse;
use actix_web::http::StatusCode;
use actix_web::middleware::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::{middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer, Result, Responder};
use actix_files::NamedFile;



struct MyWs;

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

//routing for websocket 
async fn index_web_socket(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(MyWs {}, &req, stream);
    println!("{:?}", resp);
    resp
}

//routing for getting HTML resources
async fn index_html() -> impl Responder {
    NamedFile::open_async("./templates/index.html").await.unwrap()
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    // start http server
    HttpServer::new(move || {
        App::new()
            .service(web::resource("/").to(index_html))
            .service(web::resource("/ws").route(web::get().to(index_web_socket)))
            .service(web::scope("").wrap(error_handlers()))
            .wrap(middleware::Logger::default())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

fn error_handlers() -> ErrorHandlers<BoxBody> {
    ErrorHandlers::new().handler(StatusCode::NOT_FOUND, not_found)
}

fn not_found<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<BoxBody>> {
    let response = HttpResponse::build(res.status())
        .content_type("json/text")
        .body("Error");
    Ok(ErrorHandlerResponse::Response(ServiceResponse::new(
        res.into_parts().0,
        response.map_into_left_body(),
    )))
}
