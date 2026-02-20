use linfa::prelude::*;
use linfa_anomaly::IsolationForest;
use ndarray::Array2;

pub fn detect_anomaly(features: Vec<Vec<f64>>) -> Vec<bool> {

    let data = Array2::from_shape_vec(
        (features.len(), features[0].len()),
        features.into_iter().flatten().collect(),
    ).unwrap();

    let model = IsolationForest::params().fit(&data).unwrap();
    let scores = model.predict(&data);

    scores.into_iter().map(|v| v < 0.0).collect()
}
