mod error;
pub mod inline_local;
pub mod lifetimes;
mod rename;
pub mod rename_function;
pub mod rename_type;
pub mod rename_variable;

pub use self::error::Response;
pub use self::inline_local::inline_local;
pub use self::lifetimes::{elide_fn_lifetime, restore_fn_lifetime};
pub use self::rename::{rename, rename_dec_and_ref};
pub use self::rename_function::rename_function;
pub use self::rename_type::rename_type;
pub use self::rename_variable::rename_variable;

#[derive(Copy, Clone, PartialEq)]
pub enum RefactorType {
    Variable,
    Function,
    Type,
    InlineLocal,
    Reduced,
    ReifyLifetime,
    ElideLifetime,
    Nil,
}
