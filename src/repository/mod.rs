pub mod activity;
pub mod collection;
pub mod git;
pub mod link;
pub mod project;
pub mod tag;
pub mod todo;

pub(crate) fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}
