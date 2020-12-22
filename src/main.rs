use url::{Url};
use github_rs::client::{Executor, Github};
use serde::{Deserialize, Serialize};
use std::env;

fn main() {
    let mut args = env::args();
    args.next();
    let url = args.next().expect("not 1st arg");
    let token = env::var("GITHUB_API_TOKEN").expect("not env GITHUB_API_TOKEN");
    // TODO コマンドライン引数から取得
    let pull_request_id = parse(&url).unwrap();
    let pull = PullRequest::new(&token, &pull_request_id);

    match pull {
        Ok(pull) => println!("{}", pull.summary()),
        Err(e) => println!("{}", e),
    }
}

fn parse(url: &str) -> Result<PullRequestID, String> {
    let pull_url = Url::parse(url).map_err( |_e| format!("invalid url:  {}", url))?;
    let mut paths = pull_url.path_segments().ok_or("url pull request parse error")?;
    let owner: &str = paths.next().ok_or("non owner: pull request url")?;
    let repository: &str = paths.next().ok_or("non repository: pull request url")?;
    let pull = paths.next().ok_or("non pull path: pull request url")?;
    if !pull.eq("pull") {
        return Err("non pull request url".to_owned());
    }
    let id: &str = paths.next().ok_or("non pull request id: pull request url")?;
    Ok(PullRequestID{
        owner: owner.to_owned(),
        repository: repository.to_owned(),
        id: id.to_owned(),
    })
}

struct PullRequestID{
    owner: String,
    repository: String,
    id: String,
}

#[derive(Serialize, Deserialize)]
struct PullRequest {
    title: String,
    html_url: String,
    additions: u32,
    deletions: u32,
    changed_files: u32,
}

impl PullRequest {
    fn new(
        token: &str,
        pull_request_id: &PullRequestID,
        // FIXME Errorオブジェクトをつくる
    ) -> Result<PullRequest, String> {
        let client = Github::new(token).map_err( |_| "not create github client")?;
        let (_, _, pull_request) = client
            .get()
            .repos()
            .owner(&pull_request_id.owner)
            .repo(&pull_request_id.repository)
            .pulls()
            .number(&pull_request_id.id)
            .execute::<PullRequest>().map_err(|_| "not get pull request")?;
        if let Some(p) = pull_request {
            Ok(p)
        } else {
            Err("no pull request".to_string())
        }
    }

    fn summary(&self) -> String {
        format!(
            "{} (+{},-{}) changed files {}\n{}",
            self.title, self.additions, self.deletions, self.changed_files, self.html_url
        )
    }
}
