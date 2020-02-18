extern crate reqwest;
extern crate serde_json;
extern crate dotenv;

use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug)]
struct Cli {
    msg: String,
    params: Vec<String>,
    twitch_client_id: String,
    twitch_client_secret: String,
    twitch_client_token: String
}

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: String,
    login: String,
    display_name: String,
    #[serde(rename = "type")]
    user_type: String,
    broadcaster_type: String,
    description: String,
    view_count: u32,
    offline_image_url: String,
    profile_image_url: String
}

impl Cli {
    async fn get_token( &self ) -> Result<String, reqwest::Error> {
        let client = reqwest::Client::new();
        let resp = client.post(format!("https://id.twitch.tv/oauth2/token?client_id={}&client_secret={}&grant_type=client_credentials", self.twitch_client_id, self.twitch_client_secret).as_str())
            .send().await?;
    
        let text = resp.text().await?;
        Ok(text)
    }
    fn remove_data_scope( &self, text: &str ) -> Result<String, serde_json::Error> {
        let deserialized: serde_json::Value = serde_json::from_str(&text).unwrap();
        let data = deserialized.get("data").unwrap();
        let serialized = serde_json::to_string(&data).unwrap();
        Ok(serialized)
    }
    async fn get_user( &self, uid: u32 ) -> Result<Vec<User>, reqwest::Error> {    
        let client = reqwest::Client::new();
        let resp = client.get(format!("https://api.twitch.tv/helix/users?id={}", &uid).as_str())
            .bearer_auth(self.twitch_client_token.as_str())
            .send().await?;
    
        let text = resp.text().await?;
        let data = self.remove_data_scope(&text.as_str()).unwrap();
        let user: Vec<User> = serde_json::from_str(&data).unwrap();
        Ok(user)
    }
}




#[tokio::main]
async fn main() -> Result<(), reqwest::Error>  {
    dotenv().ok();
    let mut twitch_client_id = String::new();
    let mut twitch_client_secret = String::new();
    let mut twitch_client_token = String::new();

    for (key, value) in env::vars() {
        if key == "TWITCH_CLIENT_ID" { twitch_client_id = String::from(value) }
        else if key == "TWITCH_CLIENT_SECRET" { twitch_client_secret = String::from(value) }
        else if key == "TWITCH_CLIENT_TOKEN" { twitch_client_token = String::from(value) }
    }

    let args: Vec<String> = env::args().collect();
    let cli = Cli { msg: String::from(args[0].trim()), params: args, twitch_client_id, twitch_client_secret, twitch_client_token };
    
    for param in &cli.params {
        let full_param: Vec<&str> = param.split("=").collect();
        match full_param[0] {
            "user" => {
                let uid: u32 = full_param[1].parse().unwrap();
                let user = cli.get_user(uid).await?;
                println!("{:?}", &user);
            },
            "token" => {
                let token = cli.get_token().await?;
                println!("{}", &token);
            },
            _      => {}
        };
    }

    Ok(())
}