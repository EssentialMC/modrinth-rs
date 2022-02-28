use super::{get, Error, Result};
use crate::{base62::Base62, query_string::JsonQueryParams};
use chrono::{DateTime, Utc};
use derive_more::Display;
use getset::Getters;
use serde::{Deserialize, Serialize};
use serde_with::SerializeDisplay;
use std::collections::VecDeque;

pub fn get_search(params: &SearchParams, token: Option<&str>) -> Result<SearchResults> {
    get(
        &format!(
            "https://api.modrinth.com/v2/search?{}",
            &params.to_query_string()
        ),
        token,
    )
}

pub fn get_search_iter(params: SearchParams, token: Option<&str>) -> SearchResultsPaginator {
    SearchResultsPaginator::new(params, token)
}

#[derive(Debug, Clone, Display, SerializeDisplay)]
pub enum SearchFacet {
    #[display(fmt = "categories:'{}'", _0)]
    Category(String),
    #[display(fmt = "versions:'{}'", _0)]
    Version(String),
    #[display(fmt = "license:'{}'", _0)]
    License(String),
    #[display(fmt = "project_type:'{}'", _0)]
    ProjectType(String),
}

impl SearchFacet {
    pub fn category<S>(value: S) -> Self
    where
        S: AsRef<str>,
    {
        Self::Category(value.as_ref().to_owned())
    }

    pub fn version<S>(value: S) -> Self
    where
        S: AsRef<str>,
    {
        Self::Version(value.as_ref().to_owned())
    }

    pub fn license<S>(value: S) -> Self
    where
        S: AsRef<str>,
    {
        Self::License(value.as_ref().to_owned())
    }

    pub fn project_type<S>(value: S) -> Self
    where
        S: AsRef<str>,
    {
        Self::ProjectType(value.as_ref().to_owned())
    }
}

pub type SearchFacets = Vec<Vec<SearchFacet>>;

#[derive(Debug, Clone, Serialize)]
pub enum SearchIndex {
    Relevance,
    Downloads,
    Follows,
    Newest,
    Updated,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct SearchParams {
    pub query: Option<String>,
    /// <https://docs.modrinth.com/docs/tutorials/api_search/#facets>
    pub facets: Option<SearchFacets>,
    pub index: Option<SearchIndex>,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
    // filters: Option<SearchFilters>,
}

impl JsonQueryParams<'_> for SearchParams {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub hits: VecDeque<ProjectResult>,
    pub offset: usize,
    pub limit: usize,
    pub total_hits: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectResult {
    pub project_id: Base62,
    pub project_type: String,
    pub slug: Option<String>,
    pub author: String,
    pub title: String,
    pub description: String,
    pub categories: Vec<String>,
    pub versions: Vec<String>,
    // Should `downloads` and `follows`be a usize but the API returns -1 sometimes
    // Reference:
    // > `labrinth::models::projects::Project` and
    // > `labrinth::database::models::project_item::Project`
    pub downloads: isize,
    pub follows: isize,
    pub icon_url: String,
    pub date_created: DateTime<Utc>,
    pub date_modified: DateTime<Utc>,
    pub license: String,
    pub client_side: String,
    pub server_side: String,
    pub gallery: Vec<String>,
}

#[derive(Debug, Getters)]
pub struct SearchResultsPaginator<'a> {
    params: SearchParams,
    token: Option<&'a str>,
    results: VecDeque<ProjectResult>,
    total_hits: Option<usize>,
    #[getset(get = "pub")]
    error: Option<Error>,
}

impl<'a> SearchResultsPaginator<'a> {
    pub fn new(params: SearchParams, token: Option<&'a str>) -> Self {
        Self {
            params,
            token,
            results: VecDeque::new(),
            total_hits: None,
            error: None,
        }
    }
}

impl<'a> Iterator for SearchResultsPaginator<'a> {
    type Item = ProjectResult;

    fn next(&mut self) -> Option<Self::Item> {
        if self.results.is_empty() {
            let results = if self.total_hits.is_none() {
                let mut params = self.params.clone();
                params.limit = Some(1);
                get_search(&params, self.token)
            } else {
                get_search(&self.params, self.token)
            };

            let mut results = match results {
                Ok(result) => result,
                Err(error) => {
                    self.error = Some(error);
                    return None;
                }
            };

            if self.total_hits.is_none() {
                self.total_hits = Some(results.total_hits);
            }

            self.results.append(&mut results.hits);
            self.params.offset = Some(self.params.offset.unwrap_or(0) + self.results.len());
        }

        self.results.pop_front()
    }

    /// Requires one item to have been recieved with [`Self::next`],
    /// otherwise the upper bound will be `None`.
    /// This cannot be precomputed because this method cannot have mutable access to `self`
    /// and therefore cannot process the results of an initial get request.
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.total_hits)
    }
}
