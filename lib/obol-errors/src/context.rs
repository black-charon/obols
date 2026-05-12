pub trait ErrorContext {}

pub trait ErrorContextExt {
    fn with_context<C>(self, context: C) -> Self;
}
