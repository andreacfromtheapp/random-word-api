use serde::{Deserialize, Serialize};
use std::fmt;

pub const REPO_URL: &str = "https://github.com/andreacfromtheapp/random-word-api";

pub fn api_url() -> String {
    dotenvy::var("API_URL").unwrap_or_else(|_| "http://localhost:3000".to_string())
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GrammaticalType {
    Random,
    Noun,
    Verb,
    Adjective,
    Adverb,
    // Pronoun,
    // Preposition,
    // Conjunction,
    // Interjection,
    // Article,
}

impl GrammaticalType {
    pub fn name(&self) -> &'static str {
        match self {
            GrammaticalType::Random => "Random",
            GrammaticalType::Noun => "Noun",
            GrammaticalType::Verb => "Verb",
            GrammaticalType::Adjective => "Adjective",
            GrammaticalType::Adverb => "Adverb",
            // GrammaticalType::Pronoun => "Pronoun",
            // GrammaticalType::Preposition => "Preposition",
            // GrammaticalType::Conjunction => "Conjunction",
            // GrammaticalType::Interjection => "Interjection",
            // GrammaticalType::Article => "Article",
        }
    }

    pub fn api_name(&self) -> &'static str {
        match self {
            GrammaticalType::Random => "random",
            GrammaticalType::Noun => "noun",
            GrammaticalType::Verb => "verb",
            GrammaticalType::Adjective => "adjective",
            GrammaticalType::Adverb => "adverb",
            // GrammaticalType::Pronoun => "pronoun",
            // GrammaticalType::Preposition => "preposition",
            // GrammaticalType::Conjunction => "conjunction",
            // GrammaticalType::Interjection => "interjection",
            // GrammaticalType::Article => "article",
        }
    }

    pub fn all() -> &'static [GrammaticalType] {
        &[
            GrammaticalType::Random,
            GrammaticalType::Noun,
            GrammaticalType::Verb,
            GrammaticalType::Adjective,
            GrammaticalType::Adverb,
            // GrammaticalType::Pronoun,
            // GrammaticalType::Preposition,
            // GrammaticalType::Conjunction,
            // GrammaticalType::Interjection,
            // GrammaticalType::Article,
        ]
    }
}

impl fmt::Display for GrammaticalType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.api_name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LanguageCode {
    English,
    // German,
    // French,
    // Spanish,
    // Italian,
    // Dutch,
}

impl LanguageCode {
    pub fn code(&self) -> &'static str {
        match self {
            LanguageCode::English => "en",
            // LanguageCode::German => "de",
            // LanguageCode::French => "fr",
            // LanguageCode::Spanish => "es",
            // LanguageCode::Italian => "it",
            // LanguageCode::Dutch => "nl",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            LanguageCode::English => "English",
            // LanguageCode::German => "Deutsch",
            // LanguageCode::French => "Francais",
            // LanguageCode::Spanish => "Espanol",
            // LanguageCode::Italian => "Italiano",
            // LanguageCode::Dutch => "Dutch",
        }
    }

    pub fn flag_code(&self) -> &'static str {
        match self {
            LanguageCode::English => "us",
            // LanguageCode::German => "de",
            // LanguageCode::French => "fr",
            // LanguageCode::Spanish => "es",
            // LanguageCode::Italian => "it",
            // LanguageCode::Dutch => "nl",
        }
    }

    pub fn all() -> &'static [LanguageCode] {
        &[
            LanguageCode::English,
            // LanguageCode::German,
            // LanguageCode::French,
            // LanguageCode::Spanish,
            // LanguageCode::Italian,
            // LanguageCode::Dutch,
        ]
    }
}

impl fmt::Display for LanguageCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

#[derive(Default, Deserialize, Serialize, Clone)]
pub struct Word {
    pub word: String,
    pub definition: String,
    pub pronunciation: String,
}
