
use actix_web::{get, post, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use solana_sdk::{bs58, signature::{Keypair, Signature}, signer::Signer};
use spl_token::{instruction::{initialize_mint, mint_to}, solana_program::{instruction::Instruction, pubkey::Pubkey}};
use base64::{engine::general_purpose, Engine as _};



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

#[derive(Deserialize)]
struct MintToRequest {
    mint: String,
    destination: String,
    authority: String,
    amount: u64,
}
#[derive(Deserialize)]
struct SignMessageRequest {
    message: String,
    secret: String, // base58-encoded secret key (64 bytes)
}

#[derive(Serialize)]
struct SignedMessageResponse {
    signature: String,
    public_key: String,
    message: String,
}

#[derive(Deserialize)]
struct VerifyMessageRequest {
    message: String,
    signature: String, // base64-encoded
    pubkey: String,    // base58-encoded
}

#[derive(Serialize)]
struct VerifyMessageResponse {
    valid: bool,
    message: String,
    pubkey: String,
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

    println!("What is happening");
    HttpResponse::Ok().json(&response)
}

#[post("/token/create")]
async fn generate_token(req: actix_web::web::Json<MintRequest>) -> impl Responder {
    let mint_pubkey: Pubkey = match bs58::decode(&req.mint).into_vec() {
        Ok(bytes) => Pubkey::try_from(bytes.as_slice()).unwrap(),
        Err(_) => return HttpResponse::BadRequest().json(ApiResponse::<()>::Error { success: false, error: "Invalid pub key".to_string() }),
    };

    let authority_pubkey: Pubkey = match bs58::decode(&req.mintAuthority).into_vec() {
        Ok(bytes) => Pubkey::try_from(bytes.as_slice()).unwrap(),
        Err(_) => return HttpResponse::BadRequest().json(ApiResponse::<()>::Error { success: false, error: "Invalid pub key".to_string() }),
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

    let instruction_data = general_purpose::STANDARD.encode(ix.data);

    let response = ApiResponse::Success {
        success: true,
        data: InstructionData {
            program_id: ix.program_id.to_string(),
            accounts,
            instruction_data,
        },
    };

    HttpResponse::Ok().json(&response)
}


#[post("/token/mint")]
async fn mint_token(req: actix_web::web::Json<MintToRequest>) -> impl Responder {
    let mint = match bs58::decode(&req.mint).into_vec() {
        Ok(bytes) => Pubkey::try_from(bytes.as_slice()).unwrap(),
        Err(_) => return HttpResponse::BadRequest().json(ApiResponse::<()>::Error {
            success: false,
            error: "Invalid mint pubkey".to_string(),
        }),
    };

    let destination = match bs58::decode(&req.destination).into_vec() {
        Ok(bytes) => Pubkey::try_from(bytes.as_slice()).unwrap(),
        Err(_) => return HttpResponse::BadRequest().json(ApiResponse::<()>::Error {
            success: false,
            error: "Invalid destination pubkey".to_string(),
        }),
    };

    let authority = match bs58::decode(&req.authority).into_vec() {
        Ok(bytes) => Pubkey::try_from(bytes.as_slice()).unwrap(),
        Err(_) => return HttpResponse::BadRequest().json(ApiResponse::<()>::Error {
            success: false,
            error: "Invalid authority pubkey".to_string(),
        }),
    };

    let ix = mint_to(
        &spl_token::ID,
        &mint,
        &destination,
        &authority,
        &[], // No multisig signers
        req.amount,
    ).unwrap();

    let accounts: Vec<AccountMetaInfo> = ix.accounts
        .into_iter()
        .map(|a| AccountMetaInfo {
            pubkey: a.pubkey.to_string(),
            is_signer: a.is_signer,
            is_writable: a.is_writable,
        })
        .collect();

    let instruction_data = general_purpose::STANDARD.encode(ix.data);

    let response = ApiResponse::Success {
        success: true,
        data: InstructionData {
            program_id: ix.program_id.to_string(),
            accounts,
            instruction_data,
        },
    };

    HttpResponse::Ok().json(&response)
}


#[post("/message/sign")]
async fn sign_message(req: actix_web::web::Json<SignMessageRequest>) -> impl Responder {
    if req.message.is_empty() || req.secret.is_empty() {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::Error {
            success: false,
            error: "Missing required fields".to_string(),
        });
    }

    let secret_bytes = match bs58::decode(&req.secret).into_vec() {
        Ok(bytes) => bytes,
        Err(_) => {
            return HttpResponse::BadRequest().json(ApiResponse::<()>::Error {
                success: false,
                error: "Invalid base58-encoded secret".to_string(),
            });
        }
    };

    // Expecting 64-byte secret key
    if secret_bytes.len() != 64 {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::Error {
            success: false,
            error: "Secret key must be 64 bytes (base58 encoded)".to_string(),
        });
    }

    let keypair = match Keypair::from_bytes(&secret_bytes) {
        Ok(kp) => kp,
        Err(_) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::Error {
                success: false,
                error: "Failed to parse keypair from secret".to_string(),
            });
        }
    };

    let signature: Signature = keypair.sign_message(req.message.as_bytes());

    let response = ApiResponse::Success {
        success: true,
        data: SignedMessageResponse {
            signature: general_purpose::STANDARD.encode(signature.as_ref()),
            public_key: bs58::encode(keypair.pubkey().to_bytes()).into_string(),
            message: req.message.clone(),
        },
    };

    HttpResponse::Ok().json(&response)
}


#[post("/message/verify")]
async fn verify_message(req: actix_web::web::Json<VerifyMessageRequest>) -> impl Responder {
    // Decode the signature from base64
    let sig_bytes = match base64::decode(&req.signature) {
        Ok(bytes) => bytes,
        Err(_) => {
            return HttpResponse::BadRequest().json(ApiResponse::<()>::Error {
                success: false,
                error: "Invalid base64-encoded signature".to_string(),
            });
        }
    };

    let signature = match Signature::try_from(sig_bytes.as_slice()) {
        Ok(sig) => sig,
        Err(_) => {
            return HttpResponse::BadRequest().json(ApiResponse::<()>::Error {
                success: false,
                error: "Invalid signature format".to_string(),
            });
        }
    };

    // Decode the public key from base58
    let pubkey = match bs58::decode(&req.pubkey).into_vec() {
        Ok(bytes) => match Pubkey::try_from(bytes.as_slice()) {
            Ok(pk) => pk,
            Err(_) => {
                return HttpResponse::BadRequest().json(ApiResponse::<()>::Error {
                    success: false,
                    error: "Invalid public key".to_string(),
                });
            }
        },
        Err(_) => {
            return HttpResponse::BadRequest().json(ApiResponse::<()>::Error {
                success: false,
                error: "Invalid base58-encoded public key".to_string(),
            });
        }
    };

    let valid = signature.verify(pubkey.as_ref(), req.message.as_bytes());

    HttpResponse::Ok().json(ApiResponse::Success {
        success: true,
        data: VerifyMessageResponse {
            valid,
            message: req.message.clone(),
            pubkey: req.pubkey.clone(),
        },
    })
}


#[actix_web::main]
async fn main()->std::io::Result<()> {


    HttpServer::new(|| {
        App::new()
    .service(gen_keypair)
    .service(generate_token)
    .service(mint_token)
    .service(sign_message) 

    })
.bind("0.0.0.0:8080")?
.run()
       .await
}

