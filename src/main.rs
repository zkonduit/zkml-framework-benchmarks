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
    ensemble::random_forest_classifier::*, linalg::basic::matrix::DenseMatrix,
    linear::linear_regression::LinearRegression, svm::svc::SVC,
};
use smartcore_ml_methods::LINEAR_REGRESSION_ELF;
use smartcore_ml_methods::RANDOM_FOREST_ELF;
use smartcore_ml_methods::SVM_CLASSIFICATION_ELF;
use std::fs;
use std::time::Instant;

// The serialized trained model and input data are embedded from files
// corresponding paths listed below. Alternatively, the model can be trained in
// the host and/or data can be manually inputted as a smartcore DenseMatrix. If
// this approach is desired, be sure to import the corresponding SmartCore
// modules and serialize the model and data to byte arrays before transfer to
// the guest.

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
                .help("Specifies the model to prove (linear_regression or random_forest)"),
        )
        .get_matches();

    // Determine which model to prove based on user input
    let model_type = matches.value_of("model").unwrap_or("default_model");

    let output = match model_type {
        "linear_regressions" => {
            let model_linear_regression =
                &fs::read_to_string("./res/ml-model/linear_regression_model_bytes.json").unwrap();
            let data_linear_regression =
                &fs::read_to_string("./res/input-data/linear_regression_data_bytes.json").unwrap();
            // Convert the model and input data from JSON into byte arrays.
            let model_bytes: Vec<u8> = serde_json::from_str(model_linear_regression).unwrap();
            let data_bytes: Vec<u8> = serde_json::from_str(data_linear_regression).unwrap();

            // Deserialize the data from rmp into native rust types.
            type Model = LinearRegression<f64, u32, DenseMatrix<f64>, Vec<u32>>;
            let model: Model = rmp_serde::from_slice(&model_bytes)
                .expect("model failed to deserialize byte array");
            let data: DenseMatrix<f64> =
                rmp_serde::from_slice(&data_bytes).expect("data failed to deserialize byte array");
            Ok(predict(&model, data, LINEAR_REGRESSION_ELF))
        }
        "random_forests" => {
            let model_random_forest =
                &fs::read_to_string("./res/ml-model/random_forest_model_bytes.json").unwrap();
            let data_random_forest =
                &fs::read_to_string("./res/input-data/random_forest_data_bytes.json").unwrap();
            // Convert the model and input data from JSON into byte arrays.
            let model_bytes: Vec<u8> = serde_json::from_str(model_random_forest).unwrap();
            let data_bytes: Vec<u8> = serde_json::from_str(data_random_forest).unwrap();

            // Deserialize the data from rmp into native rust types.
            type Model = RandomForestClassifier<f64, u8, DenseMatrix<f64>, Vec<u8>>;
            let model: Model = rmp_serde::from_slice(&model_bytes)
                .expect("model failed to deserialize byte array");
            let data: DenseMatrix<f64> =
                rmp_serde::from_slice(&data_bytes).expect("data failed to deserialize byte array");
            Ok(predict(&model, data, RANDOM_FOREST_ELF))
        }
        "svm_classifications" => {
            let model_svm_classification =
                &fs::read_to_string("./res/ml-model/svm_classification_model_bytes.json").unwrap();
            let data_svm_classification =
                &fs::read_to_string("./res/input-data/svm_classification_data_bytes.json").unwrap();
            // Convert the model and input data from JSON into byte arrays.
            let model_bytes: Vec<u8> = serde_json::from_str(model_svm_classification).unwrap();
            let data_bytes: Vec<u8> = serde_json::from_str(data_svm_classification).unwrap();

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
    use std::env;
    use std::fs;

    use smartcore::{
        ensemble::random_forest_classifier::*, linalg::basic::matrix::DenseMatrix,
        linear::linear_regression::LinearRegression, svm::svc::SVC,
    };
    use smartcore_ml_methods::LINEAR_REGRESSION_ELF;
    use smartcore_ml_methods::RANDOM_FOREST_ELF;
    use smartcore_ml_methods::SVM_CLASSIFICATION_ELF;

    #[test]
    fn linear_regression() {
        let model_linear_regression =
            &fs::read_to_string("./res/ml-model/linear_regression_model_bytes.json").unwrap();
        let data_linear_regression =
            &fs::read_to_string("./res/input-data/linear_regression_data_bytes.json").unwrap();
        const EXPECTED: &[u32] = &[3];
        // Convert the model and input data from JSON into byte arrays.
        let model_bytes: Vec<u8> = serde_json::from_str(model_linear_regression).unwrap();
        let data_bytes: Vec<u8> = serde_json::from_str(data_linear_regression).unwrap();

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
    fn random_forest() {
        println!(
            "Current working directory: {:?}",
            env::current_dir().unwrap()
        );
        let model_random_forest =
            &fs::read_to_string("./res/ml-model/random_forest_model_bytes.json").unwrap();
        let data_random_forest =
            &fs::read_to_string("./res/input-data/random_forest_data_bytes.json").unwrap();
        const EXPECTED: &[u8] = &[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2,
        ];
        // Convert the model and input data from JSON into byte arrays.
        let model_bytes: Vec<u8> = serde_json::from_str(model_random_forest).unwrap();
        let data_bytes: Vec<u8> = serde_json::from_str(data_random_forest).unwrap();

        // Deserialize the data from rmp into native rust types.
        type Model = RandomForestClassifier<f64, u8, DenseMatrix<f64>, Vec<u8>>;
        let model: Model =
            rmp_serde::from_slice(&model_bytes).expect("model failed to deserialize byte array");
        let data: DenseMatrix<f64> =
            rmp_serde::from_slice(&data_bytes).expect("data failed to deserialize byte array");
        let result = super::predict(&model, data, RANDOM_FOREST_ELF);
        // convert result.0 to a Vec<u8>
        let result: Vec<u8> = result.0.iter().map(|x| *x as u8).collect();
        assert_eq!(EXPECTED, result);
    }
    #[test]
    fn svm_classification() {
        let model_svm_classification =
            &fs::read_to_string("./res/ml-model/svm_classification_model_bytes.json").unwrap();
        let data_svm_classification =
            &fs::read_to_string("./res/input-data/svm_classification_data_bytes.json").unwrap();
        const EXPECTED: &[i32] = &[
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        ];

        let model_bytes: Vec<u8> = serde_json::from_str(model_svm_classification).unwrap();
        let data_bytes: Vec<u8> = serde_json::from_str(data_svm_classification).unwrap();

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
