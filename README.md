# ZKML-Framework-Comparisons

## Getting started

To run the benchmarks, you need to first install python (version 3.9.18 specifically), rust and scarb on your machine. If you are using a mac, execute the `install_dep_run.sh` at the root of the directory to install of these dependencies in one go. 

After that we need to setup our python virtual environment:

```bash
# Setup Python virtual environment
python3.9 -m venv venv
source venv/bin/activate
```

Finally run this cargo nextest test command to get the benchmarks:

```bash
source .env/bin/activate; cargo nextest run benchmarking_tests::tests::run_benchmarks_ --test-threads 1
```

The data will stored in a `benchmarks.json` file in the root directory.

If you run into any issues feel free to open a PR and we will try to help you out ASAP. 

Enjoy! :)

