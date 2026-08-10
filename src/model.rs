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
    originally_published: Option<usize>,
    edition: Option<String>,
    edition_published: Option<usize>,
    status: Status,
    created: Timestamp,
    updated: Timestamp,
}
