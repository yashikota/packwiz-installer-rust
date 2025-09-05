#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ValidityToken(String);

impl ValidityToken {
    pub fn new(token: impl Into<String>) -> Self {
        Self(token.into())
    }
}
