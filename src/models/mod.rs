pub mod contests;
pub mod errors;
pub mod problems;
pub mod testing;
pub mod users;
pub mod verdicts;

pub use aj_models::DeletionRequest;

/// Deep-compare props values that come from `aj-models` (whose types have
/// no `PartialEq`, required by Dioxus `Props`). Used by manual `PartialEq`
/// impls on component props to keep memoization working.
pub fn props_json_eq<T: serde::Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).ok() == serde_json::to_value(b).ok()
}