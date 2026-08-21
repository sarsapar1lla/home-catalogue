use bon::Builder;
use jiff::{Timestamp, civil::Date};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Status {
    Available,
    LoanedIn { on: Date, from: String },
    LoanedOut { on: Date, to: String },
    Removed { on: Date, reason: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Author {
    Single(String),
    Several {
        first: String,
        second: String,
        rest: Vec<String>,
    },
}

#[derive(Debug, Clone, Builder, PartialEq, Eq)]
pub struct Book {
    id: Uuid,
    title: String,
    subtitle: Option<String>,
    author: Author,
    isbn: Option<String>,
    first_published: Option<u16>,
    owner: String,
    notes: Option<String>,
    status: Status,
    created: Timestamp,
    updated: Timestamp,
}

impl Book {
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn subtitle(&self) -> Option<&str> {
        self.subtitle.as_deref()
    }

    pub fn author(&self) -> &Author {
        &self.author
    }

    pub fn isbn(&self) -> Option<&str> {
        self.isbn.as_deref()
    }

    pub fn first_published(&self) -> Option<u16> {
        self.first_published
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }

    pub fn status(&self) -> &Status {
        &self.status
    }

    pub fn created(&self) -> Timestamp {
        self.created
    }

    pub fn updated(&self) -> Timestamp {
        self.updated
    }
}
