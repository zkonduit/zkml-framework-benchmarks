#!/bin/bash

# Initialize a flag to track if all dependencies are already installed
all_dependencies_installed=true

# Function to install pyenv
install_pyenv() {
    echo "Installing pyenv..."
    curl https://pyenv.run | bash

    # Add pyenv to path and initialize immediately after installation
    export PATH="$HOME/.pyenv/bin:$PATH"
    eval "$(pyenv init --path)"
    eval "$(pyenv virtualenv-init -)"
}

# Function to setup Python environment
setup_python_env() {
    echo "Setting up Python 3.9 environment..."
    sudo apt-get update
    sudo apt-get install libbz2-dev libsqlite3-dev liblzma-dev
    pyenv uninstall 3.9.18
    pyenv install 3.9.18
    pyenv local 3.9.18
    python -m venv .env
    source .env/bin/activate
    echo "Python 3.9 environment setup complete. Run 'deactivate' to deactivate the virtual environment. Run 'source .env/bin/activate' to activate the virtual environment."
}

# Main installation function
install_python() {
    install_pyenv
    setup_python_env
}


# Check if pyenv is installed, if not, install everything
if ! command -v pyenv &> /dev/null
then
    all_dependencies_installed=false
    install_python
else
    echo "pyenv is already installed. Setting up a Python 3.9 environment for the folder..."
    all_dependencies_installed=false
    setup_python_env
fi

# Install jq
if ! command -v jq &> /dev/null
then
    echo "jq not found, installing jq..."
    yes | sudo apt-get install jq
    all_dependencies_installed=false
else
    echo "jq is already installed."
fi

# Install SQLite3
if ! command -v sqlite3 &> /dev/null
then
    echo "sqlite3 not found, installing sqlite3..."
    yes | sudo apt-get install sqlite3
    all_dependencies_installed=false
else
    echo "sqlite3 is already installed."
fi

if rustup component list --installed | grep -q rust-src; then
    rustup component remove rust-src
fi
rustup component add rust-src


# Check Rust
if ! command -v rustc &> /dev/null; then
    echo "Rust not found, installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
    rustup toolchain remove stable
    rustup toolchain install stable

    # Check if Rust installation was successful
    if ! command -v rustc &> /dev/null; then
        echo "Rust installation failed."
        exit 1
    fi

    all_dependencies_installed=false
else
    echo "Rust is already installed."
fi

# Check if cargo binstall is installed
if ! command -v cargo-binstall &> /dev/null
then
    echo "cargo-binstall not found, installing ..."
    cargo install cargo-binstall
    all_dependencies_installed=false
fi

# Check if cargo-nextest is installed
if ! command -v cargo-nextest &> /dev/null
then
    echo "nextest toolchain not found, installing ..."
    yes | cargo binstall cargo-nextest --secure
    all_dependencies_installed=false
fi

# Check if cargo-risczero is installed
if ! command -v cargo-risczero &> /dev/null
then
    echo "risczero toolchain not found, installing ..."
    yes | cargo binstall cargo-risczero
    cargo risczero install
    all_dependencies_installed=false
fi

# Check Scarb
if ! command -v scarb &> /dev/null
then
    echo "Scarb not found, installing Scarb..."
    curl --proto '=https' --tlsv1.2 -sSf https://docs.swmansion.com/scarb/install.sh | sh
    all_dependencies_installed=false
fi

# Install Rust jupyter kernel

source $HOME/.cargo/env
cargo install evcxr_jupyter
evcxr_jupyter --install

# Check if all dependencies were already installed
if [ "$all_dependencies_installed" = true ]; then
    echo "All dependencies (Python 3.9, Rust, and Scarb) are already installed."
fi
