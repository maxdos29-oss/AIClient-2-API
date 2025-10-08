/*!
 * AIClient-2-API Rust Library
 *
 * Core library modules for the AI API proxy server.
 */

pub mod common;
pub mod convert;
pub mod convert_detailed;
pub mod logger;
pub mod system_prompt;

// Re-export commonly used types
pub use common::{ModelProtocol, ModelProvider};

