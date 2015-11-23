// The general idea is to expand the variants to be more fine-grained.
#[derive(Debug, PartialEq)]
pub enum Response {
    Error,
    Conflict,
    Possible,
}