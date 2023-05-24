
use std::{sync::OnceLock, collections::HashMap};

use crate::firm::Claims;

static L1: OnceLock<UserData> = OnceLock::new();

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SUser {
    pub uid: u32,
    pub pid: u32,
    pub cn : String,
    pub dn: String,
}
struct UserData {
    users: Vec<SUser>,
}
impl UserData {
    fn load(filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let f = std::fs::File::open(filename)?;
        let mut users:Vec<SUser> = serde_yaml::from_reader(f)?;
        users.sort_by_key(|v| v.uid);
        println!("loading user data from file {filename}");
        let _ud = L1.get_or_init(|| UserData { users });
        println!("loaded user data from file {filename}");
        Ok(())
    }
}
pub fn init(filename: &str) -> Result<(), Box<dyn std::error::Error>>{
    UserData::load(filename)
}
pub async fn create_evp_file(filename: &str) -> Result<(), std::io::Error>{
    let ud = L1.get().unwrap();
    let mut vec : Vec<String> = Vec::with_capacity(ud.users.len() + 10);
    ud.users.iter().map(|u| (u.uid, 0u8, 0b1111_1111_1111_1111_1111_1111_1111_1111u32, u.pid, false))
    .for_each(| (uuid, cate, days, muid,appr ) | {
                  vec.push( format!("- uuid: {uuid}"));
                  vec.push( format!("  stat: 0"));
                  vec.push( format!("  cate: {cate}"));
                  vec.push( format!("  days: {days}"));
                  vec.push( format!("  muid: {muid}"));
                  vec.push( format!("  appr: {appr}"));
             }
          );
    std::fs::write(filename, vec.join("\n"))?;
    Ok(())      
} 
pub fn get_cn(id: &u32) -> Option<SUser> {
    let ud = L1.get().unwrap();
    match ud.users.binary_search_by_key(id, |u| u.uid) {
        Ok(i) => {
            let u = &ud.users[i];
            Some(SUser { uid : u.uid, pid : u.pid, cn : u.cn.to_string(), dn : u.dn.to_string()})
            
        }
        Err(_) => None
    }
}
pub async fn search_users(word: &str) -> HashMap<u32, Claims> {
    let ud = L1.get().unwrap();
    let map : HashMap<u32,Claims> =
    ud.users.iter().filter(|u| u.cn.contains(&word) ).map(|u|
         (u.uid,
            Claims { uid : u.uid, pid : u.pid, cn : u.cn.to_string(), dn : u.dn.to_string()}
         )).collect();
     map  
}