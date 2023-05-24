
use crate::df::{combine, Kind};
pub static L1: OnceLock<OrgText> = OnceLock::new();

use std::{
    collections::BTreeMap,
    fs::File,
    io::{prelude::*, BufReader},
    sync::OnceLock,
};



#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Default, Debug)]
struct Object {
    ty: Kind,
    id: u32,
}

impl Object {
    fn new(ty: Kind, id: u32) -> Object {
        Object { ty, id }
    }
}

#[derive(Debug)]
pub struct Text {
    pub name: String,
    pub text: String,
}
impl Text {
    fn new(a: &str, b: &str) -> Text {
        Text {
            name: a.to_owned(),
            text: b.to_owned(),
        }
    }
}

pub struct OrgText {
    texts: BTreeMap<Object, Text>,
}
impl OrgText {
    fn from_file(filename: &str) {
        let file = File::open(filename).expect("no such file");
        let buf = BufReader::new(file);
        let mut texts: BTreeMap<Object, Text> = BTreeMap::new();
        let mut orgkeys: BTreeMap<String, u16> = BTreeMap::new();
        let mut locations: BTreeMap<String, u16> = BTreeMap::new();

        let mut location: u16 = 1000;
        let mut orgkey: u16 = 1000;
        let mut count = 0;

        for line in buf.lines() {
            // tracing::debug!("{count}");
            if count == 0 {
                count += 1;
                continue;
            };
            count += 1;
            let string = match line {
                Ok(s) => s,
                Err(e) => {
                    println!(" FILE LINE [{count}]  {e} ");
                    continue;
                }
            };

            let parts = string.split("\t");
            let v = parts.collect::<Vec<&str>>();
            //tracing::debug!("{} {} {}" , v[0], v[1], v[2]);
            if let None = orgkeys.get(v[9]) {
                orgkeys.insert(v[9].to_owned(), orgkey);
                orgkey += 1;
            };

            if let None = locations.get(v[16]) {
                locations.insert(v[11].to_owned(), location);
                location += 1;
            };

            let region = combine(orgkeys.get(v[9]).unwrap(), locations.get(v[11]).unwrap());

            let o = Rec::new(v[1], v[2], v[3], v[4], v[5], v[6], v[7], region, v[8], v[10] );

            let shortened_job = match v[13].split_once(":") {
                Some((_a, b)) => b.trim(),
                None => v[13],
            };
            let shortened_posname = match v[22].split_once(":") {
                Some((_a, b)) => b.trim(),
                None => v[22],
            };
            let shortened_ouname = match v[17].split_once(":") {
                Some((_a, b)) => b.trim(),
                None => v[17],
            };
            texts.insert(Object::new(Kind::Area, o.division), Text::new(v[13], v[14]));
            texts.insert(
                Object::new(Kind::Unit, o.orgunit),
                Text::new(v[17], shortened_ouname),
            );
            texts.insert(
                Object::new(Kind::Trap, o.job),
                Text::new(v[13], shortened_job),
            );

            texts.insert(
                Object::new(Kind::Post, o.manager),
                Text::new(v[23], shortened_posname),
            );

            // texts.insert(
            //     Object::new(Kind::Post, o.position),
            //     Text::new("Not Provided", "Not Provided"),
            // );

           // if o.employee != 0 {
            texts.insert(Object::new(Kind::Body, o.employee), Text::new(v[20], v[21]));
            //};
            texts.insert(Object::new(Kind::Site, o.region), Text::new(v[9], v[11]));

            texts.insert(
                Object::new(Kind::Area, o.division),
                 Text::new(v[15], v[16]),
            );
        }

        let _tx = L1.get_or_init(|| OrgText { texts });
    }
}
pub fn init(filename: &str) {
    OrgText::from_file(filename);
}
pub fn get_text(ty: Kind, id: &u32) -> Text {
    let tx = L1.get().unwrap();
    match tx.texts.get(&Object { ty: ty, id: *id }) {
        Some(rec) => Text {
            name: rec.name.to_owned(),
            text: rec.text.to_owned(),
        },
        None => Text {
            name: "".to_owned(),
            text: "".to_owned(),
        },
    }
}

struct Rec {
    position: u32,
    division: u32,
    region: u32,
    orgunit: u32,
    job: u32,
    employee: u32,
    manager: u32,
    manager_eid : u32,
    manager_sid : u32
}

impl Rec {
    fn new(
        poid: &str,
        emid: &str,
        pacode: &str,
        subcode: &str,
        ouid: &str,
        joid: &str,
        mpid: &str,
        region: u32,
        manager_eid: &str,
        manager_sid: &str,
    ) -> Rec {
        let pa = pacode.split_at(1).1.parse::<u16>().unwrap();
        let psa = subcode.split_at(1).1.parse::<u16>().unwrap();
        let division = combine(&pa, &psa);
        let orgunit = ouid.parse::<u32>().unwrap();
        let job = joid.parse::<u32>().unwrap();
        let position = poid.parse::<u32>().unwrap();
        let employee = emid.parse::<u32>().unwrap();
        let manager = mpid.parse::<u32>().unwrap_or(0);
        let manager_eid = manager_eid.parse::<u32>().unwrap_or(0);

        let manager_sid = match manager_sid.is_empty() {
             true => "S000000",
             false => manager_sid
        };
        let manager_sid = manager_sid.parse::<u32>().unwrap_or(0);
        

        Rec {
            position,
            division,
            region,
            orgunit,
            job,
            employee,
            manager,
            manager_eid,
            manager_sid
        }
    }
}