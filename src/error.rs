pub type MinilateResult<T> = std::result::Result<T, MinilateError>;

#[derive(Debug, Clone, PartialEq, Eq, Hash, thiserror::Error)]
pub enum MinilateError {
    TemplateExists { template_name: String },
    MissingTemplate { template_name: String },
}

impl std::fmt::Display for MinilateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
