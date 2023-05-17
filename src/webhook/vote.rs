use crate::snowflake;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

/// A struct representing a dispatched [Top.gg](https://top.gg) bot/server vote event.
#[must_use]
#[cfg_attr(docsrs, doc(cfg(feature = "webhook")))]
#[derive(Clone, Debug, Deserialize)]
pub struct Vote {
  /// The ID of the bot/server that received a vote.
  #[serde(
    deserialize_with = "snowflake::deserialize",
    alias = "bot",
    alias = "guild"
  )]
  pub receiver_id: u64,

  /// The ID of the user who voted.
  #[serde(deserialize_with = "snowflake::deserialize", rename = "user")]
  pub voter_id: u64,

  /// Whether this vote is just a test coming from the bot/server owner or not. Most of the time this would be `false`.
  #[serde(deserialize_with = "deserialize_is_test", rename = "type")]
  pub is_test: bool,

  /// Whether the weekend multiplier is active or not, meaning a single vote counts as two.
  /// If the dispatched event came from a server being voted, this will always be `false`.
  #[serde(default, rename = "isWeekend")]
  pub is_weekend: bool,

  /// Query strings found on the vote page.
  #[serde(default, deserialize_with = "deserialize_query_string")]
  pub query: HashMap<String, String>,
}

#[inline(always)]
fn deserialize_is_test<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
  D: Deserializer<'de>,
{
  String::deserialize(deserializer).map(|s| s == "test")
}

fn deserialize_query_string<'de, D>(deserializer: D) -> Result<HashMap<String, String>, D::Error>
where
  D: Deserializer<'de>,
{
  Ok(
    String::deserialize(deserializer)
      .map(|s| {
        let mut output = HashMap::new();

        for mut it in s.split('&').map(|pair| pair.split('=')) {
          if let (Some(k), Some(v)) = (it.next(), it.next()) {
            if let Ok(v) = urlencoding::decode(v) {
              output.insert(k.to_owned(), v.into_owned());
            }
          }
        }

        output
      })
      .unwrap_or_default(),
  )
}

cfg_if::cfg_if! {
  if #[cfg(any(feature = "actix", feature = "rocket"))] {
    /// A struct that represents an unauthenticated request containing a [`Vote`] data.
    #[must_use]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "actix", feature = "rocket"))))]
    #[derive(Clone)]
    pub struct IncomingVote {
      pub(crate) authorization: String,
      pub(crate) vote: Vote,
    }

    impl IncomingVote {
      /// Authenticates a valid password with this request.
      /// Returns [`Some(Vote)`][`Vote`] if succeeds, otherwise `None`.
      #[must_use]
      #[inline(always)]
      pub fn authenticate<S>(self, password: &S) -> Option<Vote>
      where
        S: AsRef<str> + ?Sized,
      {
        if self.authorization == password.as_ref() {
          Some(self.vote)
        } else {
          None
        }
      }
    }
  }
}

cfg_if::cfg_if! {
  if #[cfg(any(feature = "axum", feature = "warp"))] {
    pub(crate) struct WebhookState<T> {
      pub(crate) state: T,
      pub(crate) password: String,
    }

    /// An async trait for adding an on-vote event handler to your application logic.
    ///
    /// It's described as follows (without `async_trait`'s macro expansion):
    /// ```rust,no_run
    /// #[async_trait::async_trait]
    /// pub trait VoteHandler: Send + Sync + 'static {
    ///   async fn voted(&self, vote: Vote);
    /// }
    /// ```
    #[cfg_attr(docsrs, doc(cfg(any(feature = "axum", feature = "warp"))))]
    #[async_trait::async_trait]
    pub trait VoteHandler: Send + Sync + 'static {
      async fn voted(&self, vote: Vote);
    }
  }
}
