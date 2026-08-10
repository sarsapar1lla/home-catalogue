use bon::Builder;
use jiff::{Timestamp, civil::Date};
use uuid::Uuid;

#[derive(Debug)]
pub enum Status {
    Available,
    Loaned { to: String },
    Removed { on: Date, reason: Option<String> },
}

#[derive(Debug, Builder)]
pub struct Book {
    id: Uuid,
    title: String,
    author: String,
    isbn: Option<String>,
    originally_published: Option<u8>,
    edition: Option<String>,
    edition_published: Option<u8>,
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

    pub fn originally_published(&self) -> Option<u8> {
        self.originally_published
    }

    pub fn edition(&self) -> Option<&str> {
        self.edition.as_deref()
    }

    pub fn edition_published(&self) -> Option<u8> {
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
