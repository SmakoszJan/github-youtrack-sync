use std::{fmt::Display, sync::Arc};

use http::{HeaderMap, HeaderValue, header};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    InvalidHeaderValue(http::header::InvalidHeaderValue),
}

impl From<http::header::InvalidHeaderValue> for Error {
    fn from(v: http::header::InvalidHeaderValue) -> Self {
        Self::InvalidHeaderValue(v)
    }
}

impl From<reqwest::Error> for Error {
    fn from(v: reqwest::Error) -> Self {
        Self::Reqwest(v)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reqwest(err) => err.fmt(f),
            Self::InvalidHeaderValue(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct YouTrack {
    host: Arc<Url>,
    client: Client,
}

impl YouTrack {
    #[must_use]
    pub fn new(host: Url, token: impl Into<String>) -> Result<Self> {
        let mut headers = HeaderMap::new();

        headers.insert(header::ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token.into()))?,
        );

        Ok(Self {
            client: Client::builder().default_headers(headers).build()?,
            host: Arc::new(host),
        })
    }

    /// Returns the first page of candidates for a given query
    pub async fn find_project(&self, query: &str) -> Result<Vec<Project>> {
        #[derive(Deserialize)]
        struct FindProjectResponse(Vec<FindProjectItem>);

        #[derive(Deserialize)]
        struct FindProjectItem {
            id: Arc<str>,
            name: String,
        }

        #[derive(Serialize)]
        struct FindParams {
            query: String,
        }

        let mut get = self
            .client
            .get(self.host.join("api/admin/projects?fields=id,name").unwrap());

        if !query.is_empty() {
            get = get.query(&FindParams {
                query: query.replace(" ", "+"),
            });
        }
        let response = get.send().await.expect(":(");
        Ok(response
            .json::<FindProjectResponse>()
            .await?
            .0
            .into_iter()
            .map(|v| Project {
                youtrack: self.clone(),
                id: ProjectId { id: v.id },
                name: v.name,
            })
            .collect())
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ProjectId {
    id: Arc<str>,
}

pub struct Project {
    youtrack: YouTrack,
    id: ProjectId,
    name: String,
}

impl Project {
    #[must_use]
    pub fn id(&self) -> &ProjectId {
        &self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    pub async fn create_issue(&self, project: ProjectId, issue: &IssueData) -> Result<IssueId> {
        #[derive(Serialize)]
        struct CreateIssue<'r> {
            project: ProjectId,
            #[serde(flatten)]
            data: &'r IssueData,
        }

        #[derive(Deserialize)]
        struct CreateIssueResponse {
            id: IssueId,
        }

        let response = self
            .youtrack
            .client
            .post(self.youtrack.host.join("api/issues").unwrap())
            .json(&CreateIssue {
                project,
                data: issue,
            })
            .send()
            .await?;

        Ok(response.json::<CreateIssueResponse>().await?.id)
    }

    pub async fn update_issue(&self, issue_id: &IssueId, issue: &IssueData) -> Result<()> {
        self.youtrack
            .client
            .post(
                self.youtrack
                    .host
                    .join("api/issues/")
                    .unwrap()
                    .join(&issue_id.0)
                    .unwrap(),
            )
            .json(issue)
            .send()
            .await?;

        Ok(())
    }
}

/// Data used to create an issue.
#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct IssueData {
    pub summary: String,
    pub description: Option<String>,
    pub custom_fields: Vec<CustomField>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct IssueId(String);

#[derive(Serialize)]
pub struct CustomField {
    pub name: String,
    #[serde(rename = "$type")]
    pub type_: String,
    /// This should either be an enum or custom field should be generic
    /// but that's outside the scope
    pub value: StateBundleElement,
}

#[derive(Serialize)]
pub struct StateBundleElement {
    pub name: String,
}
