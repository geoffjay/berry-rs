//! CLI command implementations.

pub mod forget;
pub mod init;
pub mod mcp;
pub mod recall;
pub mod remember;
pub mod search;
pub mod serve;

pub use forget::run as forget;
pub use init::run as init;
pub use mcp::run as mcp;
pub use recall::run as recall;
pub use remember::run as remember;
pub use search::run as search;
pub use serve::run as serve;
