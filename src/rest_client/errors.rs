
#[derive(Debug, PartialEq, Eq)]
pub struct RestClientError {
    pub(super) kind: RestClientErrorKind,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RestClientErrorKind {

}