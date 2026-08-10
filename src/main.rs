use jiff::Timestamp;
use uuid::Uuid;

use crate::model::{Book, Status};

mod model;

fn main() {
    let book = Book::builder()
        .id(Uuid::new_v4())
        .title("The Lord of the Rings".to_string())
        .author("J. R. R. Tolkein".to_string())
        .status(Status::Loaned {
            to: "Oz".to_string(),
        })
        .created(Timestamp::now())
        .updated(Timestamp::now())
        .build();

    println!("{book:?}");
}
