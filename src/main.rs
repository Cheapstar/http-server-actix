
use actix_web::{get, post, App, HttpResponse, HttpServer, Responder};
use solana_sdk::{bs58, signature::Keypair, signer::Signer};

#[derive(serde::Serialize)]
#[serde(untagged)]
pub enum ApiResponse<T> {
    Success { success: bool, data: T },
    Error { success: bool, error: String },
}
#[derive(serde::Serialize)]
struct KeypairResponse {
    pubkey: String,
    secret: String,
}


#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/keypair")]
async fn gen_keypair()-> impl Responder {

    let keypair = Keypair::new();
    let pubkey_bs = bs58::encode(keypair.pubkey().to_bytes()).into_string();
    let secretkey_bs = bs58::encode(keypair.secret().to_bytes()).into_string();
    let response = ApiResponse::Success { success: true, data: KeypairResponse{
        pubkey:pubkey_bs,
        secret:secretkey_bs
    }}; 

    HttpResponse::Ok().body(format!("{}", serde_json::to_string(&response).unwrap()))
}

#[post("/token/create")]
async fn create_token()->impl Responder{

        HttpResponse::Ok().body(format!("{}", serde_json::to_string(&response).unwrap()))

}


#[actix_web::main]
async fn main()->std::io::Result<()> {
        let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(gen_keypair)

    })
        .bind(("0.0.0.0", port))?
       .run()
       .await
}

