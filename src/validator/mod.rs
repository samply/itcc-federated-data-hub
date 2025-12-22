mod core;
pub mod schema;
mod strict;

enum ValidatorMode {
    Strict,
    Schema,
    Core,
}
