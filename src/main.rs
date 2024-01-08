// Copyright 2023 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use clap::{App, Arg};
use risc0_zkvm::{default_prover, ExecutorEnv};
use serde::Serialize;
use serde_json;
use smartcore::{
    linalg::basic::matrix::DenseMatrix, linear::linear_regression::LinearRegression, svm::svc::SVC,
    tree::decision_tree_classifier::*,
};
use smartcore_ml_methods::DECISION_TREE_ELF;
use smartcore_ml_methods::LINEAR_REGRESSION_ELF;
use smartcore_ml_methods::SVM_CLASSIFICATION_ELF;
use std::time::Instant;

// The serialized trained model and input data are embedded from files
// corresponding paths listed below. Alternatively, the model can be trained in
// the host and/or data can be manually inputted as a smartcore DenseMatrix. If
// this approach is desired, be sure to import the corresponding SmartCore
// modules and serialize the model and data to byte arrays before transfer to
// the guest.
const MODEL_DECISION_TREE: &str = include_str!("../res/ml-model/decision_tree_model_bytes.json");
const DATA_DECISION_TREE: &str = include_str!("../res/input-data/decision_tree_data_bytes.json");
const MODEL_LINEAR_REGRESSION: &str =
    include_str!("../res/ml-model/linear_regression_model_bytes.json");
const DATA_LINEAR_REGRESSION: &str =
    include_str!("../res/input-data/linear_regression_data_bytes.json");

const MODEL_SVM_CLASSIFICATION: &str =
    include_str!("../res/ml-model/svm_classification_model_bytes.json");
const DATA_SVM_CLASSIFICATION: &str =
    include_str!("../res/input-data/svm_classification_data_bytes.json");

fn main() {
    let matches = App::new("Model Prover")
        .version("1.0")
        .author("Your Name")
        .about("Proves models using RISC Zero")
        .arg(
            Arg::with_name("model")
                .short('m')
                .long("model")
                .takes_value(true)
                .help("Specifies the model to prove (linear_regression or decision_tree)"),
        )
        .get_matches();

    // Determine which model to prove based on user input
    let model_type = matches.value_of("model").unwrap_or("default_model");

    let output = match model_type {
        "linear_regressions" => {
            // Convert the model and input data from JSON into byte arrays.
            let model_bytes: Vec<u8> = serde_json::from_str(MODEL_LINEAR_REGRESSION).unwrap();
            let data_bytes: Vec<u8> = serde_json::from_str(DATA_LINEAR_REGRESSION).unwrap();

            // Deserialize the data from rmp into native rust types.
            type Model = LinearRegression<f64, u32, DenseMatrix<f64>, Vec<u32>>;
            let model: Model = rmp_serde::from_slice(&model_bytes)
                .expect("model failed to deserialize byte array");
            let data: DenseMatrix<f64> =
                rmp_serde::from_slice(&data_bytes).expect("data failed to deserialize byte array");
            Ok(predict(&model, data, LINEAR_REGRESSION_ELF))
        }
        "decision_trees" => {
            // Convert the model and input data from JSON into byte arrays.
            let model_bytes: Vec<u8> = serde_json::from_str(MODEL_DECISION_TREE).unwrap();
            let data_bytes: Vec<u8> = serde_json::from_str(DATA_DECISION_TREE).unwrap();

            // Deserialize the data from rmp into native rust types.
            type Model = DecisionTreeClassifier<f64, u32, DenseMatrix<f64>, Vec<u32>>;
            let model: Model = rmp_serde::from_slice(&model_bytes)
                .expect("model failed to deserialize byte array");
            let data: DenseMatrix<f64> =
                rmp_serde::from_slice(&data_bytes).expect("data failed to deserialize byte array");
            Ok(predict(&model, data, DECISION_TREE_ELF))
        }
        "svm_classifications" => {
            // Convert the model and input data from JSON into byte arrays.
            let model_bytes: Vec<u8> = serde_json::from_str(MODEL_SVM_CLASSIFICATION).unwrap();
            let data_bytes: Vec<u8> = serde_json::from_str(DATA_SVM_CLASSIFICATION).unwrap();

            // Deserialize the data from rmp into native rust types.
            let model: SVC<f64, i32, DenseMatrix<f64>, Vec<i32>> =
                rmp_serde::from_slice(&model_bytes)
                    .expect("model failed to deserialize byte array");
            let data: DenseMatrix<f64> =
                rmp_serde::from_slice(&data_bytes).expect("data failed to deserialize byte array");
            Ok(predict(&model, data, SVM_CLASSIFICATION_ELF))
        }
        _ => {
            // return an error if the model type is not recognized
            Err("Model type not recognized")
        }
    };
    let output = output.unwrap();
    println!("Prediction recorded in journal is: {:?}", &output.0);
    println!("Proving time: {:?}", &output.1);
}

fn predict<T: Serialize>(
    model: &T,
    data: DenseMatrix<f64>,
    exec_env: &[u8],
) -> (Vec<u32>, std::time::Duration) {
    let env = ExecutorEnv::builder()
        .write(model)
        .expect("model failed to serialize")
        .write(&data)
        .expect("data failed to serialize")
        .build()
        .unwrap();

    // Obtain the default prover.
    // Note that for development purposes we do not need to run the prover. To
    // bypass the prover, use:
    // ```
    // RISC0_DEV_MODE=1 cargo run -r
    // ```
    let prover = default_prover();

    // This initiates a session, runs the STARK prover on the resulting exection
    // trace, and produces a receipt.
    let start_time = Instant::now();
    let receipt = prover.prove_elf(env, exec_env).unwrap();
    let proving_time = start_time.elapsed();
    // We read the result that the guest code committed to the journal. The
    // receipt can also be serialized and sent to a verifier.
    (receipt.journal.decode().unwrap(), proving_time)
}

#[cfg(test)]
mod test {
    use smartcore::{
        linalg::basic::matrix::DenseMatrix, linear::linear_regression::LinearRegression,
        svm::svc::SVC, tree::decision_tree_classifier::*,
    };
    use smartcore_ml_methods::DECISION_TREE_ELF;
    use smartcore_ml_methods::LINEAR_REGRESSION_ELF;
    use smartcore_ml_methods::SVM_CLASSIFICATION_ELF;
    const MODEL_DECISION_TREE: &str =
        include_str!("../res/ml-model/decision_tree_model_bytes.json");
    const DATA_DECISION_TREE: &str =
        include_str!("../res/input-data/decision_tree_data_bytes.json");
    const MODEL_LINEAR_REGRESSION: &str =
        include_str!("../res/ml-model/linear_regression_model_bytes.json");
    const DATA_LINEAR_REGRESSION: &str =
        include_str!("../res/input-data/linear_regression_data_bytes.json");
    const MODEL_SVM_CLASSIFICATION: &str =
        include_str!("../res/ml-model/svm_classification_model_bytes.json");
    const DATA_SVM_CLASSIFICATION: &str =
        include_str!("../res/input-data/svm_classification_data_bytes.json");
    #[test]
    fn linear_regression() {
        const EXPECTED: &[u32] = &[3];
        // Convert the model and input data from JSON into byte arrays.
        let model_bytes: Vec<u8> = serde_json::from_str(MODEL_LINEAR_REGRESSION).unwrap();
        let data_bytes: Vec<u8> = serde_json::from_str(DATA_LINEAR_REGRESSION).unwrap();

        // Deserialize the data from rmp into native rust types.
        type Model = LinearRegression<f64, u32, DenseMatrix<f64>, Vec<u32>>;
        let model: Model =
            rmp_serde::from_slice(&model_bytes).expect("model failed to deserialize byte array");
        let data: DenseMatrix<f64> =
            rmp_serde::from_slice(&data_bytes).expect("data failed to deserialize byte array");
        let result = super::predict(&model, data, LINEAR_REGRESSION_ELF);
        assert_eq!(EXPECTED, result.0);
    }
    #[test]
    fn decision_tree() {
        const EXPECTED: &[u32] = &[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2,
        ];
        // Convert the model and input data from JSON into byte arrays.
        let model_bytes: Vec<u8> = serde_json::from_str(MODEL_DECISION_TREE).unwrap();
        let data_bytes: Vec<u8> = serde_json::from_str(DATA_DECISION_TREE).unwrap();

        // Deserialize the data from rmp into native rust types.
        type Model = DecisionTreeClassifier<f64, u32, DenseMatrix<f64>, Vec<u32>>;
        let model: Model =
            rmp_serde::from_slice(&model_bytes).expect("model failed to deserialize byte array");
        let data: DenseMatrix<f64> =
            rmp_serde::from_slice(&data_bytes).expect("data failed to deserialize byte array");
        let result = super::predict(&model, data, DECISION_TREE_ELF);
        assert_eq!(EXPECTED, result.0);
    }
    #[test]
    fn svm_classification() {
        const EXPECTED: &[i32] = &[
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        ];

        let model_bytes: Vec<u8> = serde_json::from_str(MODEL_SVM_CLASSIFICATION).unwrap();
        let data_bytes: Vec<u8> = serde_json::from_str(DATA_SVM_CLASSIFICATION).unwrap();

        // Deserialize the data from rmp into native rust types.
        let model: SVC<f64, i32, DenseMatrix<f64>, Vec<i32>> =
            rmp_serde::from_slice(&model_bytes).expect("model failed to deserialize byte array");
        let data: DenseMatrix<f64> =
            rmp_serde::from_slice(&data_bytes).expect("data failed to deserialize byte array");
        let result = super::predict(&model, data, SVM_CLASSIFICATION_ELF);
        // convert result.0 to a Vec<i32>
        let result: Vec<i32> = result.0.iter().map(|x| *x as i32).collect();
        assert_eq!(EXPECTED, result);
    }
}
