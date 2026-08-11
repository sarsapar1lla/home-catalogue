use bon::Builder;
use jiff::{Timestamp, civil::Date};
use uuid::Uuid;

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum Status {
    Available,
    Loaned { on: Date, to: String },
    Removed { on: Date, reason: Option<String> },
}

#[derive(Debug, Clone, Builder)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Book {
    id: Uuid,
    title: String,
    author: String,
    isbn: Option<String>,
    originally_published: Option<u16>,
    edition: Option<String>,
    edition_published: Option<u16>,
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

    pub fn author(&self) -> &str {
        &self.author
    }

    pub fn isbn(&self) -> Option<&str> {
        self.isbn.as_deref()
    }

    pub fn originally_published(&self) -> Option<u16> {
        self.originally_published
    }

    pub fn edition(&self) -> Option<&str> {
        self.edition.as_deref()
    }

    pub fn edition_published(&self) -> Option<u16> {
        self.edition_published
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
