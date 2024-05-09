source "/etc/machinegenesis/mg_env"
source "$INFRA_DIR/root.sh"

darwin_read_file_to_array() {
    log_trace "function darwin_read_file_to_array()"
    local filename="$1"
    local -n arr=$2

    if [[ ! -f "$filename" ]]; then
        log_fatal "File does not exist: $filename"
        return 1
    fi

    arr=()
    while IFS= read -r line || [[ -n "$line" ]]; do
        arr+=("$line")
    done < "$filename"
}

darwin_read_list_file_to_array() {
    log_trace "function darwin_read_list_file_to_array()"
    local filename="$1"
    local arr_name=$2

    if [[ ! -f "$filename" ]]; then
        log_fatal "File does not exist: $filename"
        return 1
    fi

    # Reset the array using eval
    eval "$arr_name=()"

    local line
    while IFS= read -r line || [[ -n "$line" ]]; do
        line=$(echo "$line" | awk '{$1=$1};1')  # Trim spaces using awk
        [[ "$line" == "" || "$line" =~ ^# ]] && continue
        # Append to the array using eval
        eval "$arr_name+=(\"\$line\")"
    done < "$filename"
}

darwin_install_package() {
    log_trace "function darwin_install_package()"
    local package_name=$1
    if ! brew list --formula | grep -q "^${package_name}$"; then
        log_state "Installing package $package_name..."
        brew install $package_name
    else
        log_debug "Package $package_name is already installed."
    fi
}

darwin_install_packages() {
    log_trace "function darwin_install_packages()"
    local packages=("$@")
    for package in "${packages[@]}"; do
        darwin_install_package $package
    done
}

darwin_install_dependencies() {
    log_trace "function darwin_install_dependencies()"
    declare -a packages
    darwin_read_list_file_to_array "$PACKAGES_DIR/darwin.txt" packages

    darwin_install_packages "${packages[@]}"
}

darwin_install_package_repo() {
    log_trace "function darwin_install_package_repo()"
    local repo_url=$1  # This should be in the format of 'user/repo'

    if [ -z "$repo_url" ]; then
        log_fatal "Usage: darwin_install_package_repo <user/repo>"
        return 1
    fi

    if ! brew tap | grep -q "^${repo_url}$"; then
        log_state "Adding Homebrew tap $repo_url..."
        brew tap $repo_url
    else
        log_debug "Homebrew tap $repo_url is already added."
    fi
}

darwin_install_package_repos() {
    log_trace "function darwin_install_package_repos()"
    local package_repos=("$@")

    for repo in "${package_repos[@]}"; do
        darwin_install_package_repo $repo
    done
}

darwin_install_dependency_repos() {
    log_trace "function darwin_install_dependency_repos()"
    declare -a package_repos
    darwin_read_list_file_to_array "$PACKAGE_REPOS_DIR/darwin.txt" package_repos

    darwin_install_package_repos "${package_repos[@]}"
}

darwin_download_homebrew_packages() {
    log_trace "function darwin_download_homebrew_packages()"
    local download_dir=$1
    local packages=("$@")

    for package in "${packages[@]}"; do
        if [ ! -f "$download_dir/$package" ]; then
            log_info "Package $package not found, attempting to download..."
            brew fetch --force-bottle $package
        fi
    done
}
