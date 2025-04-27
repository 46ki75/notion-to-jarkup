#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("notionrs error: {0}")]
    NotionRs(#[from] notionrs::error::Error),

    #[error("reqwest error: {0}")]
    Reqewst(#[from] reqwest::Error),

    #[error("scraper error: {0}")]
    Scraper(#[from] scraper::error::SelectorErrorKind<'static>),
}
