#!/bin/bash

ceil() {
    local a="$1"
    local b="$2"

    # Perform integer division
    local quotient=$((a / b))
    local remainder=$((a % b))

    # If there's any remainder, increment quotient to achieve ceiling
    if [ "$remainder" -ne 0 ]; then
        quotient=$((quotient + 1))
    fi

    # Return the result
    echo "$quotient"
}

floor() {
    local a="$1"
    local b="$2"

    # Perform integer division
    local quotient=$((a / b))

    # Return the result
    echo "$quotient"
}

source ~/.xbashrc

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    if [ -z "$1" ]; then
        # No function name supplied, do nothing
        exit 0
    fi

    func_name="$1" # Store the first argument (function name)
    shift          # Remove the first argument, now $@ contains only the arguments for the function

    # Check if the function exists
    if declare -f "$func_name" >/dev/null; then
        "$func_name" "$@" # Call the function with the remaining arguments
    else
        log_fatal "'$func_name' is not a valid function name."
        exit 1
    fi
fi
