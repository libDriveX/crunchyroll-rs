pub mod browse {
    use crate::common::{BulkResult, Panel};
    use crate::error::Result;
    use crate::media_collection::MediaType;
    use crate::{enum_values, options, Crunchyroll};

    enum_values! {
        BrowseSortType,
        #[derive(Debug)],
        Popularity = "popularity",
        NewlyAdded = "newly_added",
        Alphabetical = "alphabetical"
    }

    options! {
        BrowseOptions,
        categories(Vec<String>, "categories") = None,
        // Specifies whether the entries should be dubbed.
        is_dubbed(bool, "is_dubbed") = None,
        // Specifies whether the entries should be subbed.
        is_subbed(bool, "is_subbed") = None,
        // Specifies a particular simulcast season by id in which the entries have been aired.
        simulcast(String, "season_tag") = None,
        // Specifies how the entries should be sorted.
        sort(BrowseSortType, "sort") = Some(BrowseSortType::NewlyAdded),
        // Specifies the media type of the entries.
        media_type(MediaType, "type") = None,

        // Limit of results to return.
        limit(u32, "n") = Some(20),
        // Specifies the index from which the entries should be returned.
        start(u32, "start") = None
    }

    impl Crunchyroll {
        pub async fn browse(&self, options: BrowseOptions) -> Result<BulkResult<Panel>> {
            let executor = self.executor.clone();

            let endpoint = "https://beta.crunchyroll.com/content/v1/browse";
            let builder = executor.client.get(endpoint).query(&options.to_query(&[(
                "locale".to_string(),
                self.executor.details.locale.to_string(),
            )]));

            executor.request(builder).await
        }
    }
}

pub mod query {
    use crate::common::Request;
    use crate::error::{CrunchyrollError, CrunchyrollErrorContext, Result};
    use crate::{enum_values, options, BulkResult, Collection, Crunchyroll, Executor};
    use serde::Deserialize;
    use std::sync::Arc;

    #[derive(Debug, Deserialize, Default)]
    #[serde(try_from = "BulkResult<QueryBulkResult>")]
    #[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
    pub struct QueryResults {
        #[serde(skip)]
        executor: Arc<Executor>,

        pub top_results: Option<BulkResult<Collection>>,
        pub series: Option<BulkResult<Collection>>,
        pub movie_listing: Option<BulkResult<Collection>>,
        pub episode: Option<BulkResult<Collection>>,
    }

    impl Request for QueryResults {
        fn __set_executor(&mut self, executor: Arc<Executor>) {
            self.executor = executor.clone();

            if let Some(top_results) = &mut self.top_results {
                for collection in top_results.items.iter_mut() {
                    collection.__set_executor(executor.clone());
                }
            }
            if let Some(series) = &mut self.series {
                for collection in series.items.iter_mut() {
                    collection.__set_executor(executor.clone());
                }
            }
            if let Some(movie_listing) = &mut self.movie_listing {
                for collection in movie_listing.items.iter_mut() {
                    collection.__set_executor(executor.clone());
                }
            }
            if let Some(episode) = &mut self.episode {
                for collection in episode.items.iter_mut() {
                    collection.__set_executor(executor.clone());
                }
            }
        }

        fn __get_executor(&self) -> Option<Arc<Executor>> {
            Some(self.executor.clone())
        }
    }

    impl TryFrom<BulkResult<QueryBulkResult>> for QueryResults {
        type Error = CrunchyrollError;

        fn try_from(value: BulkResult<QueryBulkResult>) -> std::result::Result<Self, Self::Error> {
            let mut top_results: Option<BulkResult<Collection>> = None;
            let mut series: Option<BulkResult<Collection>> = None;
            let mut movie_listing: Option<BulkResult<Collection>> = None;
            let mut episode: Option<BulkResult<Collection>> = None;

            for item in value.items.clone() {
                let result = BulkResult {
                    items: item.items,
                    total: item.total,
                };
                match item.result_type.as_str() {
                    "top_results" => top_results = Some(result),
                    "series" => series = Some(result),
                    "movie_listing" => movie_listing = Some(result),
                    "episode" => episode = Some(result),
                    _ => {
                        return Err(CrunchyrollError::Decode(
                            CrunchyrollErrorContext::new(format!(
                                "invalid result type found: '{}'",
                                item.result_type
                            ))
                            .with_value(format!("{:?}", value).as_bytes()),
                        ))
                    }
                };
            }

            Ok(Self {
                executor: Default::default(),
                top_results,
                series,
                movie_listing,
                episode,
            })
        }
    }

    #[derive(Clone, Debug, Default, Deserialize, Request)]
    #[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
    #[cfg_attr(not(feature = "__test_strict"), serde(default))]
    struct QueryBulkResult {
        #[serde(rename = "type")]
        result_type: String,
        items: Vec<Collection>,
        total: u32,
    }

    enum_values! {
        QueryType,
        #[derive(Debug)],
        Series = "series",
        MovieListing = "movie_listing",
        Episode = "episode"
    }

    options! {
        QueryOptions,
        limit(u32, "n") = Some(20),
        result_type(QueryType, "type") = None
    }

    impl Crunchyroll {
        pub async fn query(&self, query: String, options: QueryOptions) -> Result<QueryResults> {
            let executor = self.executor.clone();

            let endpoint = "https://beta.crunchyroll.com/content/v1/search";
            let builder = executor.client.get(endpoint).query(&options.to_query(&[
                ("q".to_string(), query.clone()),
                (
                    "locale".to_string(),
                    self.executor.details.locale.to_string(),
                ),
            ]));

            executor.request(builder).await
        }
    }
}
