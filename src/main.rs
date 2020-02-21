extern crate dotenv;
extern crate reqwest;
extern crate serde_json;

use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug)]
struct Cli {
    msg: String,
    params: Vec<String>,
    twitch_client_id: String,
    twitch_client_secret: String,
    twitch_client_token: String,
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
    profile_image_url: String,
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
    thumbnail_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TopGames {
    id: String,
    name: String,
    box_art_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct BannedEvents {
    id: String,
    event_type: String,
    event_timestamp: String,
    version: String,
    event_data: BannedEventsData,
}
// TO DO : optimizing this struct, maybe reducing it to one struct
#[derive(Debug, Serialize, Deserialize)]
struct BannedEventsData {
    broadcaster_id: String,
    broadcaster_name: String,
    user_id: String,
    user_name: String,
    expires_at: String,
}

#[derive(Debug)]
enum TokenOption<T> {
    Some(T),
    None,
}

impl Cli {
    async fn get_token(&self, scopes: TokenOption<Vec<String>>) -> Result<String, reqwest::Error> {
        let client = reqwest::Client::new();
        match &scopes {
            TokenOption::None => {
                println!("No scopes given");
                let resp = client.post(format!("https://id.twitch.tv/oauth2/token?client_id={}&client_secret={}&grant_type=client_credentials", 
                    self.twitch_client_id, self.twitch_client_secret).as_str())
                    .send().await?;
                let text = resp.text().await?;
                Ok(text)
            }
            TokenOption::Some(t) => {
                println!("Scopes given: {:?}", t);
                let resp = client.post(format!("https://id.twitch.tv/oauth2/token?client_id={}&client_secret={}&grant_type=client_credentials&scope={}", 
                    self.twitch_client_id, self.twitch_client_secret, t.join(" ")).as_str()).send().await?;
                let text = resp.text().await?;
                Ok(text)
            }
        }
    }
    fn remove_data_scope(&self, text: &str) -> Result<String, serde_json::Error> {
        let deserialized: serde_json::Value = serde_json::from_str(&text).unwrap();
        let data = deserialized.get("data").unwrap();
        let serialized = serde_json::to_string(&data).unwrap();
        Ok(serialized)
    }
    async fn get_stream_user(&self, uid: u32) -> Result<Vec<Stream>, reqwest::Error> {
        let client = reqwest::Client::new();
        let resp = client
            .get(format!("https://api.twitch.tv/helix/streams?user_id={}", &uid).as_str())
            .header("Client-ID", self.twitch_client_id.as_str())
            .send()
            .await?;
        let text = resp.text().await?;
        let data = self.remove_data_scope(&text.as_str()).unwrap();
        let stream: Vec<Stream> = serde_json::from_str(&data).unwrap();
        Ok(stream)
    }
    async fn get_user_by_login(&self, login: String) -> Result<Vec<User>, reqwest::Error> {
        let client = reqwest::Client::new();
        let resp = client
            .get(format!("https://api.twitch.tv/helix/users?login={}", &login).as_str())
            .bearer_auth(self.twitch_client_token.as_str())
            .send()
            .await?;
        let text = resp.text().await?;
        let data = self.remove_data_scope(&text.as_str()).unwrap();
        let user: Vec<User> = serde_json::from_str(&data).unwrap();
        Ok(user)
    }
    async fn get_user(&self, uid: u32) -> Result<Vec<User>, reqwest::Error> {
        let client = reqwest::Client::new();
        let resp = client
            .get(format!("https://api.twitch.tv/helix/users?id={}", &uid).as_str())
            .bearer_auth(self.twitch_client_token.as_str())
            .send()
            .await?;
        let text = resp.text().await?;
        let data = self.remove_data_scope(&text.as_str()).unwrap();
        let user: Vec<User> = serde_json::from_str(&data).unwrap();
        Ok(user)
    }
    async fn get_top_games(&self) -> Result<Vec<TopGames>, reqwest::Error> {
        let client = reqwest::Client::new();
        let resp = client
            .get("https://api.twitch.tv/helix/games/top")
            .header("Client-ID", self.twitch_client_id.as_str())
            .send()
            .await?;
        let text = resp.text().await?;
        let data = self.remove_data_scope(&text.as_str()).unwrap();
        let top_games: Vec<TopGames> = serde_json::from_str(&data).unwrap();
        Ok(top_games)
    }
    async fn get_banned_events(&self, bid: u32) -> Result<Vec<BannedEvents>, reqwest::Error> {
        let client = reqwest::Client::new();
        let resp = client
            .get(
                format!(
                    "https://api.twitch.tv/helix/moderation/banned/events?broadcaster_id={}",
                    &bid
                )
                .as_str(),
            )
            .bearer_auth(self.twitch_client_token.as_str())
            .send()
            .await?;
        let text = resp.text().await?;
        let deserialized_resp: serde_json::Value = serde_json::from_str(&text).unwrap();
        match deserialized_resp.get("status") {
            None => panic!("error request, maybe no right to make this request ?"),
            Some(nbr) => {
                if nbr.as_u64().unwrap() != 401u64 {
                    let data = self.remove_data_scope(&text.as_str()).unwrap();
                    let banned_events: Vec<BannedEvents> = serde_json::from_str(&data).unwrap();
                    Ok(banned_events)
                } else {
                    panic!("{}", deserialized_resp)
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    dotenv().ok();
    let twitch_client_id = dotenv::var("TWITCH_CLIENT_ID").unwrap();
    let twitch_client_secret = dotenv::var("TWITCH_CLIENT_SECRET").unwrap();
    let twitch_client_token = dotenv::var("TWITCH_CLIENT_TOKEN").unwrap();

    let args: Vec<String> = env::args().collect();
    let cli = Cli {
        msg: String::from(args[0].trim()),
        params: args,
        twitch_client_id,
        twitch_client_secret,
        twitch_client_token,
    };
    for param in &cli.params {
        let full_param: Vec<&str> = param.split("=").collect();
        match full_param[0] {
            "info-user" => {
                if full_param.len() == 2usize {
                    let uid: u32 = full_param[1].parse().unwrap();
                    let user = cli.get_user(uid).await?;
                    println!("{:?}", &user);
                } else {
                    println!("Please, give userid like this : info-user=5555469");
                }
            }
            "isonlive-user" => {
                if full_param.len() == 2usize {
                    let uid: u32 = full_param[1].parse().unwrap();
                    let stream = cli.get_stream_user(uid).await?;
                    println!("{:?}", &stream);
                } else {
                    println!("Please, give userid like this : isonlive-user=5555469");
                }
            }
            "token" => {
                if full_param.len() == 1usize {
                    let token = cli.get_token(TokenOption::None).await?;
                    println!("{}", &token);
                } else if full_param.len() == 2usize {
                    let scopes: Vec<String> =
                        full_param[1].split(",").map(|i| i.to_string()).collect();
                    let token = cli.get_token(TokenOption::Some(scopes)).await?;
                    println!("{}", &token);
                }
            }
            "uid" => {
                if full_param.len() == 2usize {
                    let login: String = full_param[1].parse().unwrap();
                    let user: Vec<User> = cli.get_user_by_login(login).await?;
                    let uid: &User = user.first().unwrap();
                    println!("{}", &uid.id);
                } else {
                    println!("Please, give login name like this : uid=zaekof");
                }
            }
            "topgames" => {
                let top_games = cli.get_top_games().await?;
                println!("{:?}", top_games);
            }
            "topgame" => {
                let top_games = cli.get_top_games().await?;
                let first_game: &TopGames = &top_games.first().unwrap();
                println!("{:?}", first_game);
            }
            "bannedevents" => {
                if full_param.len() == 2usize {
                    let bid: u32 = full_param[1].parse().unwrap();
                    let banned_events: Vec<BannedEvents> = cli.get_banned_events(bid).await?;
                    println!("{:?}", &banned_events);
                } else {
                    println!("Please, give broadcaster id like this : bannedevents=198704263");
                }
            }
            "help" => {
                println!("Please, see the readme file for help.");
            }
            _ => {}
        };
    }

    if cli.params.len() <= 1 {
        println!("Don't forget to give me some parameters..");
    }

    Ok(())
}
