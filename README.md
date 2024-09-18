# ZKML-Framework-Comparisons

## Getting started

To run the benchmarks, you need to first install python (version 3.9.18 specifically), rust, rust jupyter kernel, risc0 toolchain, and scarb on your unix-like machine.

Install the latest ezkl version with the following command
```bash
curl https://raw.githubusercontent.com/zkonduit/ezkl/main/install_ezkl_cli.sh | bash
```

To install the other required dependencies run: 

```bash
bash install_dep_run.sh
```

For windows systems, you will need to install the dependencies manually.

For linux systems, you may need to install jq.

```bash
sudo apt-get install jq
```

You may run the following to activate the virtual environment if had been deactivated.

```bash
source .env/bin/activate
```

For linux systems you will also need to set the OS environment variable to linux (default is Mac).

```bash
export OS=linux
```

Finally run this cargo nextest test command to get the benchmarks:

```bash
source .env/bin/activate; cargo nextest run benchmarking_tests::tests::run_benchmarks_ --no-capture
```

The data will stored in a `benchmarks.json` file in the root directory.

If you run into any issues feel free to open a PR and we will try to help you out ASAP. 

Enjoy! :)

