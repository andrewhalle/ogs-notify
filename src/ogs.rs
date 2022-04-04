use std::fs::File;
use std::io::{BufReader, ErrorKind};
use std::path::Path;

use anyhow::anyhow;
use cookie_store::CookieStore;
use serde::{Deserialize, Serialize};
use ureq::AgentBuilder;

use crate::utils::{prompt, prompt_password};

const LOG_IN_URL: &'static str = "https://online-go.com/api/v0/login";
const OVERVIEW_URL: &'static str = "https://online-go.com/api/v1/ui/overview";
const ME_URL: &'static str = "https://online-go.com/api/v1/me";

#[derive(Serialize)]
pub struct LoginRequest<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(Deserialize)]
struct OverviewResponse {
    active_games: Vec<Game>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct User {
    pub id: u32,
    pub username: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Game {
    pub id: u32,
    pub black: User,
    pub white: User,
    pub json: GameJson,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GameJson {
    pub clock: GameClock,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GameClock {
    pub current_player: u32,
}

pub struct OgsAgent(ureq::Agent);

impl OgsAgent {
    pub fn new(cookie_file: &Path) -> anyhow::Result<Self> {
        match File::open(cookie_file) {
            // the cookie file does not exist, attempt to prompt for username/password
            Err(e) if matches!(e.kind(), ErrorKind::NotFound) => {
                let username = prompt("username")?;
                let password = prompt_password("password")?;
                let login_request = LoginRequest {
                    username: &username,
                    password: &password,
                };

                let agent = ureq::agent();
                let resp = agent.post(LOG_IN_URL).send_json(login_request).unwrap();
                if resp.status() != 200 {
                    return Err(anyhow!("Login request did not return 200."));
                }

                let mut file = File::create(cookie_file).unwrap();
                agent.cookie_store().save_json(&mut file).unwrap();

                Ok(OgsAgent(agent))
            }
            // the cookie file does exist, load an agent with that file as the cookie jar
            Ok(cookie_file) => {
                let read = BufReader::new(cookie_file);
                let cookies = CookieStore::load_json(read).unwrap();
                let agent = AgentBuilder::new().cookie_store(cookies).build();

                Ok(OgsAgent(agent))
            }
            // Something else went wrong, return the error
            Err(e) => {
                return Err(anyhow!(e));
            }
        }
    }

    pub fn logged_in_user(&self) -> anyhow::Result<User> {
        self.0
            .get(ME_URL)
            .call()
            .map_err(|e| anyhow!(e))
            .and_then(|response| response.into_json::<User>().map_err(|e| anyhow!(e)))
    }

    pub fn active_games(&self) -> anyhow::Result<Vec<Game>> {
        self.0
            .get(OVERVIEW_URL)
            .call()
            .map_err(|e| anyhow!(e))
            .and_then(|response| {
                response
                    .into_json::<OverviewResponse>()
                    .map_err(|e| anyhow!(e))
            })
            .map(|response| response.active_games)
    }
}

impl Game {
    pub fn other_user(&self, logged_in_user: &User) -> &User {
        if logged_in_user.id == self.black.id {
            &self.white
        } else {
            &self.black
        }
    }
}
