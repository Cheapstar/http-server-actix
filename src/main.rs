use std::{fmt::format, str::FromStr};

use actix_web::{get, main, web, App, HttpResponse, HttpServer, Responder};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, native_token::LAMPORTS_PER_SOL, pubkey::Pubkey};


#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/balance")]
async fn get_balance()->impl Responder {

    let result = web::block(move || {

        let client =  RpcClient::new_with_commitment(
            String::from("https://api.devnet.solana.com/")
            , CommitmentConfig::confirmed());
            
            let pubkey = Pubkey::from_str("4L4EPzFQQi5mGXDXUY88DHurX6ukm7Nm5AGpEjEqSocH").unwrap();
            let balance = client.get_balance(&pubkey).unwrap();

            balance
        }).await;
    
    match result {
        Ok(balance) => HttpResponse::Ok().body(format!("Balance: {} SOL", balance)),
        Err(e) => HttpResponse::Ok().body(format!("Error Occured {}",e))
    }
}

#[get("/airdrop")]
async fn airdrop()->impl Responder {
    let result = web::block(move|| {

        let client =  RpcClient::new_with_commitment(
            String::from("https://api.devnet.solana.com/")
            , CommitmentConfig::confirmed());
            
            let receiver = Pubkey::from_str("4L4EPzFQQi5mGXDXUY88DHurX6ukm7Nm5AGpEjEqSocH").unwrap();
            let lamports = 1*LAMPORTS_PER_SOL;
            let transaction_signature = client.request_airdrop(&receiver, lamports).unwrap();
            
                loop {
                    if client.confirm_transaction(&transaction_signature).unwrap() {
                        break;
                    }
                }

        }).await;

            match result {
        Ok(()) => HttpResponse::Ok().body(format!("Sol have been airdropped")),
        Err(e) => HttpResponse::Ok().body(format!("Error Occured {}",e))
    }
}

#[actix_web::main]
async fn main()->std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(get_balance)
            .service(airdrop)

    })
.bind(("127.0.0.1",8080))?
       .run()
       .await
}

