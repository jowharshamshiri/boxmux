source "/etc/machinegenesis/mg_env"
source "$INFRA_DIR/root.sh"

jammy_read_file_to_array() {
    local filename="$1"
    local -n arr=$2 # Declare arr as a nameref, to use it as a reference to the passed array variable

    # Check if the file exists
    if [[ ! -f "$filename" ]]; then
        log_fatal "File does not exist."
        return 1
    fi

    # Read file line by line into the array
    mapfile -t arr < "$filename" # mapfile (or readarray) reads lines from a file into an array
}

jammy_read_list_file_to_array() {
    local filename="$1"
    local -n arr=$2 # Use a nameref to refer to the array variable provided in the argument

    # Check if the file exists
    if [[ ! -f "$filename" ]]; then
        log_fatal "File does not exist."
        return 1
    fi

    # Clear the array
    arr=()

    # Read file line by line, trim whitespace, and skip comments and empty lines
    while IFS= read -r line; do
        # Trim leading and trailing whitespace
        line=$(echo "$line" | awk '{$1=$1};1')

        # Skip empty lines and lines starting with '#'
        [[ "$line" == "" || "$line" =~ ^# ]] && continue

        # Append the line to the array
        arr+=("$line")
    done < "$filename"
}

jammy_install_package() {
    local package_name=$1
    if ! dpkg -l | grep -q $package_name; then
        log_state "Installing package $package_name..."
        sudo apt-get install -y $package_name
    else
        log_debug "Package $package_name is already installed."
    fi
}

jammy_install_packages() {
    local packages=("$@")
    local update_apt=${2:-true}

    if [ "$update_apt" = true ]; then
        sudo apt-get update
    fi

    for package in "${packages[@]}"; do
        jammy_install_package $package
    done
}

jammy_install_dependencies() {
    declare -a packages
    jammy_read_list_file_to_array "$PACKAGES_DIR/jammy.txt" apt_deps_jammy

    jammy_install_packages "${apt_deps_jammy[@]}"
}

jammy_install_package_repo() {
    local repo_url=$1

    if [ -z "$repo_url" ]; then
        log_fatal "Usage: jammy_install_package_repo <repo-url>"
        return 1
    fi

    # Determine if the repository URL is a PPA
    if [[ "$repo_url" =~ ^ppa: ]]; then
        # It's a PPA, so parse out the username/repository part
        local ppa_name="${repo_url#ppa:}"
        local ppa_search_term="http://ppa.launchpad.net/${ppa_name}/"

        # Adjust grep to search for PPA format in the sources
        if ! grep -qR "^deb .*${ppa_search_term}" /etc/apt/sources.list /etc/apt/sources.list.d/*; then
            log_state "Adding PPA repository $repo_url..."
            sudo add-apt-repository -y "$repo_url"
        else
            log_debug "PPA repository $repo_url is already added."
        fi
    else
        # Regular repository URL, adjust grep to search directly for the repo_url
        if ! grep -qR "^deb .*${repo_url}" /etc/apt/sources.list /etc/apt/sources.list.d/*; then
            log_state "Adding APT repository $repo_url..."
            sudo add-apt-repository -y "$repo_url"
        else
            log_debug "APT repository $repo_url is already added."
        fi
    fi
}

jammy_install_package_repos() {
    local package_repos=("$@")

    for repo in "${package_repos[@]}"; do
        jammy_install_package_repo $repo
    done
}

jammy_install_dependency_repos() {
    declare -a package_repos
    jammy_read_list_file_to_array "$PACKAGES_DIR/jammy.txt" package_repos

    jammy_install_package_repos "${package_repos[@]}"
}

jammy_download_packages() {
    local WORKSPACE=$1
    local DEB_DIR=$2
    local apt_deps_bookworm=("${@:3}")

    docker run --rm -v $WORKSPACE:/workspace -v $DEB_DIR:/deb -w /workspace ubuntu:22.04 bash -c "apt update && apt install -y ${apt_deps_bookworm[@]} && cp /var/cache/apt/archives/*.deb /deb"

    if [ $? -ne 0 ]; then
        log_error "Failed to download apt dependencies deb packages."
        exit 1
    fi

    log_state "Apt dependencies deb packages downloaded successfully."
}
