use crate::common::{BulkResult, Pagination};
use crate::{enum_values, options, EmptyJsonProxy, Executor, Locale, Request};
use crate::{MovieListing, Result, Series};
use chrono::{DateTime, Utc};
use futures_util::FutureExt;
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use serde_json::json;
use std::sync::Arc;

enum_values! {
    /// Starts a rating can have. Crunchyroll does not use simple numbers which would be much easier
    /// to work with but own names for every star.
    pub enum RatingStar {
        OneStar = "1s"
        TwoStars = "2s"
        ThreeStars = "3s"
        FourStars = "4s"
        FiveStars = "5s"
    }
}

/// Details about a star rating of [`Rating`].
#[derive(Clone, Debug, Default, Deserialize)]
pub struct RatingStarDetails {
    /// The amount of user ratings.
    pub displayed: String,
    /// If [`RatingStarDetails::displayed`] is > 1000 it gets converted from a normal integer to a
    /// float. E.g. 1700 becomes 1.7. [`RatingStarDetails::unit`] is then `K` (= representing
    /// thousand). If its < 1000, [`RatingStarDetails::unit`] is just an empty string.
    pub unit: String,

    /// How many percent of user voted this star. Only populated if this struct is obtained via
    /// [`Rating`].
    pub percentage: Option<u8>,
}

/// Overview about rating statistics for a series or movie listing.
#[derive(Clone, Debug, Default, Deserialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Rating {
    #[serde(alias = "1s")]
    pub one_star: RatingStarDetails,
    #[serde(alias = "2s")]
    pub two_stars: RatingStarDetails,
    #[serde(alias = "3s")]
    pub three_stars: RatingStarDetails,
    #[serde(alias = "4s")]
    pub four_stars: RatingStarDetails,
    #[serde(alias = "5s")]
    pub five_stars: RatingStarDetails,

    pub total: u32,
    #[serde(deserialize_with = "crate::internal::serde::deserialize_try_from_string")]
    pub average: f64,

    #[serde(deserialize_with = "crate::internal::serde::deserialize_empty_pre_string_to_none")]
    pub rating: Option<RatingStar>,
}

/// Ratings for a review a user has made about a series or movie listing.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct ReviewRatings {
    pub yes: RatingStarDetails,
    pub no: RatingStarDetails,
    pub total: u32,

    #[serde(rename = "rating")]
    #[serde(deserialize_with = "deserialize_rating_to_bool")]
    pub helpful: Option<bool>,
}

/// Content of a review a user has made about a series or movie listing.
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct ReviewContent {
    pub id: String,

    pub title: String,
    pub body: String,

    pub language: Locale,
    pub spoiler: bool,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub created_at: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub modified_at: DateTime<Utc>,

    pub authored_reviews: u32,
}

/// Author of a review.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct ReviewAuthor {
    pub id: String,

    pub username: String,

    pub avatar: String,
}

/// A review a user has made about a series or movie listing.
#[derive(Clone, Debug, Default, Deserialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Review {
    #[serde(skip)]
    executor: Arc<Executor>,

    pub review: ReviewContent,

    pub author_rating: RatingStar,
    pub author: ReviewAuthor,

    pub ratings: ReviewRatings,

    pub reported: bool,
}

impl Review {
    /// Mark a review as helpful. A review can only be marked once as helpful (or not). If
    /// [`Review::review.helpful`] is [`Some`], a review were already made.
    pub async fn mark_helpful(&mut self, helpful: bool) -> Result<()> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content-reviews/v2/user/{}/rating/review/{}",
            self.executor.details.account_id.clone()?,
            self.review.id
        );
        let rating = if helpful { "yes" } else { "no" };
        self.executor
            .put(endpoint)
            .json(&json!({ "rating": rating }))
            .request()
            .await?;
        self.ratings.helpful = Some(helpful);
        Ok(())
    }

    /// Report this review. You can report or unreport it, use the function parameter to control it.
    /// See [`Review::reported`] if the comment was already reported from your account or not.
    pub async fn report(&mut self, report: bool) -> Result<()> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content-reviews/v2/user/{}/report/review/{}",
            self.executor.details.account_id.clone()?,
            self.review.id
        );
        let builder = if report {
            self.executor.put(endpoint)
        } else {
            self.executor.delete(endpoint)
        };
        builder.json(&json!({})).request::<EmptyJsonProxy>().await?;
        self.reported = report;
        Ok(())
    }
}

/// Review which were made by your account.
#[derive(Clone, Debug, Default, Deserialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct SelfReview {
    #[serde(skip)]
    executor: Arc<Executor>,
    #[serde(skip)]
    endpoint: String,

    pub review: ReviewContent,

    pub author_rating: RatingStar,
    pub author: ReviewAuthor,

    pub ratings: ReviewRatings,
}

impl SelfReview {
    /// Edit your review.
    pub async fn edit<S: AsRef<str>>(&mut self, title: S, body: S, spoiler: bool) -> Result<()> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content-reviews/v2/{}/user/{}/rating/{}/{}",
            self.executor.details.account_id.clone()?,
            self.executor.details.locale,
            self.endpoint,
            self.review.id
        );
        *self = self
            .executor
            .patch(endpoint)
            .json(&json!({"title": title.as_ref(), "body": body.as_ref(), "spoiler": spoiler}))
            .request()
            .await?;

        Ok(())
    }

    /// Delete your review.
    pub async fn delete(&self) -> Result<()> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content-reviews/v2/{}/user/{}/rating/{}/{}",
            self.executor.details.account_id.clone()?,
            self.executor.details.locale,
            self.endpoint,
            self.review.id
        );
        self.executor.delete(endpoint).request().await
    }
}

fn deserialize_rating_to_bool<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    match value.as_str() {
        "yes" => Ok(Some(true)),
        "no" => Ok(Some(false)),
        "" => Ok(None),
        _ => Err(Error::custom(format!(
            "could not deserialize rating value '{value}'"
        ))),
    }
}

enum_values! {
    pub enum ReviewSortType {
        Newest = "newest"
        Oldest = "oldest"
        Helpful = "helpful"
    }
}

options! {
    ReviewOptions;
    sort(ReviewSortType, "sort") = Some(ReviewSortType::Helpful),
    filter(RatingStar, "filter") = None
}

macro_rules! impl_rating {
    ($($s:path = $endpoint:literal),*) => {
        $(
            impl $s {
                pub async fn rating(&self) -> Result<Rating> {
                    let endpoint = format!(
                        "https://www.crunchyroll.com/content-reviews/v2/user/{}/rating/{}/{}",
                        self.executor.details.account_id.clone()?, $endpoint, self.id
                    );
                    self.executor.get(endpoint).request().await
                }

                pub fn reviews(&self, options: ReviewOptions) -> Result<Pagination<Review>> {
                    Ok(Pagination::new(|options| {
                        async move {
                            let endpoint = format!(
                                "https://www.crunchyroll.com/content-reviews/v2/{}/user/{}/review/{}/{}/list",
                                options.extra.get("locale").unwrap(), options.extra.get("account_id").unwrap(), $endpoint, options.extra.get("id").unwrap()
                            );
                            let result: BulkResult<Review> = options
                                .executor
                                .get(endpoint)
                                .query(&[("page", options.page), ("page_size", options.page_size)])
                                .query(&options.query)
                                .request()
                                .await?;
                            Ok((result.items, result.total))
                        }
                        .boxed()
                    }, self.executor.clone(), Some(options.into_query()), Some(vec![("locale", self.executor.details.locale.to_string()), ("account_id", self.executor.details.account_id.clone()?), ("id", self.id.clone())])))
                }

                pub async fn rate(&self, stars: RatingStar) -> Result<Rating> {
                    let endpoint = format!(
                        "https://www.crunchyroll.com/content-reviews/v2/user/{}/rating/{}/{}",
                        self.executor.details.account_id.clone()?, $endpoint, self.id
                    );
                    self.executor.put(endpoint)
                        .json(&json!({"rating": stars}))
                        .request()
                        .await
                }

                pub async fn create_review<S: AsRef<str>>(&self, title: S, body: S, spoiler: bool) -> Result<SelfReview> {
                    let endpoint = format!(
                        "https://www.crunchyroll.com/content-reviews/v2/user/{}/rating/{}/{}",
                        self.executor.details.account_id.clone()?, $endpoint, self.id
                    );
                    self.executor.post(endpoint)
                        .json(&json!({"title": title.as_ref(), "body": body.as_ref(), "spoiler": spoiler}))
                        .request()
                        .await
                }

                pub async fn self_review(&self) -> Result<SelfReview> {
                    let endpoint = format!(
                        "https://www.crunchyroll.com/content-reviews/v2/{}/user/{}/rating/{}/{}",
                        self.executor.details.account_id.clone()?, self.executor.details.locale, $endpoint, self.id
                    );
                    let mut self_review: SelfReview = self.executor.get(endpoint).request().await?;
                    self_review.endpoint = stringify!($endpoint).to_string();

                    Ok(self_review)
                }
            }
        )*
    }
}

impl_rating! {
    Series = "series",
    MovieListing = "movie_listing"
}
