pub mod storage_manager;
pub mod ser;
pub mod ser_v1;

#[cfg(target_arch = "wasm32")]
pub mod web;
#[cfg(not(target_arch = "wasm32"))]
pub mod native;