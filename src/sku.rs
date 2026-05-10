#[repr(transparent)]
pub struct Sku(pub String);

impl Sku {
    pub fn new(sku: impl Into<String>) -> Self {
        Self(sku.into())
    }
}

pub trait IsErrorContext {
    type Context<'a>;
    fn context<'a>(self, context: &'a str) -> Result<Self, &'static str> where Self: Sized;
}
