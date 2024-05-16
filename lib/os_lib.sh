#!/usr/bin/env bash

# Detecting specific Linux distributions and macOS versions
detect_distro_and_call_function() {
    local base_function_name=$1
    local argument=$2
    local distro_prefix

    if [[ "$(uname -s)" == "Darwin" ]]; then
        # For macOS, checking the version might require parsing `sw_vers` output
        local macos_version="$(sw_vers -productVersion)"
        if [[ "$macos_version" > "10.14" ]]; then
            distro_prefix="darwin"
        else
            log_error "Unsupported macOS version: $macos_version, must be 10.15 or later"
            return 1
        fi
    elif [[ "$(uname -s)" == "Linux" ]]; then
        # Check for Ubuntu or Debian by examining lsb-release contents
        if grep -q "Ubuntu 22.04" /etc/os-release; then
            distro_prefix="jammy"
        elif grep -q "Debian" /etc/os-release && grep -q "bookworm" /etc/os-release; then
            distro_prefix="bookworm"
        else
            log_error "Unsupported Linux distribution"
            return 1
        fi
    else
        log_error "Unsupported operating system"
        return 1
    fi

    # Build function name and call it if available
    local function_name="${distro_prefix}_${base_function_name}"
    if declare -f "$function_name" >/dev/null; then
        log_debug "Calling function: $function_name"
        "$function_name" "$argument"
    else
        log_error "Function does not exist: $function_name"
        return 1
    fi
}

read_file_to_array() {
    detect_distro_and_call_function "read_file_to_array" "$@"
}

read_list_file_to_array() {
    detect_distro_and_call_function "read_list_file_to_array" "$@"
}

install_package() {
    detect_distro_and_call_function "install_package" "$@"
}

install_package_repo() {
    detect_distro_and_call_function "install_package_repo" "$@"
}

install_packages() {
    detect_distro_and_call_function "install_packages" "$@"
}

install_package_repos() {
    detect_distro_and_call_function "install_package_repos" "$@"
}

install_dependencies() {
    detect_distro_and_call_function "install_dependencies" "$@"
}

install_dependency_repos() {
    detect_distro_and_call_function "install_dependency_repos" "$@"
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
