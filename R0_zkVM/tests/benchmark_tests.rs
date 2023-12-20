#[cfg(test)]
mod benchmarking_tests {

    use lazy_static::lazy_static;
    use std::env::var;
    use std::process::{Child, Command};
    use std::sync::Once;
    use tempdir::TempDir;
    static COMPILE: Once = Once::new();
    static ENV_SETUP: Once = Once::new();
    static BENCHMARK_FILE: Once = Once::new();

    // Sure to run this once

    lazy_static! {
        static ref CARGO_TARGET_DIR: String =
            var("CARGO_TARGET_DIR").unwrap_or_else(|_| "./R0_zkVM/target".to_string());
    }

    fn start_anvil(limitless: bool) -> Child {
        let mut args = vec!["-p", "3030"];
        if limitless {
            args.push("--code-size-limit=41943040");
            args.push("--disable-block-gas-limit");
        }
        let child = Command::new("anvil")
            .args(args)
            // .stdout(Stdio::piped())
            .spawn()
            .expect("failed to start anvil process");

        std::thread::sleep(std::time::Duration::from_secs(3));
        child
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

    fn mv_test_(test_dir: &str, test: &str) {
        let source_path = format!("./notebooks/{}", test);
        let destination_path = format!("{}/{}", test_dir, test);

        // Check if the destination directory already exists
        let dest_path: std::path::PathBuf = destination_path.clone().into();
        if !dest_path.exists() {
            // Copy the entire directory in one go
            let status = Command::new("cp")
                .args(["-R", &source_path, &destination_path])
                .status()
                .expect("failed to execute process");

            assert!(status.success());
        }
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
            fn run_notebook_(test: &str) {
                crate::benchmarking_tests::init_binary();
                crate::benchmarking_tests::create_benchmark_json_file();
                let limitless = false;
                let mut anvil_child = crate::benchmarking_tests::start_anvil(limitless);
                let test_dir: TempDir = TempDir::new("nb").unwrap();
                let path = test_dir.path().to_str().unwrap();
                crate::benchmarking_tests::mv_test_(path, test);
                run_notebooks(path, test);
                test_dir.close().unwrap();
                anvil_child.kill().unwrap();
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

    test_func!();
}
