pub mod ai;
pub mod assets;
pub mod models;
pub mod rules;
#[cfg(all(feature = "tui", not(target_arch = "wasm32")))]
pub mod ui;
pub mod utils;
#[cfg(feature = "web")]
pub mod web_ui;

#[cfg(test)]
mod test_support;
