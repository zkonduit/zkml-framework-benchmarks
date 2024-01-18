#[cfg(test)]
mod benchmarking_tests {

    use lazy_static::lazy_static;
    use serde_json::Value;
    use std::env::var;
    use std::process::Command;
    use std::sync::Once;
    static COMPILE: Once = Once::new();
    static ENV_SETUP: Once = Once::new();
    static BENCHMARK_FILE: Once = Once::new();
    use regex::Regex;

    // Sure to run this once

    lazy_static! {
        static ref CARGO_TARGET_DIR: String =
            var("CARGO_TARGET_DIR").unwrap_or_else(|_| "./target".to_string());
    }

    fn create_benchmark_json_file() {
        BENCHMARK_FILE.call_once(|| {
            let status = Command::new("bash")
                .args(["benchmark_file.sh"])
                .status()
                .expect("failed to execute process");
            assert!(status.success());
        });
    }

    fn setup_py_env() {
        ENV_SETUP.call_once(|| {
            // supposes that you have a virtualenv called .env and have run the following
            // equivalent of python -m venv .env
            // source .env/bin/activate
            // pip install -r requirements.txt
            // maturin develop --release --features python-bindings
            let python_interpreter = ".env/bin/python";

            // now install torch, pandas, numpy, seaborn, jupyter
            let status = Command::new(python_interpreter)
                .args([
                    "-m",
                    "pip",
                    "install",
                    "ezkl==7.1.4",
                    "onnx==1.14.0",
                    "hummingbird-ml==0.4.9",
                    "torch==2.0.1",
                    "jupyter==1.0.0",
                    "pandas==2.0.3",
                    "sk2torch==1.2.0",
                    "matplotlib==3.4.3",
                    "starknet-py==0.18.3",
                    "skl2onnx==1.16.0",
                ])
                .status()
                .expect("failed to execute process");
            assert!(status.success());
            let status = Command::new("pip")
                .args(["install", "numpy==1.23"])
                .status()
                .expect("failed to execute process");

            assert!(status.success());
        });
    }

    fn init_binary() {
        COMPILE.call_once(|| {
            println!("using cargo target dir: {}", *CARGO_TARGET_DIR);
            setup_py_env();
        });
    }

    const TESTS: [&str; 3] = [
        "linear_regressions",
        "random_forests",
        "svm_classifications",
    ];

    macro_rules! test_func {
        () => {
            #[cfg(test)]
            mod tests {
                use seq_macro::seq;
                use crate::benchmarking_tests::TESTS;
                use test_case::test_case;
                use super::*;

                const RUNS: usize = 10;

                seq!(N in 0..=2 {

                    #(#[test_case(TESTS[N])])*
                    fn run_benchmarks_(test: &str) {
                        // if test is the first test, we need to create the benchmarks.json file
                        if test == TESTS[0] {
                            crate::benchmarking_tests::create_benchmark_json_file();
                        }
                        crate::benchmarking_tests::init_binary();
                        for _ in 0..RUNS {
                            // artifact generation and proving happens all in the ezkl notebook
                            // only artifacts are generated in the risc0 notebook
                            run_notebooks("./notebooks", test);
                            // we need to run the risc0 zkVM VM on the host to get the proving time
                            run_risc0_zk_vm(test);
                        }
                        // pretty print the benchmarks.json file
                        let benchmarks_json = std::fs::read_to_string("./benchmarks.json").unwrap();
                        let benchmarks_json: serde_json::Value = serde_json::from_str(&benchmarks_json).unwrap();
                        println!("{}", serde_json::to_string_pretty(&benchmarks_json).unwrap());
                    }
                });
            }
        };
    }

    fn run_notebooks(test_dir: &str, test: &str) {
        // Define the path to the Python interpreter in the virtual environment
        let python_interpreter = ".env/bin/python";

        let status = Command::new(python_interpreter)
            .args([
                "-m",
                "jupyter",
                "nbconvert",
                "--to",
                "notebook",
                "--execute",
                &format!("{}/{}/{}", test_dir, test, "ezkl.ipynb"),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());
        let status = Command::new(python_interpreter)
            .args([
                "-m",
                "jupyter",
                "nbconvert",
                "--to",
                "notebook",
                "--execute",
                &format!("{}/{}/{}", test_dir, test, "riscZero.ipynb"),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());
        let status = Command::new(python_interpreter)
            .args([
                "-m",
                "jupyter",
                "nbconvert",
                "--to",
                "notebook",
                "--execute",
                &format!("{}/{}/{}", test_dir, test, "orion.ipynb"),
            ])
            .status()
            .expect("failed to execute process");
        assert!(status.success());
    }

    fn run_risc0_zk_vm(test: &str) {
        // Check OS environment variable to dertermine whether to use gtime or time
        let time_command = match var("OS") {
            Ok(val) => {
                if val == "linux" {
                    "/usr/bin/time"
                } else {
                    "gtime"
                }
            }
            Err(_) => "gtime",
        };
        // Command to measure memory usage
        let time_command = format!(
            "{} -v cargo run --release -- --model {}",
            time_command, test
        );

        // Run the command using Bash, capturing both stdout and stderr
        let output = Command::new("bash")
            .arg("-c")
            .arg(&time_command)
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Print stdout and stderr for debugging
        println!("stdout: {}", stdout);
        println!("stderr: {}", stderr);

        // Use regex to extract the Proving time and Memory usage
        let proving_time_re = Regex::new(r"Proving time: (\d+\.\d+)s").unwrap();
        let memory_usage_re = Regex::new(r"Maximum resident set size \(kbytes\): (\d+)").unwrap();

        let proving_time_r0 = proving_time_re
            .captures(&stdout)
            .and_then(|caps| caps.get(1))
            .map_or("".to_string(), |m| m.as_str().to_string() + "s");

        let memory_usage_r0 = memory_usage_re
            .captures(&stderr)
            .and_then(|caps| caps.get(1))
            .map_or("".to_string(), |m| m.as_str().to_string() + "kb");

        // read in benchmarks.json file
        let benchmarks_json = std::fs::read_to_string("./benchmarks.json").unwrap();
        let mut benchmarks_json: serde_json::Value =
            serde_json::from_str(&benchmarks_json).unwrap();

        // Append proving time and memory usage to the list
        let proving_time_list = benchmarks_json[test]["riscZero"]["provingTime"]
            .as_array_mut()
            .unwrap();
        proving_time_list.push(Value::String(proving_time_r0));

        let memory_usage_list = benchmarks_json[test]["riscZero"]["memoryUsage"]
            .as_array_mut()
            .unwrap();
        memory_usage_list.push(Value::String(memory_usage_r0));

        // Write to benchmarks.json file
        std::fs::write(
            "./benchmarks.json",
            serde_json::to_string_pretty(&benchmarks_json).unwrap(),
        )
        .unwrap();
    }

    test_func!();
}
