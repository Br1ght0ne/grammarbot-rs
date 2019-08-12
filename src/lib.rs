//! # grammarbot-rs
//!
//! An API client for GrammarBot - a free grammar checking API.
//!
//! # Usage
//!
//! A [`Client`](Client) is the primary way to interact with the API.
//!
//! ```no_run
//! use grammarbot::Client;
//! let client = Client::new("your_api_key");
//! ```
//!
//! Use [`check`](Client::check) to check your texts.
//!
//! ```no_run
//! # use grammarbot::Client;
//! # let client = Client::new("your_api_key");
//! let response = client.check("I can't remember how to go their.");
//! ```
//!
//! The [`Response`](Response) struct stores all information
//! in the fields that you can access directly:
//!
//! ```no_run
//! # use grammarbot::Client;
//! # let client = Client::new("your_api_key");
//! # let response = client.check("I can't remember how to go their.")?;
//! assert_eq!("CONFUSION_RULE", response.matches[0].rule.id);
//! # Ok::<(), grammarbot::Error>(())
//! ```

extern crate reqwest;
extern crate serde;
extern crate snafu;

use serde::Deserialize;
use snafu::{ResultExt, Snafu};

/// All the entities used in responses from the API.
pub mod types {
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Software {
        pub name: String,
        pub version: String,
        pub api_version: u8,
        pub premium: bool,
        pub premium_hint: String,
        pub status: String,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Warnings {
        pub incomplete_results: bool,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Language {
        pub name: String,
        pub code: String,
        pub detected_language: DetectedLanguage,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DetectedLanguage {
        pub name: String,
        pub code: String,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Match {
        pub message: String,
        pub short_message: String,
        pub replacements: Vec<Replacement>,
        pub offset: u32,
        pub length: u32,
        pub context: Context,
        pub sentence: String,
        pub r#type: Type,
        pub rule: Rule,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Replacement {
        pub value: String,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Context {
        pub text: String,
        pub offset: u32,
        pub length: u32,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Type {
        pub type_name: String,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Rule {
        pub id: String,
        pub description: String,
        pub issue_type: String,
        pub category: Category,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Category {
        /// Category ID. `"TYPOS"`
        pub id: String,
        /// Category name. `"Possible Typo"`
        pub name: String,
    }
}

/// A typed representation of the JSON response.
#[derive(Debug, Deserialize)]
pub struct Response {
    pub software: types::Software,
    pub warnings: types::Warnings,
    pub language: types::Language,
    pub matches: Vec<types::Match>,
}

/// The primary way to interact with the API.
pub struct Client {
    api_key: String,
    language: String,
    base: reqwest::Url,
    client: reqwest::Client,
}

impl Client {
    /// Create a new `Client`.
    ///
    /// Currently it requires an API key.
    /// If you need one, [sign up](https://www.grammarbot.io/signup)
    /// with GrammarBot.
    ///
    /// ```
    /// # use grammarbot::Client;
    /// let api_key = "your_api_key";
    /// let client = Client::new(api_key);
    /// ```
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            language: "en-US".to_string(),
            base: reqwest::Url::parse("http://api.grammarbot.io").unwrap(),
            client: reqwest::Client::new(),
        }
    }

    pub fn check(&self, text: &str) -> Result<Response> {
        self.request(reqwest::Method::GET, "/v2/check", &[("text", text)])?
            .json()
            .context(InvalidJSON)
    }

    /// Set the API key for the client.
    ///
    /// ```no_run
    /// # use grammarbot::Client;
    /// # let api_key = "test";
    /// # let other_api_key = "test2";
    /// let client = Client::new(api_key).api_key(other_api_key);
    /// ```
    pub fn api_key(&mut self, api_key: &str) -> &mut Self {
        self.api_key = api_key.to_string();
        self
    }

    /// Set the language for the client.
    ///
    /// ```no_run
    /// # use grammarbot::Client;
    /// # let api_key = "test";
    /// let client = Client::new(api_key).language("en-UK");
    /// ```
    pub fn language(&mut self, language: &str) -> &mut Self {
        self.language = language.to_string();
        self
    }

    /// Set the base URL for the client.
    ///
    /// ```no_run
    /// # use grammarbot::Client;
    /// # let api_key = "test";
    /// let client = Client::new(api_key).base("http://pro.grammarbot.io");
    /// ```
    pub fn base(&mut self, base: &str) -> Result<&mut Self> {
        self.base = reqwest::Url::parse(base).context(InvalidUrl)?;
        Ok(self)
    }

    fn request(
        &self,
        method: reqwest::Method,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<reqwest::Response> {
        self.client
            .request(method, self.base.join(path).context(InvalidUrl)?)
            .query(&query)
            .send()
            .context(RequestFailed)
    }
}

/// A domain-specific error type.
#[derive(Debug, Snafu)]
pub enum Error {
    /// A request failed to execute; there is no valid response.
    #[snafu(display("request failed: {}", source))]
    RequestFailed {
        /// A source error from `reqwest`.
        source: reqwest::Error,
    },
    /// An invalid URL was supplied to [`parse`](reqwest::Url::parse).
    #[snafu(display("invalid URL: {}", source))]
    InvalidUrl {
        /// A source error from `reqwest`.
        source: reqwest::UrlError,
    },
    /// Response returned invalid JSON.
    #[snafu(display("invalid JSON: {}", source))]
    InvalidJSON {
        /// A source error from `reqwest`.
        source: reqwest::Error,
    },
}

/// A short alias for a [`Result`] with [`Error`].
///
/// [Result]: std::result::Result
pub type Result<T, E = Error> = std::result::Result<T, E>;
