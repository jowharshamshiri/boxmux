source "/etc/machinegenesis/mg_env"
source "$INFRA_DIR/root.sh"

bookworm_read_file_to_array() {
    local filename="$1"
    local -n arr=$2  # Declare arr as a nameref to reference the passed array variable

    # Check if the file exists
    if [[ ! -f "$filename" ]]; then
        log_fatal "File does not exist: $filename"
        return 1
    fi

    # Read the file line by line into the array
    mapfile -t arr < "$filename"
}

bookworm_read_list_file_to_array() {
    local filename="$1"
    local -n arr=$2  # Declare arr as a nameref to reference the passed array variable

    # Check if the file exists
    if [[ ! -f "$filename" ]]; then
        log_fatal "File does not exist: $filename"
        return 1
    fi

    # Clear the array
    arr=()

    # Read the file line by line, trim whitespace, and skip comments and empty lines
    while IFS= read -r line; do
        # Trim leading and trailing whitespace
        line=$(echo "$line" | awk '{$1=$1};1')

        # Skip empty lines and lines starting with '#'
        [[ "$line" == "" || "$line" =~ ^# ]] && continue

        # Append the line to the array
        arr+=("$line")
    done < "$filename"
}

bookworm_install_package() {
    local package_name=$1
    if ! dpkg -l | grep -q $package_name; then
        log_state "Installing package $package_name..."
        sudo apt-get install -y $package_name
    else
        log_debug "Package $package_name is already installed."
    fi
}

bookworm_install_package_repo() {
    local repo_url=$1

    if [ -z "$repo_url" ]; then
        log_fatal "Usage: bookworm_install_package_repo <repo-url>"
        return 1
    fi

    # Debian does not use PPAs, so we only handle regular repos
    if ! grep -qR "^deb .*${repo_url}" /etc/apt/sources.list /etc/apt/sources.list.d/*; then
        log_state "Adding APT repository $repo_url..."
        echo "deb $repo_url bookworm main" | sudo tee /etc/apt/sources.list.d/additional-repos.list
        sudo apt-get update
    else
        log_debug "APT repository $repo_url is already added."
    fi
}

bookworm_install_packages() {
    local packages=("$@")
    local update_apt=${2:-true}

    if [ "$update_apt" = true ]; then
        sudo apt-get update
    fi

    for package in "${packages[@]}"; do
        bookworm_install_package $package
    done
}

bookworm_install_package_repos() {
    local package_repos=("$@")

    for repo in "${package_repos[@]}"; do
        bookworm_install_package_repo $repo
    done
}

bookworm_install_dependencies() {
    declare -a packages
    bookworm_read_list_file_to_array "$PACKAGES_DIR/bookworm.txt" apt_deps_bookworm

    bookworm_install_packages "${apt_deps_bookworm[@]}"
}

bookworm_install_dependency_repos() {
    declare -a package_repos
    bookworm_read_list_file_to_array "$PACKAGE_REPOS_DIR/bookworm.txt" package_repos_bookworm

    bookworm_install_package_repos "${package_repos_bookworm[@]}"
}

bookworm_download_packages() {
    local WORKSPACE=$1
    local DEB_DIR=$2
    local apt_deps_bookworm=("${@:3}")

    if [ -z "$WORKSPACE" ] || [ -z "$DEB_DIR" ] || [ -z "$apt_deps_bookworm" ]; then
        log_fatal "Usage: bookworm_download_packages <workspace> <deb-dir> <apt-dep1> [<apt-dep2> ...]"
        return 1
    fi

    docker run --rm -v $WORKSPACE:/workspace -v $DEB_DIR:/deb -w /workspace debian:12 bash -c "apt update && apt install -y ${apt_deps_bookworm[@]} && cp /var/cache/apt/archives/*.deb /deb"

    if [ $? -ne 0 ]; then
        log_error "Failed to download apt dependencies deb packages."
        exit 1
    fi

    log_state "Apt dependencies deb packages downloaded successfully."
}
