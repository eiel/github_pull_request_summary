use github_rs::client::{Executor, Github};
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt;
use std::process;
use std::str::FromStr;
use url::Url;

#[argopt::cmd]
fn main(pull_request_url: String) {
    let token = env::var("GITHUB_API_TOKEN").expect("not env GITHUB_API_TOKEN");
    let pull_request_id = match PullRequestID::from_str(&pull_request_url) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }

    };
    let client = Github::new(token).expect("not create github client");

    let pull_request = PullRequestSummary::new(&client, &pull_request_id);
    match pull_request {
        Ok(pull) => println!("{}", pull),
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}

struct PullRequestID {
    owner: String,
    repository: String,
    id: String,
}

impl FromStr for PullRequestID {
    type Err = String;
    /**
     * URL形式の文字列を解析しPull Request の ID を取り出す
     * FIXME Errorの型を用意する
     */
    fn from_str(url: &str) -> Result<PullRequestID, String> {
        let pull_url = Url::parse(url).map_err(|_e| format!("invalid url:  {}", url))?;
        let mut paths = pull_url
            .path_segments()
            .ok_or("url pull request parse error")?;
        let owner: &str = paths.next().ok_or("non owner: pull request url")?;
        let repository: &str = paths.next().ok_or("non repository: pull request url")?;
        let pull = paths.next().ok_or("non pull path: pull request url")?;
        if !pull.eq("pull") {
            return Err("non pull request url".to_owned());
        }
        let id: &str = paths
            .next()
            .ok_or("non pull request id: pull request url")?;
        Ok(PullRequestID {
            owner: owner.to_owned(),
            repository: repository.to_owned(),
            id: id.to_owned(),
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct PullRequestSummary {
    title: String,
    html_url: String,
    additions: u32,
    deletions: u32,
    changed_files: u32,
}

impl PullRequestSummary {
    fn new(
        client: &Github,
        pull_request_id: &PullRequestID,
        // FIXME Errorオブジェクトをつくる
    ) -> Result<PullRequestSummary, String> {
        let (_, _, pull_request) = client
            .get()
            .repos()
            .owner(&pull_request_id.owner)
            .repo(&pull_request_id.repository)
            .pulls()
            .number(&pull_request_id.id)
            .execute::<PullRequestSummary>()
            .map_err(|_| "not get pull request")?;
        if let Some(p) = pull_request {
            Ok(p)
        } else {
            Err("no pull request".to_string())
        }
    }
}


impl fmt::Display for PullRequestSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "{} (+{},-{}) changed files {}\n{}",
            self.title, self.additions, self.deletions, self.changed_files, self.html_url
        )
    }
}
