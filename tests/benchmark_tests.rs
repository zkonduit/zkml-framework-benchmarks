#[cfg(test)]
mod benchmarking_tests {

    use lazy_static::lazy_static;
    use serde_json::{json, Value};
    use std::env::var;
    use std::sync::Once;
    use std::{process::Command, time::Instant};
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
            // Run `cargo build --release` first to build the risc0 binary
            let status = Command::new("cargo")
                .args(["build", "--release"])
                .status()
                .expect("failed to execute process");
            assert!(status.success());
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

                const RUNS: usize = 1;

                const TIME_CMD: &str = if cfg!(target_os = "linux") {
                    "/usr/bin/time"
                } else {
                    "gtime"
                };

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
                            run_risc0_zk_vm(test, TIME_CMD);
                            run_cairo_vm(test, TIME_CMD);
                        }
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

    fn run_risc0_zk_vm(test: &str, time_cmd: &str) {
        // Wrap the risc0 binry run command in the gnu time command
        let command = format!(
            "{} -v target/release/zkml-benchmarks --model {}",
            time_cmd, test
        );

        // Run the command using Bash, capturing both stdout and stderr
        let output = Command::new("bash")
            .arg("-c")
            .arg(&command)
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

        update_benchmarks_json(
            test,
            "riscZero",
            Value::String(proving_time_r0),
            Value::String(memory_usage_r0),
        );
    }

    fn run_cairo_vm(test: &str, time_cmd: &str) {
        // run `scarb build`
        let status = Command::new("scarb")
            .args(["build"])
            .status()
            .expect("failed to execute process");
        assert!(status.success());

        let cmd = vec![
            "scarb",
            "cairo-run",
            "--no-build",
            "--available-gas",
            "99999999999999999",
        ];

        let full_cmd = format!("{} -f \"%M\" -- {}", time_cmd, cmd.join(" "));
        let start_time = Instant::now();
        let output = Command::new("bash")
            .arg("-c")
            .arg(&full_cmd)
            .output()
            .expect("Failed to execute command");

        let end_time = Instant::now();
        let proving_time = end_time.duration_since(start_time).as_secs_f64();

        println!("Full Output:");
        println!("{}", String::from_utf8_lossy(&output.stdout));
        println!("{}", String::from_utf8_lossy(&output.stderr));

        // panic if output is not successful
        if !output.status.success() {
            panic!("Error: Cairo VM failed to run with gnu time command");
        }

        let memory_usage = String::from_utf8_lossy(&output.stderr).trim().to_string();

        let memory_usage_kb = format!("{}kb", memory_usage);
        println!("Memory Usage: {}", memory_usage_kb);
        println!("Compilation Time: {:.3} seconds", proving_time);

        update_benchmarks_json(
            test,
            "orion",
            json!(format!("{:.3}s", proving_time)),
            Value::String(memory_usage),
        );
    }

    fn update_benchmarks_json(test: &str, framework: &str, time: Value, memory: Value) {
        // Read in the benchmarks.json file
        let benchmarks_json = std::fs::read_to_string("./benchmarks.json").unwrap();
        let mut benchmarks_json: serde_json::Value =
            serde_json::from_str(&benchmarks_json).unwrap();

        // Append proving time and memory usage to the list
        let proving_time_list = benchmarks_json[test][framework]["provingTime"]
            .as_array_mut()
            .unwrap();
        proving_time_list.push(time);

        let memory_usage_list = benchmarks_json[test][framework]["memoryUsage"]
            .as_array_mut()
            .unwrap();
        memory_usage_list.push(memory);

        // Write to benchmarks.json file
        std::fs::write(
            "./benchmarks.json",
            serde_json::to_string_pretty(&benchmarks_json).unwrap(),
        )
        .unwrap();
    }

    test_func!();
}
