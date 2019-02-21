use std::env;

use serde::ser::Serialize;
use serde::de::DeserializeOwned;

use reqwest;

use error::Res;
use auth::{AuthSource, AuthProvider};
use user::{User, Org, UserRecord, OrgRecord};

pub static GITLAB_CLIENT_ID: &'static str = "05568e094f02af3b1593fe1b7e6f6651684885968232d87812334d8b74deb995";



#[derive(Serialize)]
struct OAuthResponse {
    client_id: String,
    client_secret: String,
    code: String,
    grant_type: String,
    redirect_uri: String,
}

#[derive(Deserialize, Debug)]
pub struct OAuthToken {
    pub access_token: String,
    pub token_type: String,
}

#[derive(Deserialize, Debug)]
pub struct GitlabUser {
    username: String,
    id: usize,
    email: String,
    avatar_url: String,
    // name: String,
}

#[derive(Deserialize, Debug)]
pub struct GitlabGroup {
    path: String,
    id: usize,
    // web_url: String,
    // description: String,
    // avatar_url: String,
}



pub struct Gitlab {
    http: reqwest::Client,
}

impl Gitlab {
    pub fn new() -> Res<Self> {
        Ok(Gitlab { http: reqwest::Client::new() })
    }

    #[allow(dead_code)]
    fn post<A, B>(&self, url: &str, token: &str, payload: &A) -> Res<B>
    where
        A: Serialize,
        B: DeserializeOwned,
    {
        Ok(self.http
            .post(&format!("https://gitlab.com/api/v4/{}", url))
            .header("Accept", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .form(payload)
            .send()?
            .json()?)
    }

    fn get<B>(&self, url: &str, token: &str) -> Res<B>
    where
        B: DeserializeOwned,
    {
        Ok(self.http
            .get(&format!("https://gitlab.com/api/v4/{}", url))
            .header("Accept", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .send()?
            .json()?)
    }

    pub fn validate_callback(&self, code: &str) -> Res<OAuthToken> {
        Ok(self.http
            .post("https://gitlab.com/oauth/token")
            .header("Accept", "application/json")
            .form(&OAuthResponse {
                client_id: GITLAB_CLIENT_ID.to_string(),
                client_secret: env::var("GITLAB_SECRET")?,
                code: code.to_string(),
                grant_type: "authorization_code".to_string(),
                redirect_uri: "http://localhost:8000/gitlab/callback".to_string(),
            })
            .send()?
            .json()?)
    }
}

impl AuthProvider for Gitlab {
    fn user(&self, token: &str) -> Res<UserRecord> {
        let user: GitlabUser = self.get("user", token)?;
        Ok(UserRecord::new(
            &User {
                provider: AuthSource::Gitlab,
                id: format!("{}", user.id),
            },
            &user.username,
            &user.email,
            &user.avatar_url,
        ))
    }

    fn orgs(&self, token: &str) -> Res<Box<Iterator<Item = OrgRecord>>> {
        let orgs: Vec<GitlabGroup> = self.get("groups", token)?;
        Ok(Box::new(orgs.into_iter().map(|org| {
            OrgRecord {
                id: Org {
                    provider: AuthSource::Gitlab,
                    id: format!("{}", org.id),
                },
                name: org.path,
            }
        })))
    }
}
