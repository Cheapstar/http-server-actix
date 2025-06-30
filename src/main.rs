
use actix_web::{get, post, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use solana_sdk::{bs58, signature::Keypair, signer::Signer};
use spl_token::{instruction::initialize_mint, solana_program::{instruction::Instruction, pubkey::Pubkey}};

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



#[derive(Deserialize)]
struct MintRequest {
    mintAuthority: String,
    mint: String,
    decimals: u8,
}

#[derive(Serialize)]
struct AccountMetaInfo {
    pubkey: String,
    is_signer: bool,
    is_writable: bool,
}

#[derive(Serialize)]
struct InstructionData {
    program_id: String,
    accounts: Vec<AccountMetaInfo>,
    instruction_data: String,
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
async fn generate_instruction(req: actix_web::web::Json<MintRequest>) -> impl Responder {
    let mint_pubkey: Pubkey = match bs58::decode(&req.mint).into_vec() {
        Ok(bytes) => Pubkey::try_from(bytes.as_slice()).unwrap(),
        Err(_) => return HttpResponse::BadRequest().body("Invalid mint pubkey"),
    };

    let authority_pubkey: Pubkey = match bs58::decode(&req.mintAuthority).into_vec() {
        Ok(bytes) => Pubkey::try_from(bytes.as_slice()).unwrap(),
        Err(_) => return HttpResponse::BadRequest().body("Invalid authority pubkey"),
    };

    let ix: Instruction = initialize_mint(
        &spl_token::ID,
        &mint_pubkey,
        &authority_pubkey,
        Some(&authority_pubkey),
        req.decimals,
    )
    .unwrap();

    let accounts: Vec<AccountMetaInfo> = ix
        .accounts
        .into_iter()
        .map(|a| AccountMetaInfo {
            pubkey: a.pubkey.to_string(),
            is_signer: a.is_signer,
            is_writable: a.is_writable,
        })
        .collect();

    let instruction_data = bs58::encode(ix.data).into_string();

    // Wrap response
    let response = ApiResponse::Success {
        success: true,
        data: InstructionData {
            program_id: ix.program_id.to_string(),
            accounts,
            instruction_data,
        },
    };

    HttpResponse::Ok().json(serde_json::to_string(&response).unwrap())
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

