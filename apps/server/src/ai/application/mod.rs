//! AI application layer: orchestrator, tool executor, and prompt builder.

pub mod orchestrator;
pub mod prompts;
pub mod tools;

pub use orchestrator::AiOrchestrator;
pub use prompts::observability_system_prompt;
pub use tools::{ToolError, ToolRegistry};
