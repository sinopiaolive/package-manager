use std::env;

use serde::ser::Serialize;
use serde::de::DeserializeOwned;

use reqwest;
use reqwest::header::{Accept, Authorization, qitem};
use reqwest::mime::APPLICATION_JSON;

use error::Res;
use auth::{AuthSource, AuthProvider};
use user::{User, Org};

pub static GITHUB_CLIENT_ID: &'static str = "a009958d6b555fa8c1f7";



#[derive(Serialize)]
struct OAuthResponse {
    client_id: String,
    client_secret: String,
    code: String,
}

#[derive(Deserialize, Debug)]
pub struct OAuthToken {
    pub access_token: String,
    pub scope: String,
    pub token_type: String,
}

#[derive(Deserialize, Debug)]
pub struct GithubUser {
    login: String,
    id: usize,
    // name: String,
    // email: String,
    // avatar_url: String,
}

#[derive(Deserialize, Debug)]
pub struct GithubOrg {
    login: String,
    id: usize,
    // url: String,
    // description: String,
    // avatar_url: String,
}



pub struct Github {
    http: reqwest::Client,
}

impl Github {
    pub fn new() -> Res<Self> {
        Ok(Github { http: reqwest::Client::new()? })
    }

    #[allow(dead_code)]
    fn post<A, B>(&self, url: &str, token: &str, payload: &A) -> Res<B>
    where
        A: Serialize,
        B: DeserializeOwned,
    {
        Ok(self.http
            .post(&format!("https://api.github.com/{}", url))?
            .header(Accept(vec![qitem(APPLICATION_JSON)]))
            .header(Authorization(format!("token {}", token)))
            .form(payload)?
            .send()?
            .json()?)
    }

    fn get<B>(&self, url: &str, token: &str) -> Res<B>
    where
        B: DeserializeOwned,
    {
        Ok(self.http
            .get(&format!("https://api.github.com/{}", url))?
            .header(Accept(vec![qitem(APPLICATION_JSON)]))
            .header(Authorization(format!("token {}", token)))
            .send()?
            .json()?)
    }

    pub fn validate_callback(&self, code: &str) -> Res<OAuthToken> {
        Ok(self.http
            .post("https://github.com/login/oauth/access_token")?
            .header(Accept(vec![qitem(APPLICATION_JSON)]))
            .form(&OAuthResponse {
                client_id: GITHUB_CLIENT_ID.to_string(),
                client_secret: env::var("GITHUB_SECRET")?,
                code: code.to_string(),
            })?
            .send()?
            .json()?)
    }
}

impl AuthProvider for Github {
    fn user(&self, token: &str) -> Res<User> {
        let user: GithubUser = self.get("user", token)?;
        Ok(User {
            provider: AuthSource::Github,
            id: format!("{}", user.id),
        })
    }

    fn orgs(&self, token: &str) -> Res<Box<Iterator<Item = Org>>> {
        let orgs: Vec<GithubOrg> = self.get("user/orgs", token)?;
        Ok(Box::new(orgs.into_iter().map(|org| {
            Org {
                provider: AuthSource::Github,
                id: format!("{}", org.id),
            }
        })))
    }
}
