use firm::firm_server::{Firm, FirmServer};
use firm::{Ack, Ask, Auth, HomeFile, Search};
use tonic::{transport::Server, Request, Response, Status};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
pub mod firm {
    tonic::include_proto!("firm");
}
mod ad;
mod df;
mod oldap;
mod tx;

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
static EVENT_ID: AtomicU64 = AtomicU64::new(80000000000);

#[derive(Debug, Default)]
pub struct FirmService {}

#[tonic::async_trait]
impl Firm for FirmService {
    async fn search_users(&self, request: Request<Search>) -> Result<Response<Ack>, Status> {
        tracing::debug!("Got a request : {:?}", request);
        let mmap = request.metadata();
        tracing::debug!("request: {:?}", mmap.get("token"));
        let input: Search = request.into_inner(); //
        let word = input.word;

        let map = ad::search_users(&word).await;
            let reply = Ack {
                     ok: true,
                    message: format!("{} users found", map.len()),
                    user: None,
                    claims: None,
                    users: map,                    
                };
                Ok(Response::new(reply))
 }

    async fn create_evp_file(&self, request: Request<HomeFile>) -> Result<Response<Ack>, Status> {
        tracing::debug!("Got a request : {:?}", request);
        let mmap = request.metadata();
        tracing::debug!("request: {:?}", mmap.get("token"));
        let input: HomeFile = request.into_inner(); //
        let filename = input.name;

        let r = ad::create_evp_file(&filename).await;
         match r {
            Ok(()) => {
                let reply = Ack {
                    ok: true,
                    message: format!("file {filename} created"),
                    user: None,
                    claims: None,
                    users : HashMap::new()
                };
                Ok(Response::new(reply))
            },
            Err(e) => {
                let reply = Ack {
                    ok: false,
                    message: format!("Error creating file {filename} : {e}"),
                    user: None,
                    claims: None,
                    users : HashMap::new()
                };
                Ok(Response::new(reply))
            }
         }
      
    }
    async fn get_mesh(&self, request: Request<Ask>) -> Result<Response<Ack>, Status> {
        tracing::debug!("Got a request : {:?}", request);
        let mmap = request.metadata();
        tracing::debug!("request: {:?}", mmap.get("token"));
        let input: Ask = request.into_inner(); //

        let user = df::get_mesh(input.id).await;

        let reply = Ack {
            ok: true,
            message : format!("ok"),
            user: Some(user),
            claims: None,
            users : HashMap::new()
        };
        Ok(Response::new(reply))
    }
    async fn get_claims(&self, request: Request<Ask>) -> Result<Response<Ack>, Status> {
        tracing::debug!("Got a request : {:?}", request);
        let mmap = request.metadata();
        tracing::debug!("request: {:?}", mmap.get("token"));
        let input: Ask = request.into_inner(); //

        let claims = df::get_claims(input.id).await;

        let reply = Ack {
            ok: true,
            message: format!("ok"),
            user: None,
            claims: claims,
            users: HashMap::new()
        };
        Ok(Response::new(reply))
    }
    async fn authenticate(&self, request: Request<Auth>) -> Result<Response<Ack>, Status> {
        let mmap = request.metadata();
        tracing::debug!("request: {:?}", mmap.get("token"));
        let input: Auth = request.into_inner(); //

        let r = oldap::authenticate(&input).await;
        match r {
            Ok(claims) => {
                let reply = Ack {
                    ok: true,
                    message: format!("ok"),
                    user: None,
                    claims: claims,
                    users : HashMap::new()
                };
                Ok(Response::new(reply))
            }
            Err(e) => {
                let reply = Ack {
                    ok: false,
                    message: format!("{e}"),
                    user: None,
                    claims: None,
                    users : HashMap::new()
                };
                Ok(Response::new(reply))
            }
        }
    }
}
// async fn get_user(id : &u32) -> Result<User, Box<dyn std::error::Error>> {
//     todo!()
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_jwt=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    df::init("/usr/local/data/mesh.txt");
    tx::init("/usr/local/data/hc.txt");
    let _ = ad::init("/usr/local/data/users.yaml")?;

    let addr = match "[::1]:50051".parse() {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("{e}");
            std::process::exit(1);
        }
    };

    let firm_service = FirmService::default();

    Server::builder()
        .add_service(FirmServer::new(firm_service))
        .serve(addr)
        .await?;

    Ok(())
}


pub fn next_id() -> u64 {
    let num = EVENT_ID.fetch_add(1, Ordering::SeqCst);
    num
}
