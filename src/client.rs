use std::env::args;
use std::fmt::Display;

use firm::firm_client::FirmClient;
use firm::{Ack, Ask, User, Claims, HomeFile, Search};
use rpassword::read_password;
use std::io::Write;

use crate::firm::Auth;

//use tonic::metadata::{AsciiMetadataValue};

pub mod firm {
    tonic::include_proto!("firm");
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq)]
#[repr(u8)]
pub enum Link {
    Boss,
    Home, // division Workgroup job region <->
    Head, //       //Orgunit                         <->
    Tail, //
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq, Default)]
#[repr(u8)]
pub enum Kind {
    Area, //Division Department
    Site, //Region
    Unit, //Org unit
    Trap, //Job
    Post, //Spot
    Body, //Employee
    #[default]
    Face, //User
    Crew, //Work Group
    Oops,
}
impl Display for Kind   {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {     
        match self {
            Kind::Area => write!(f, "Area"),
            Kind::Site => write!(f, "Site"),
            Kind::Unit => write!(f, "Unit"),
            Kind::Trap => write!(f, "Trap"),
            Kind::Post => write!(f, "Post"),
            Kind::Body => write!(f, "Body"),
            Kind::Face => write!(f, "Face"),
            Kind::Crew => write!(f, "Crew"),
            Kind::Oops => write!(f, "Oops"),
           }
    }   
}
impl Kind {
fn from(i: u8) -> Kind {
    match i {
        0 => Kind::Post,
        1 => Kind::Area,
        2 => Kind::Site,
        3 => Kind::Unit,
        4 => Kind::Trap,
        5 => Kind::Body,
        6 => Kind::Crew,
        7 => Kind::Face,
        _ => Kind::Oops,
    }
}
}

impl Display for Link   {
     
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
       
        match self {
            Link::Boss => write!(f,  "Boss"),
            Link::Home => write!(f,  "Home"),
            Link::Head => write!(f,  "Head"),
            Link::Tail => write!(f,  "Tail"),
            Link::Other => todo!(),

           }
    }
}
impl Link {
    fn from(i: u8) -> Link {
        match i {
            0 => Link::Boss,
            1 => Link::Home,
            2 => Link::Head,
            3 => Link::Tail,
            _ => Link::Other,
        }
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //let token= AsciiMetadataValue::try_from("The token that identifies of Dukiness").unwrap();
    let args: Vec<String> = args().collect();
    let id = &args[1];
    let id = id.parse::<u32>().unwrap();
    let mut client = FirmClient::connect("http://[::1]:50051").await?;
    let msg = Ask { id };
    let tonic_request: tonic::Request<Ask> = tonic::Request::new(msg);
    //tonic_request.metadata_mut().insert("token", token);
    let tonic_response: tonic::Response<Ack> = client.get_mesh(tonic_request).await?;
    let cr: Ack = tonic_response.into_inner();
    match cr.ok {
        true => {
            let user: User = cr.user.unwrap();
            //let user: &User = &user;
            println!();
            println!("{:>100}", "get_mesh -> org");
            println!();
            user.org.iter().map(|v| (Link::from(v.link as u8), Kind::from(v.ty2 as u8), v.id2, v.name.to_string(), v.text.to_string() ))
            .for_each(|(link,ty, id, name, text)| println!("\t{link:>4} {ty:>4} {id:>08} {name:<40} {text}"));
        println!();
            println!("{:>100}", "get_mesh -> subs");
            println!();
            user.subs
                .iter()
                .for_each(|u| println!("\t{:>4} {:>4} S{} {:<40} {}", "", "", u.uid, u.cn, u.dn));
        }
        false => println!("user not found"),
    }
    println!();
    let msg = Ask { id };
    let tonic_request: tonic::Request<Ask> = tonic::Request::new(msg);
    //tonic_request.metadata_mut().insert("token", token);
    let tonic_response: tonic::Response<Ack> = client.get_claims(tonic_request).await?;
    let cr: Ack = tonic_response.into_inner();
    match cr.ok {
        true => {
            let claims: Claims = cr.claims.unwrap();
             println!();
             println!("{:>100}", "get_claims -> claims");
             println!();
             println!("\t{:>4} {:>4} S{} {} {} {}", "", "", claims.uid, claims.pid, claims.cn, claims.dn);
        }
        false => println!("user not found"),
    }

    println!();
    println!("{:>100}", "authenticate -> claims");
    let user_id = format!("S{id}");
    print!("Enter the password: ");
    std::io::stdout().flush()?;
    let user_pwd = read_password()?;
    println!("{user_id} -> {user_pwd}");
    let msg: Auth = Auth  { user_id, user_pwd };
    let tonic_request: tonic::Request<Auth> = tonic::Request::new(msg);
    //tonic_request.metadata_mut().insert("token", token);
    let tonic_response: tonic::Response<Ack> = client.authenticate(tonic_request).await?;
    let cr: Ack = tonic_response.into_inner();
    match cr.ok {
        true => {
            let claims: Claims = cr.claims.unwrap();
             println!();
             println!("{:>100}", "get_claims -> claims");
             println!();
             println!("\t{:>4} {:>4} S{} {} {} {}", "", "", claims.uid, claims.pid, claims.cn, claims.dn);
        }
        false => println!("invalid log in details"),
    }

    println!();
    println!("{:>100}", "create evp file (init)-> ()");
  
    let msg: HomeFile = HomeFile  { name : "/data/sello/evp.yaml".to_string() };
    let tonic_request: tonic::Request<HomeFile> = tonic::Request::new(msg);
    //tonic_request.metadata_mut().insert("token", token);
    let tonic_response: tonic::Response<Ack> = client.create_evp_file(tonic_request).await?;
    let cr: Ack = tonic_response.into_inner();
    match cr.ok {
        true => {
             println!();             
             println!("\t{}",cr.message);
        },
        false =>   println!("\t{}",cr.message)
    }

    println!();
    println!("{:>100}", "search");
  
    let msg: Search = Search  { word : "Sello".to_string() };
    let tonic_request: tonic::Request<Search> = tonic::Request::new(msg);
    //tonic_request.metadata_mut().insert("token", token);
    let tonic_response: tonic::Response<Ack> = client.search_users(tonic_request).await?;
    let cr: Ack = tonic_response.into_inner();
    match cr.ok {
        true => {
             println!();
             cr.users.iter().for_each(|(_u, claims)| 
                println!("\t{:>4} {:>4} S{} {:>08} {} {}", "", "", claims.uid, claims.pid, claims.cn, claims.dn));
             println!("\t{}",cr.message);
        },
        false =>   println!("\t{}",cr.message)
    }




    Ok(())
}
