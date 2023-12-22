#[cfg(test)]
mod benchmarking_tests {

    use lazy_static::lazy_static;
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
            var("CARGO_TARGET_DIR").unwrap_or_else(|_| "./R0_zkVM/target".to_string());
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

            // now install torch, pandas, numpy, seaborn, jupyter
            let status = Command::new("pip")
                .args([
                    "install",
                    "ezkl==7.0.0",
                    "onnx==1.14.0",
                    "hummingbird-ml==0.4.9",
                    "torch==2.0.1",
                    "jupyter==1.0.0",
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

    const TESTS: [&str; 1] = ["linear_regressions"];

    macro_rules! test_func {
        () => {
            #[cfg(test)]
            mod tests {
                use seq_macro::seq;
                use crate::benchmarking_tests::TESTS;
                use test_case::test_case;
                use super::*;

                seq!(N in 0..=0 {

                #(#[test_case(TESTS[N])])*
                fn run_benchmarks_(test: &str) {
                    crate::benchmarking_tests::init_binary();
                    crate::benchmarking_tests::create_benchmark_json_file();
                    run_notebooks("./notebooks", test);
                    run_risc0_zk_vm();
                }
                });
            }
        };
    }

    fn run_notebooks(test_dir: &str, test: &str) {
        // Define the path to the Python interpreter in the virtual environment
        let python_interpreter = ".env/bin/python";

        // Replace the 'source .env/bin/activate' with direct python interpreter call
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
    }

    fn run_risc0_zk_vm() {
        // Run the risc0 smartcore model on the host, then get the proving time
        let output = Command::new("cargo")
            .env("RISC0_DEV_MODE", "0") // Set the environment variable
            .args(&["run", "--release"])
            .output()
            .expect("Failed to execute command");

        // You can then print the output or handle it as needed
        println!("Status: {}", output.status);

        {
            let output = String::from_utf8_lossy(&output.stdout);
            // use regex to extract the Proving time
            let re = Regex::new(r"Proving time: (\d+\.\d+)s").unwrap();
            let caps = re.captures(&output).unwrap();
            let proving_time_r0 = caps.get(1).map_or("", |m| m.as_str()).to_string() + "s";
            // read in benchmarks.json file
            let benchmarks_json = std::fs::read_to_string("./benchmarks.json").unwrap();
            let mut benchmarks_json: serde_json::Value =
                serde_json::from_str(&benchmarks_json).unwrap();
            benchmarks_json["linear_regressions"]["riscZero"]["provingTime"] =
                serde_json::Value::String(proving_time_r0);
            // write to benchmarks.json file
            std::fs::write(
                "./benchmarks.json",
                serde_json::to_string_pretty(&benchmarks_json).unwrap(),
            )
            .unwrap();
        }
    }

    test_func!();
}
