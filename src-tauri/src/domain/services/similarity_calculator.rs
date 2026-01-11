/// SimilarityCalculator - Pure domain service for calculating vector similarity
///
/// Provides methods to calculate similarity between embedding vectors.
/// This is pure business logic with no external dependencies.

/// Calculate cosine similarity between two vectors
///
/// Returns a value between -1.0 and 1.0, where:
/// - 1.0 = identical vectors
/// - 0.0 = orthogonal vectors
/// - -1.0 = opposite vectors
pub fn cosine_similarity(vec_a: &[f32], vec_b: &[f32]) -> Result<f32, String> {
    if vec_a.len() != vec_b.len() {
        return Err(format!(
            "Vectors must have the same length: {} != {}",
            vec_a.len(),
            vec_b.len()
        ));
    }

    if vec_a.is_empty() {
        return Err("Vectors cannot be empty".to_string());
    }

    let dot_product: f32 = vec_a.iter().zip(vec_b.iter()).map(|(a, b)| a * b).sum();

    let magnitude_a: f32 = vec_a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = vec_b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return Ok(0.0);
    }

    Ok(dot_product / (magnitude_a * magnitude_b))
}

/// Calculate Euclidean distance between two vectors
///
/// Lower values indicate more similar vectors
pub fn euclidean_distance(vec_a: &[f32], vec_b: &[f32]) -> Result<f32, String> {
    if vec_a.len() != vec_b.len() {
        return Err(format!(
            "Vectors must have the same length: {} != {}",
            vec_a.len(),
            vec_b.len()
        ));
    }

    if vec_a.is_empty() {
        return Err("Vectors cannot be empty".to_string());
    }

    let distance: f32 = vec_a
        .iter()
        .zip(vec_b.iter())
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f32>()
        .sqrt();

    Ok(distance)
}

/// Calculate Manhattan distance (L1 distance) between two vectors
///
/// Lower values indicate more similar vectors
pub fn manhattan_distance(vec_a: &[f32], vec_b: &[f32]) -> Result<f32, String> {
    if vec_a.len() != vec_b.len() {
        return Err(format!(
            "Vectors must have the same length: {} != {}",
            vec_a.len(),
            vec_b.len()
        ));
    }

    if vec_a.is_empty() {
        return Err("Vectors cannot be empty".to_string());
    }

    let distance: f32 = vec_a
        .iter()
        .zip(vec_b.iter())
        .map(|(a, b)| (a - b).abs())
        .sum();

    Ok(distance)
}

/// Normalize a vector to unit length
pub fn normalize_vector(vec: &[f32]) -> Vec<f32> {
    let magnitude: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();

    if magnitude == 0.0 {
        return vec.to_vec();
    }

    vec.iter().map(|x| x / magnitude).collect()
}

