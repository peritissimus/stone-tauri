// Domain services go here
// These are pure business logic services with NO external dependencies

pub mod link_extractor;
pub mod similarity_calculator;
pub mod task_extractor;

pub use link_extractor::{ExtractedLink, LinkExtractor, LinkType};
pub use similarity_calculator::{
    cosine_similarity, euclidean_distance, manhattan_distance, normalize_vector,
};
pub use task_extractor::{RawTask, TaskExtractor, TaskState};
