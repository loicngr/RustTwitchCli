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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize)]
struct Stream {
    id: String,
    user_id: String,
    user_name: String,
    game_id: String,
    #[serde(rename = "type")]
    user_type: String,
    title: String,
    viewer_count: u32,
    started_at: String,
    language: String,
    thumbnail_url: String
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
    async fn get_stream_user( &self, uid: u32 ) -> Result<Vec<Stream>, reqwest::Error> {
        let client = reqwest::Client::new();
        let resp = client.get(format!("https://api.twitch.tv/helix/streams?user_id={}", &uid).as_str())
                .header("Client-ID", self.twitch_client_id.as_str())
                .send().await?;
        let text = resp.text().await?;
        let data = self.remove_data_scope(&text.as_str()).unwrap();
        let stream: Vec<Stream> = serde_json::from_str(&data).unwrap();
        Ok(stream)
    }
    async fn get_user_by_login( &self, login: String ) -> Result<Vec<User>, reqwest::Error> {
        let client = reqwest::Client::new();
        let resp = client.get(format!("https://api.twitch.tv/helix/users?login={}", &login).as_str())
            .bearer_auth(self.twitch_client_token.as_str())
            .send().await?;
    
        let text = resp.text().await?;
        let data = self.remove_data_scope(&text.as_str()).unwrap();
        let user: Vec<User> = serde_json::from_str(&data).unwrap();
        Ok(user)
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
    let twitch_client_id = dotenv::var("TWITCH_CLIENT_ID").unwrap();
    let twitch_client_secret = dotenv::var("TWITCH_CLIENT_SECRET").unwrap();
    let twitch_client_token = dotenv::var("TWITCH_CLIENT_TOKEN").unwrap();

    let args: Vec<String> = env::args().collect();
    let cli = Cli { msg: String::from(args[0].trim()), params: args, twitch_client_id, twitch_client_secret, twitch_client_token };
    
    for param in &cli.params {
        let full_param: Vec<&str> = param.split("=").collect();
        match full_param[0] {
            "info-user"     => {
                let uid: u32 = full_param[1].parse().unwrap();
                let user = cli.get_user(uid).await?;
                println!("{:?}", &user);
            },
            "isonlive-user" => {
                let uid: u32 = full_param[1].parse().unwrap();
                let stream = cli.get_stream_user(uid).await?;
                println!("{:?}", &stream);
            },
            "token"         => {
                let token = cli.get_token().await?;
                println!("{}", &token);
            },
            "uid"           => {
                let login: String = full_param[1].parse().unwrap();
                let user: Vec<User> = cli.get_user_by_login(login).await?;
                let uid: &User = user.first().unwrap();
                println!("{:?}", &uid.id);
            },
            "help"          => {
                println!("Please, see the readme file for help.");
            },
            _               => { }
        };
    }

    if cli.params.len() <= 1 { println!("Don't forget to give me some parameters.."); }

    Ok(())
}