use crate::{base62::Base62, endpoints, query::SearchParams};
use chrono::{DateTime, Utc};
use getset::Getters;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

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
    error: Option<endpoints::Error>,
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
                endpoints::get_search(&params, self.token)
            } else {
                endpoints::get_search(&self.params, self.token)
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