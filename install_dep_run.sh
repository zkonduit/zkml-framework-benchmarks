#!/bin/bash

# Initialize a flag to track if all dependencies are already installed
all_dependencies_installed=true

# Check Python 3.9
if ! command -v python3.9 &> /dev/null
then
    echo "Python 3.9 not found, installing Python 3.9..."
    brew install python@3.9
    all_dependencies_installed=false
fi

# Check Rust
if ! command -v rustc &> /dev/null
then
    echo "Rust not found, installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    # The script needs to source the cargo environment or restart the terminal
    source $HOME/.cargo/env
    all_dependencies_installed=false
fi

# Check Scarb
if ! command -v scarb &> /dev/null
then
    echo "Scarb not found, installing Scarb..."
    curl --proto '=https' --tlsv1.2 -sSf https://docs.swmansion.com/scarb/install.sh | sh
    all_dependencies_installed=false
fi

# Check if all dependencies were already installed
if [ "$all_dependencies_installed" = true ]; then
    echo "All dependencies (Python 3.9, Rust, and Scarb) are already installed."
fi
