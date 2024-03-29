use derive_more::Display;
use serde::Serialize;
use serde_with::SerializeDisplay;
use strum::EnumString;

#[derive(Debug, Display, Clone, PartialEq, Eq, Hash, SerializeDisplay)]
pub enum ProjectIdentifier {
    #[display(fmt = "{}", "base62::encode(*_0)")]
    Id(u64),
    #[display(fmt = "{}", _0)]
    Slug(String),
}

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct ProjectSearchParams {
    pub query: Option<String>,
    /// <https://docs.modrinth.com/docs/tutorials/api_search/#facets>
    pub facets: Option<SearchFilters<SearchFacet>>,
    pub index: Option<SearchIndex>,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
    pub filters: Option<SearchFilters<String>>,
}

#[derive(Debug, Clone, PartialEq, SerializeDisplay)]
pub struct SearchFilters<T>(Vec<Vec<T>>);

impl<T> std::fmt::Display for SearchFilters<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&serde_json::to_string(self).unwrap())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, SerializeDisplay)]
pub enum SearchFacet {
    #[display(fmt = "categories:'{}'", _0)]
    Category(String),
    #[display(fmt = "versions:'{}'", _0)]
    Version(String),
    #[display(fmt = "license:'{}'", _0)]
    License(String),
    #[display(fmt = "project_type:'{}'", _0)]
    ProjectType(String),
    #[display(fmt = "{}:'{}'", _0, _1)]
    Custom(String, String),
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

    pub fn custom<N, S>(name: N, value: S) -> Self
    where
        N: AsRef<str>,
        S: AsRef<str>,
    {
        Self::Custom(name.as_ref().to_owned(), value.as_ref().to_owned())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumString, Serialize)]
#[serde(deny_unknown_fields)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SearchIndex {
    Relevance,
    Downloads,
    Follows,
    Newest,
    Updated,
}
