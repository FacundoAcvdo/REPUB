pub mod ai_client;
pub mod import_service;
pub mod local_embeddings;
pub mod rag_service;
pub mod reader_service;
pub mod stats_service;

pub use ai_client::AiClient;
pub use import_service::{BookImportService, LibraryStorage};
pub use local_embeddings::LocalEmbeddings;
pub use rag_service::{AssistantAnswer, IndexSummary, RagService};
pub use reader_service::ReaderService;
pub use stats_service::StatsService;
