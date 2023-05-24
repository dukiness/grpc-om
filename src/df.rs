use crate::{ad::{self, get_cn}, tx::{self}, firm::{User, Claims}};
use crate::firm::{Data, Sub};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{self, prelude::*},
    path::Path,
    sync::OnceLock,
};


static L1: OnceLock<MeshData> = OnceLock::new();

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq)]
#[repr(u8)]
pub enum Link {
    Boss,
    Home, // division Workgroup job region <->
    Head, //       //Orgunit                         <->
    Tail, //
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
#[derive(Debug)]
struct Rec {
    post: u32,
    area: u32,
    site: u32,
    unit: u32,
    trap: u32,
    body: u32,
    face: u32,
    boss: u32,
    boss_body: u32,
    boss_face: u32,
}
impl Rec {
    fn new(
        face: &str,
        poid: &str,
        emid: &str,
        paco: &str,
        pasc: &str,
        ouid: &str,
        joid: &str,
        mpid: &str,
        mpbo: &str,
        mpfa: &str,
        site: u32,
    ) -> Rec {
        let paco = paco
            .split_at(1)
            .1
            .parse::<u16>()
            .expect("personnel area problem");
        let pasc = pasc
            .split_at(1)
            .1
            .parse::<u16>()
            .expect("personnel sub area problem");
        let area = combine(&paco, &pasc);
        let unit = ouid.parse::<u32>().unwrap_or(0);
        let trap = joid.parse::<u32>().unwrap_or(0);
        let post = poid.parse::<u32>().unwrap_or(0);
        let body = emid.parse::<u32>().unwrap_or(0);
        let face = face.split_at(1).1.parse::<u32>().unwrap_or(0);
        let boss = match mpid.is_empty() {
            true => "0000000",
            false => mpid,
        };
        let boss = boss.parse::<u32>().unwrap_or(0);

        let boss_face = match mpfa.is_empty() {
            true => "S000000",
            false => mpfa,
        };
        let boss_face = boss_face.split_at(1).1;
        let boss_face = boss_face.parse::<u32>().unwrap_or(0);

        let boss_body = match mpbo.is_empty() {
            true => "0",
            false => mpbo,
        };
        let boss_body = boss_body.parse::<u32>().unwrap_or(0);

        Rec {
            post,
            area,
            site,
            unit,
            trap,
            body,
            face,
            boss,
            boss_body,
            boss_face,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct Kin {
    pub ty1: Kind,
    pub id1: u32,
    pub link: Link,
    pub ty2: Kind,
    pub id2: u32,
}
impl Kin {
    fn push(ty1: Kind, id1: u32, link: Link, ty2: Kind, id2: u32, vec: &mut Vec<Kin>) {
        if id1 == 0 || id2 == 0 {
            return;
        };
        if id1 == id2 {
            println!(
                "[PROBLEM] {:?} {:>4} {:?} {:?} {:>4}",
                ty1, id1, link, ty2, id1
            );
            return;
        }
        vec.push(Kin {
            ty1,
            id1,
            link,
            ty2,
            id2,
        })
    }
}

struct MeshData {
    mesh: Vec<Kin>,
}

//use encoding_rs::WINDOWS_1252;
//use encoding_rs_io::DecodeReaderBytesBuilder;

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where  P: AsRef<Path>,{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
impl MeshData {
    fn load(filename: &str) {
        // let file = File::open(filename).expect("no such file");
        // let buf = //BufReader::new(file);
        // BufReader::new(DecodeReaderBytesBuilder::new()
        //             .encoding(Some(WINDOWS_1252))
        //             .build(file));

        let mut mesh: Vec<Kin> = Vec::with_capacity(12 * 12500);
        let mut uvec: Vec<String> = Vec::with_capacity(13000);

        let mut orgkeys: BTreeMap<String, u16> = BTreeMap::new();
        let mut locations: BTreeMap<String, u16> = BTreeMap::new();

        let mut location: u16 = 1000;
        let mut orgkey: u16 = 1000;
        let mut count = 0;
        if let Ok(lines) = read_lines(filename) {
            for line in lines {
                if count == 0 {
                    count += 1;
                    continue;
                };
                let string = match line {
                    Ok(s) => s,
                    Err(e) => {
                        println!("[{count}] {e}");
                        continue;
                    }
                };

                let parts = string.split("\t");

                let v = parts.collect::<Vec<&str>>();
                if let None = orgkeys.get(v[9]) {
                    orgkeys.insert(v[9].to_owned(), orgkey);
                    orgkey += 1;
                };
                if let None = locations.get(v[11]) {
                    locations.insert(v[16].to_owned(), location);
                    location += 1;
                };
                let region = combine(orgkeys.get(v[9]).unwrap(), locations.get(v[16]).unwrap());
                //fn                  new( face: &str, poid: &str, emid: &str, paco: &str, pasc: &str, ouid: &str, joid: &str, mpid: &str, mpbo: &str, mpfa: &str, site: u32,)
                let o = Rec::new(
                    v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], v[8], v[10], region,
                );

            
                uvec.push(format!("- uid: {}", o.face));
                uvec.push(format!("  pid: {}", o.face));

                Kin::push(
                    Kind::Face,
                    o.face,
                    Link::Boss,
                    Kind::Post,
                    o.boss,
                    &mut mesh,
                );
                Kin::push(
                    Kind::Face,
                    o.face,
                    Link::Home,
                    Kind::Post,
                    o.post,
                    &mut mesh,
                );

                Kin::push(
                    Kind::Post,
                    o.post,
                    Link::Home,
                    Kind::Area,
                    o.area,
                    &mut mesh,
                ); //belongs In
                Kin::push(
                    Kind::Post,
                    o.post,
                    Link::Home,
                    Kind::Site,
                    o.site,
                    &mut mesh,
                ); //is in
                Kin::push(
                    Kind::Post,
                    o.post,
                    Link::Tail,
                    Kind::Unit,
                    o.unit,
                    &mut mesh,
                ); //belongs To
               
                Kin::push(
                    Kind::Post,
                    o.post,
                    Link::Home,
                    Kind::Trap,
                    o.trap,
                    &mut mesh,
                ); //is
                Kin::push(
                    Kind::Post,
                    o.post,
                    Link::Tail,
                    Kind::Body,
                    o.boss_body,
                    &mut mesh,
                ); //Leads
                Kin::push(
                    Kind::Post,
                    o.post,
                    Link::Tail,
                    Kind::Post,
                    o.boss,
                    &mut mesh,
                ); //Reports To
                Kin::push(
                    Kind::Post,
                    o.boss,
                    Link::Head,
                    Kind::Face,
                    o.face,
                    &mut mesh,
                ); //leads user
                Kin::push(
                    Kind::Post,
                    o.boss,
                    Link::Head,
                    Kind::Unit,
                    o.unit,
                    &mut mesh,
                ); //leads unit  dedup later
                   // Kin::push(Kind::Post, o.post, Link::Head, Kind::Body, o.body, &mut mesh);   //leads employee
                   // Kin::push(Kind::Post, o.boss, Link::Head, Kind::Post, o.post, &mut mesh);   //leads position
                      Kin::push(Kind::Post, o.post, Link::Tail, Kind::Face, o.boss_face, &mut mesh);   //leads employee

                count += 1;
            }
        }
        {
        std::fs::write("/usr/local/data/usermanager.yaml", uvec.join("\n")).unwrap();
        println!("yaml written");
        }
        mesh.sort_unstable_by_key(|v| (v.ty1, v.id1, v.link, v.ty2, v.id2));
        mesh.dedup();
        println!(
            "Mesh Data loaded and has {} records of [{}]",
            mesh.len(),
            mesh.capacity()
        );
        let _data = L1.get_or_init(|| MeshData { mesh });
    }
}

pub fn get_manager_position(id: u32) -> u32 {
    let b = Kin {
        ty1: Kind::Face,
        id1: id,
        link: Link::Boss,
        ty2: Kind::Post,
        id2: 0,
    };
    let data = L1.get().unwrap();
    if let Err(i) = data.mesh.binary_search_by_key(&b, |v| *v) {
        let kin = data.mesh[i];
        return kin.id2;
    }
    0
}



    // let args : Vec<String> = std::env::args().collect();
    // let skip = args[1].parse::<usize>().unwrap();
    // data.mesh.iter()
    // .enumerate()
    // .skip(skip)
    // .take(100)
    // .for_each(|(i,v)|     println!("[{i:<8}] {:?} {:>4} {:?} {:?} {:>08}", v.ty1, v.id1, v.link, v.ty2, v.id2));

pub fn get_face_position(id: u32) -> u32 {
    let b = Kin {
        ty1: Kind::Face,
        id1: id,
        link: Link::Home,
        ty2: Kind::Post,
        id2: 0,
    };
    let data = L1.get().unwrap();
    if let Err(i) = data.mesh.binary_search_by_key(&b, |v| *v) {
        let kin = data.mesh[i];
        return kin.id2;
    }
    0
}
pub async fn get_mesh(uid: u32) -> User {
    let mp = get_face_position(uid);
    let id = mp;
    let ty1 = Kind::Post;
    let data = L1.get().expect("L1 is not set");

    let b = Kin {
        ty1,
        id1: id,
        link: Link::Home,
        ty2: Kind::Area,
        id2: 0,
    };
    let mut org: Vec<Data> = Vec::new();
    let mut subs: Vec<Sub>= Vec::new();
    if let Err(mut i) = data.mesh.binary_search_by_key(&b, |v| *v) {
        loop {
            let kin = &data.mesh[i];
            if kin.ty1 != b.ty1 || kin.id1 != b.id1 {
                //println!("We have reached the end");
                return  User { id, org, subs };
            };
            match (kin.link, kin.ty2){
                (Link::Head, Kind::Face) => {
                    if let Some(u) = ad::get_cn(&kin.id2) {
                        subs.push( Sub { uid : u.uid, pid : u.pid, cn : u.cn.to_string(), dn : u.dn.to_string() } );
                    };
                },
                _ =>  {
                     let t = tx::get_text(kin.ty2, &kin.id2);
                     org.push( Data { link : kin.link as i32, ty2 : kin.ty2 as i32, id2 : kin.id2, name: t.name, text : t.text } );
                  }
                }; 
                i += 1;
            };
           
            
    };

    User { id, org,  subs,}
}


pub async fn get_claims(uid: u32) -> Option<Claims> {

    match get_cn(&uid) {
        Some(u) => Some( Claims { uid: u.uid,  pid: u.pid, cn: u.cn.to_owned(), dn : u.dn.to_owned()  } ),
        None => None,
    }
}


 
pub fn init(filename : &str){
    MeshData::load(filename);
}
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     init("/data/sello/mesh.txt");
//     tx::init("/data/sello/hc.txt");
//     let _ = ad::init("/data/sello/users.yaml");
//     let args: Vec<String> = std::env::args().collect();
//     let uid = args[1].parse::<u32>().unwrap();
//     let user = get_mesh(uid).await;

//     user.org.iter().for_each(|v|      println!(
//         " {:?} {:?} {:>08} {:<40} {}", v.link, v.ty2, v.id2, v.name, v.text
//     ));
//     user.subs.iter().for_each(|u| println!("\t\t S{} {} {}", u.uid, u.cn, u.dn ));

//     Ok(())
// }

pub fn combine(a: &u16, b: &u16) -> u32 {
    let a = a.to_be_bytes();
    let b = b.to_be_bytes();
    let c: [u8; 4] = [a[0], a[1], b[0], b[1]];
    u32::from_be_bytes(c)
}

