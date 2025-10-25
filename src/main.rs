use std::{
    collections::{HashMap, VecDeque},
    time::Duration,
};

use anyhow::Context;
use clap::Parser;
use http::{HeaderMap, StatusCode, Uri};
use octocrab::{
    FromResponse, Octocrab, Page,
    etag::EntityTag,
    models::{
        self, IssueEventId, IssueState,
        issues::{Issue, IssueStateReason},
    },
};
use reqwest::Url;
use serde::Deserialize;
use youtrack_api::{CustomField, IssueData, StateBundleElement, YouTrack};

#[derive(Parser)]
#[command(name = "yousync")]
#[command(about = "A tool for synchronisation between GitHub and YouTrack")]
pub struct Args {
    /// Owner of the repository
    owner: String,
    /// Name of the repository
    repo: String,
    /// Youtrack host
    youtrack: Url,
    /// Project name (query)
    project: String,
}

async fn get_issues(octocrab: &Octocrab, args: &Args) -> anyhow::Result<Vec<Issue>> {
    println!("Fetching issues from the repository...");

    let mut page = octocrab
        .issues(&args.owner, &args.repo)
        .list()
        .state(octocrab::params::State::All)
        .per_page(100)
        .send()
        .await?;

    let mut issues = Vec::new();

    loop {
        issues.extend(page.take_items());
        page = match octocrab
            .get_page::<models::issues::Issue>(&page.next)
            .await?
        {
            Some(v) => v,
            None => break,
        };
    }

    Ok(issues)
}

// Different main so that result can be returned
async fn run() -> anyhow::Result<()> {
    let args = Args::parse();

    // Get github token
    let token = std::env::var("YOUSYNC_GITHUB_TOKEN")
        .map(Ok)
        .unwrap_or_else(|err| {
            println!("{err}");
            rpassword::prompt_password("GitHub Token: ")
        })?;

    let octocrab = octocrab::instance().user_access_token(token)?;

    // Fetch events for etag
    let issues_uri = format!(
        "/repos/{}/{}/issues/events?per_page=10",
        args.owner, args.repo
    );
    let issues_uri = Uri::builder()
        .scheme("https")
        .authority("api.github.com")
        .path_and_query(issues_uri)
        .build()?;
    println!("{issues_uri}");
    let response = octocrab._get(&issues_uri).await?;

    let mut etag = EntityTag::extract_from_response(&response);

    let github_issues = get_issues(&octocrab, &args).await?;

    println!("Found {} issues", github_issues.len());

    // Get youtrack token
    let token = std::env::var("YOUSYNC_YOUTRACK_TOKEN")
        .map(Ok)
        .unwrap_or_else(|err| {
            println!("{err}");
            rpassword::prompt_password("YouTrack Token: ")
        })?;

    let youtrack = YouTrack::new(args.youtrack, token)?;

    let projects = youtrack.find_project(&args.project).await.unwrap();

    let project = projects
        .first()
        .ok_or_else(|| anyhow::anyhow!("Project not found"))?;

    println!("Project found: {}", project.name());

    let mut issues = HashMap::new();
    for issue in github_issues.iter().filter(|v| v.pull_request.is_none()) {
        let id = create_issue(project, issue).await?;

        issues.insert(issue.id, id);
    }

    // Ok(())

    // Start syncing events
    println!("Sync active");
    let mut seen = VecDeque::with_capacity(20);
    loop {
        let response = octocrab
            ._get_with_headers(
                &issues_uri,
                Some({
                    let mut map = HeaderMap::new();
                    if let Some(etag) = etag {
                        EntityTag::insert_if_none_match_header(&mut map, etag)?;
                    }
                    map
                }),
            )
            .await
            .with_context(|| "Failed to fetch events with etag")?;
        etag = EntityTag::extract_from_response(&response);

        if response.status() != StatusCode::NOT_MODIFIED {
            let page: Page<AltEvent> = Page::from_response(response)
                .await
                .with_context(|| "Failed to parse response")?;
            for event in page {
                if !seen.contains(&event.id) {
                    if seen.len() == 20 {
                        seen.pop_front();
                    }

                    seen.push_front(event.id);
                    println!("Event {:?}", event.id);

                    // Update the youtrack issue if it changes.
                    // Create the issue if it doesn't already exist
                    // This won't create new issues the moment they're created
                    // (unless github automatically raises a relevant event that I haven't found).
                    // According to the docs, a field `action` with value `opened` should be available
                    // but that doesn't appear to be the case. The problem statement only refers to
                    // updating existing issues, though, so it should be fine.
                    if let Some(youtrack_id) = issues.get(&event.issue.id) {
                        if event.event == "closed"
                            || event.event == "reopened"
                            || event.event == "renamed"
                        {
                            project
                                .update_issue(youtrack_id, &create_issue_data(&event.issue))
                                .await?;
                        }
                    } else {
                        create_issue(project, &event.issue).await?;
                    }
                }
            }
        }

        std::thread::sleep(Duration::from_secs(1));
    }
}

async fn create_issue(
    project: &youtrack_api::Project,
    issue: &Issue,
) -> Result<youtrack_api::IssueId, anyhow::Error> {
    project
        .create_issue(project.id().clone(), &create_issue_data(issue))
        .await
        .with_context(|| "Failed to create an issue")
}

fn create_issue_data(issue: &Issue) -> IssueData {
    IssueData {
        summary: issue.title.clone(),
        description: issue.body.clone(),
        custom_fields: vec![CustomField {
            name: "State".to_string(),
            type_: "StateIssueCustomField".to_string(),
            value: StateBundleElement {
                name: match issue.state {
                    IssueState::Open => "Open",
                    IssueState::Closed => match issue.state_reason {
                        Some(IssueStateReason::NotPlanned) => "Won't fix",
                        Some(IssueStateReason::Reopened) => "Reopened",
                        Some(IssueStateReason::Duplicate) => "Duplicate",
                        _ => "Fixed",
                    },
                    _ => "Submitted",
                }
                .to_string(),
            },
        }],
    }
}

#[derive(Deserialize)]
struct AltEvent {
    id: IssueEventId,
    issue: Issue, // r#type: EventType,
    event: String,
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        println!("Error occurred: {err}");
    }
}
