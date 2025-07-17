#!/bin/bash

# This script runs boxmux via cargo run and passes all arguments to the binary
# Usage: ./run_boxmux.sh layouts/dashboard.yaml [additional args]

cargo run -- "$@"