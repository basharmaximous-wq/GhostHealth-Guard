use linfa::prelude::*;
use linfa_anomaly::IsolationForest;
use ndarray::Array2;

pub fn detect_anomaly(features: Vec<Vec<f64>>) -> anyhow::Result<Vec<bool>> {
    if features.is_empty() {
        return Ok(vec![]);
    }
    let data = Array2::from_shape_vec(
        (features.len(), features[0].len()),
        features.into_iter().flatten().collect(),
    ).map_err(|e| anyhow::anyhow!("Failed to build array: {}", e))?;

    let model = IsolationForest::params()
        .fit(&data)
        .map_err(|e| anyhow::anyhow!("Model fit failed: {}", e))?;
    let scores = model.predict(&data);

    Ok(scores.into_iter().map(|v| v < 0.0).collect())
}
