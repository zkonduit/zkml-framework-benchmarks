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

use risc0_zkvm::{default_prover, ExecutorEnv};
use serde_json;
use smartcore::{linalg::basic::matrix::DenseMatrix, linear::linear_regression::LinearRegression};
use smartcore_ml_methods::ZKML_GUEST_ELF;
use std::time::Instant;

// The serialized trained model and input data are embedded from files
// corresponding paths listed below. Alternatively, the model can be trained in
// the host and/or data can be manually inputted as a smartcore DenseMatrix. If
// this approach is desired, be sure to import the corresponding SmartCore
// modules and serialize the model and data to byte arrays before transfer to
// the guest.
const JSON_MODEL: &str = include_str!("../tree_model_bytes.json");
const JSON_DATA: &str = include_str!("../tree_model_data_bytes.json");

fn main() {
    let output = predict();
    println!("Prediction recorded in journal is: {:?}", &output.0);
    println!("Proving time: {:?}", &output.1);
}

fn predict() -> (Vec<u32>, std::time::Duration) {
    // Convert the model and input data from JSON into byte arrays.
    let model_bytes: Vec<u8> = serde_json::from_str(JSON_MODEL).unwrap();
    let data_bytes: Vec<u8> = serde_json::from_str(JSON_DATA).unwrap();

    // Deserialize the data from rmp into native rust types.
    type Model = LinearRegression<f64, u32, DenseMatrix<f64>, Vec<u32>>;
    let model: Model =
        rmp_serde::from_slice(&model_bytes).expect("model failed to deserialize byte array");
    let data: DenseMatrix<f64> =
        rmp_serde::from_slice(&data_bytes).expect("data filed to deserialize byte array");

    let env = ExecutorEnv::builder()
        .write(&model)
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
    let receipt = prover.prove_elf(env, ZKML_GUEST_ELF).unwrap();
    let proving_time = start_time.elapsed();
    // We read the result that the guest code committed to the journal. The
    // receipt can also be serialized and sent to a verifier.
    (receipt.journal.decode().unwrap(), proving_time)
}

#[cfg(test)]
mod test {
    #[test]
    fn basic() {
        const EXPECTED: &[u32] = &[3];
        let result = super::predict();
        assert_eq!(EXPECTED, result.0);
    }
}
