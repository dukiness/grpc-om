use openldap::*;
use openldap::codes::{options::*, versions::LDAP_VERSION3};
use openldap::errors::*; 
use openldap::errors::LDAPError::NativeError;

use crate::firm::{Claims, Auth};
use crate::df::get_claims;


const FIRM_DOM : &str = "SARSGOV\\";
const FIRM_URI : &str = "ldap://ptaendcs02.sars.gov.za";
const FIRM_ACC : &str = "aa-epmsadbind";
const FIRM_PWD : &str = "MQuhKQm5KW";


pub async fn authenticate(auth : &Auth) -> Result<Option<Claims>, LDAPError> {

    match std::env::var("AUTH_LOCAL") {
        Ok(_s) => {
 
          return authenticate_local(auth).await;
        },
        Err(_e) => {
 
            let ldap = RustLDAP::new(FIRM_URI)?;
            ldap.set_option(LDAP_OPT_PROTOCOL_VERSION, &LDAP_VERSION3);
            ldap.set_option(LDAP_OPT_X_TLS_REQUIRE_CERT, &LDAP_OPT_X_TLS_DEMAND);
            let ad_user = format!("{FIRM_DOM}{}", auth.user_id);
            ldap.simple_bind(&ad_user, &auth.user_pwd)?;
            let uid = auth.user_id.split_at(1).1;
            let uid = uid.parse::<u32>().unwrap(); //if you made it this far the there was an `S` and it is followed by a number
            let claims = get_claims(uid).await;
            Ok(claims)
        }
    }
  
}


pub async fn authenticate_local(auth : &Auth) -> Result<Option<Claims>, LDAPError> {    

    match (auth.user_id.as_str(), auth.user_pwd.as_str()) {
        ("S1028028", "tshepo") => {
            let uid = auth.user_id.split_at(1).1;
            let uid = uid.parse::<u32>().unwrap(); //if you made it this far the there was an `S` and it is followed by a number
            let claims = get_claims(uid).await;
            Ok(claims)
        },
        _ => {
            println!("[BAD STUFF] {} -> {}", auth.user_id, auth.user_pwd);
            Err(NativeError("Invalid user and password combination".to_string()))
        }
        }

}
   