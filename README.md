# ZKML-Framework-Comparisons

## Getting started

To run the benchmarks, you need to first install python and rust on your machine. Then, you can run the following commands to install the dependencies and run the benchmarks. The data will stored in a `benchmarks.json` file in the root directory.

### Setup Virtual Python Env

```bash
python3 -m venv venv; source .env/bin/activate;
```

### Run Benchmarks

```bash
source .env/bin/activate; cargo nextest run benchmarking_tests::tests::run_benchmarks_ --test-threads 1
```