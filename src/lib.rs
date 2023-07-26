mod cache;
mod store;
pub use cache::TinyRef;

pub fn new() -> TinyRef {
    TinyRef::new()
}
pub fn with_name(name: impl ToString) -> TinyRef {
    TinyRef::with_name(name)
}
