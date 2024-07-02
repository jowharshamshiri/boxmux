#!/bin/bash

copy_dir_to_host() {
    local host=$1
    local local_dir=$2
    local remote_dir="/tmp/deb"

    if [ -z "$host" ] || [ -z "$local_dir" ] || [ -z "$remote_dir" ]; then
        log_fatal "Usage: copy_dir_to_host <host> <local_dir> <remote_dir>"
        exit 1
    fi

    ssh "$host" "mkdir -p $remote_dir"
    scp -r "$local_dir"/*.deb "$host":$remote_dir

    if [ $? -ne 0 ]; then
        log_error "Failed to copy $local_dir to host $host:$remote_dir."
        exit 1
    fi

    log_state "Directory $local_dir copied to host $host:$remote_dir successfully."
}

determine_remote_distro() {
    local host=$1

    local remote_os=$(ssh "$host" "uname -s && grep '^ID=' /etc/os-release && grep 'VERSION_ID=' /etc/os-release")
    case "$remote_os" in
    *Darwin*) echo "darwin" ;;
    *Ubuntu*) echo "jammy" ;;
    *Debian*) echo "bookworm" ;;
    *)
        log_error "Unsupported OS detected: $remote_os"
        return 1
        ;;
    esac
}

# determine_if_boxmux_installed() {

# }

call_remote_function() {
    local host=$1
    local base_function_name=$2
    local argument=$3
    local distro_prefix

    distro_prefix=$(determine_remote_distro "$host")

    if [ $? -ne 0 ] || [ -z "$distro_prefix" ]; then
        log_error "Failed to determine remote distro."
        exit 1
    fi

    local function_name="${distro_prefix}_${base_function_name}"

    if declare -f "$function_name" >/dev/null; then
        log_debug "Calling function: $function_name"
        ssh "$host" "$function_name $argument"
    else
        log_error "Function does not exist: $function_name"
        return 1
    fi
}

download_package_if_missing() {
    local package=$1
    local distro_prefix=$2
    local download_dir=$3

    if [ ! -f "$download_dir/$package" ]; then
        log_info "Package $package not found, attempting to download..."
        local download_function="${distro_prefix}_download_package"
        if declare -f "$download_function" >/dev/null; then
            "$download_function" "$package" "$download_dir"
        else
            log_error "Download function does not exist: $download_function"
            return 1
        fi
    fi
}

install_packages_on_remote_host() {
    local HOST=$1
    local packages_dir=$2
    shift 2
    local packages=("$@")

    local distro_prefix=$(determine_remote_distro "$HOST")
    if [ -z "$distro_prefix" ]; then
        log_error "Failed to determine remote distro."
        return 1
    fi

    # Prepare a temp directory for packaging
    local tmp_dir="/tmp/packages_to_install_$$"
    mkdir -p "$tmp_dir" || log_fatal "Failed to create temp directory $tmp_dir."

    # Determine package files to copy
    if [ ${#packages[@]} -eq 0 ]; then
        # No specific packages given, use all appropriate package files in the directory
        packages=($(ls "$packages_dir"/*."${distro_prefix}"))
    fi

    for pkg in "${packages[@]}"; do
        local pkg_file=$(basename "$pkg")
        if [ -f "$packages_dir/$pkg_file" ]; then
            cp "$packages_dir/$pkg_file" "$tmp_dir/"
        else
            download_package_if_missing "$pkg_file" "$distro_prefix" "$tmp_dir"
        fi
    done

    # Copy to remote host
    scp -r "$tmp_dir" "$HOST:/tmp/"
    if [ $? -ne 0 ]; then
        log_error "Failed to copy packages to the remote host."
        rm -rf "$tmp_dir"
        return 1
    fi

    # Install packages on the remote host
    ssh "$HOST" "sudo dpkg -i /tmp/$(basename $tmp_dir)/*.deb" # Adapt this line for actual package installation commands

    # Cleanup
    rm -rf "$tmp_dir"
    ssh "$HOST" "rm -rf /tmp/$(basename $tmp_dir)"
}

bookworm_install_downloaded_packages_on_remote_host() {
    local host=$1
    local DEB_DIR=$2

    ssh "$host" "dpkg -i /tmp/*.deb"

    if [ $? -ne 0 ]; then
        log_error "Failed to install apt dependencies deb packages on remote host."
        exit 1
    fi

    log_state "Apt dependencies deb packages installed on remote host successfully."
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
