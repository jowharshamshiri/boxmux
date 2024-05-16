#!/usr/bin/env bash

download_exists() {
    local download_url="$1"
    local download_id

    download_id=$(run_duckdb_csv_query "SELECT id FROM downloads WHERE url='$download_url';" | tail -n +2)
    if [[ -n "$download_id" ]]; then
        echo 0
    else
        echo 1
    fi
}

was_downloaded() {
    local download_url="$1"
    local downloaded
    downloaded=$(run_duckdb_csv_query "SELECT downloaded FROM downloads WHERE url='$download_url';" | tail -n +2)
    [[ "$downloaded" == "TRUE" ]] && return 0 || return 1
}

set_downloaded() {
    local download_url="$1"
    local downloaded=${2:-TRUE}
    run_duckdb_csv_query "UPDATE downloads SET downloaded=$downloaded WHERE url='$download_url';"
}

add_download() {
    local url="$1"
    local expected_checksum="$2"
    local hash_type=${3:-sha256}
    local downloaded=${4:-FALSE}
    local download_time=${5:-$(date +"%Y-%m-%d %H:%M:%S")}

    if download_exists "$url"; then
        log_debug "Download already exists: $url"
        return 1
    else
        run_duckdb_csv_query "INSERT INTO downloads (url, expected_checksum, hash_type, downloaded, download_time) VALUES ('$url', '$expected_checksum', '$hash_type', '$downloaded', '$download_time');"
    fi
}

get_download_id() {
    local download_url="$1"
    local download_id
    download_id=$(run_duckdb_csv_query "SELECT id FROM downloads WHERE url='$download_url';" | tail -n +2)
    echo "$download_id"
}

get_download_expected_checksum() {
    local download_url="$1"
    local download_checksum
    download_checksum=$(run_duckdb_csv_query "SELECT expected_checksum FROM downloads WHERE url='$download_url';" | tail -n +2)
    echo "$download_checksum"
}

get_download_checksum_type() {
    local download_url="$1"
    local download_checksum_type
    download_checksum_type=$(run_duckdb_csv_query "SELECT hash_type FROM downloads WHERE url='$download_url';" | tail -n +2)
    echo "$download_checksum_type"
}

download_file() {
    local url="$1"
    local output_path="$2"
    local hash_type="$3"
    local expected_hash="$4"

    local hash_check=false

    # Check for mandatory parameters
    if [ -z "$url" ] || [ -z "$output_path" ]; then
        log_fatal "Usage: download_file_to_path <url> <output-path> [hash-type] [expected-hash]"
        return 1
    fi

    # Check if hash type is provided without an expected hash
    if [ -n "$hash_type" ] && [ -z "$expected_hash" ]; then
        log_fatal "Hash type is provided but expected hash is missing."
        return 1
    else
        hash_check=true
    fi

    # Verify that the provided hash type is supported
    if [ -n "$hash_type" ] && ! [[ "$hash_type" =~ ^(md5|sha1|sha256)$ ]]; then
        log_fatal "Unsupported hash type '$hash_type'. Supported types: md5, sha1, sha256."
        return 1
    fi

    # Check if file exists and compare hashes if necessary
    if [ -f "$output_path" ]; then
        if [ -n "$hash_type" ] && [ -n "$expected_hash" ]; then
            local hash_command="${hash_type}sum"
            if $hash_command -c <(echo "$expected_hash  $output_path") >/dev/null 2>&1; then
                log_debug "File '$output_path' already exists and has the correct $hash_type hash. Skipping download."
                return 0
            else
                log_debug "File '$output_path' already exists but has an incorrect $hash_type hash."
            fi
        else
            log_debug "File '$output_path' already exists and no hash check is requested. Skipping download."
            return 0
        fi
    fi

    # Proceed to download the file
    if curl -sSL -o "$output_path" "$url"; then
        log_debug "Downloaded file from '$url' to '$output_path'."
        if [ -n "$hash_type" ] && [ -n "$expected_hash" ]; then
            if hash_check; then
                local hash_command="${hash_type}sum"
                if $hash_command -c <(echo "$expected_hash  $output_path") >/dev/null 2>&1; then
                    log_debug "File '$output_path' has the correct $hash_type hash."
                else
                    log_error "File '$output_path' has an incorrect $hash_type hash. Removing the file."
                    rm -f "$output_path"
                    return 1
                fi
            fi
        fi
        return 0
    else
        log_error "Failed to download file from '$url' to '$output_path'."
        if [ -f "$output_path" ]; then
            log_debug "Removing the partially downloaded file."
            rm -f "$output_path"
        fi
        return 1
    fi
}

calc_download_path() {
    local download_url="$1"
    echo "$DOWNLOADS_DIR/$(hash_string "$download_url" "sha256")"
}

download() {
    local download_url="$1"
    local download_id
    download_id=$(get_download_id "$download_url")
    if [[ -z "$download_id" ]]; then
        log_error "Download does not exist: $download_url"
        return 1
    fi

    local download_link_path="$2"

    local download_path=$(calc_download_path "$download_url")

    local expected_checksum=$(get_download_expected_checksum "$download_url")
    local expected_checksum_type=$(get_download_checksum_type "$download_url")

    if was_downloaded "$download_url"; then
        if [[ -f "$download_path" ]]; then
            if [ -n "$expected_checksum" ]; then
                if check_file_hash "$download_path" "$expected_checksum_type" "$expected_checksum"; then
                    log_debug "Download already completed: $download_url"
                    return 0
                else
                    log_error "Download checksum mismatch: $download_url"
                    log_debug "Removing downloaded file: $download_path"
                    rm -f "$download_path"
                    set_downloaded "$download_url" FALSE
                fi
            fi
        else
            log_debug "Download marked as completed but file not found: $download_url"
            set_downloaded "$download_url" FALSE
        fi
    fi

    if ! was_downloaded "$download_url"; then
        download_file "$download_url" "$download_path" "$expected_checksum_type" "$expected_checksum"
        if [ $? -ne 0 ]; then
            log_error "Failed to download: $download_url"
            return 1
        else
            set_downloaded "$download_url"
            checksum=$(get_file_hash "$download_path" "$expected_checksum_type")
            run_duckdb_csv_query "UPDATE downloads SET checksum='$checksum' WHERE url='$download_url';"
        fi
    fi

    if [[ -n "$download_link_path" ]]; then
        ln -s "$download_path" "$download_link_path"
    fi
}

add_download_and_download() {
    local download_url="$1"
    local expected_checksum="$2"
    local hash_type=${3:-sha256}
    local download_link_path="$4"

    add_download "$download_url" "$expected_checksum" "$hash_type"
    download "$download_url" "$download_link_path"
}

get_download_path() {
    local download_url="$1"
    local download_path
    download_path=$(run_duckdb_csv_query "SELECT download_path FROM downloads WHERE url='$download_url';" | tail -n +2)
    echo "$download_path"
}

add_download_link() {
    local download_url="$1"
    local link_path="$2"
    local download_id
    download_id=$(get_download_id "$download_url")
    if [[ -z "$download_id" ]]; then
        log_error "Download does not exist: $download_url"
        return 1
    fi

    if [[ -z "$link_path" ]]; then
        log_error "Link path is missing."
        return 1
    fi

    if [[ -e "$link_path" ]]; then
        log_error "Link path already exists: $link_path"
        return 1
    fi

    run_duckdb_csv_query "INSERT INTO downloads_links (download_id, link_path) VALUES ($download_id, '$link_path');"

    if was_downloaded "$download_url"; then
        log_debug "Linking download: $download_url -> $link_path"
        ln -s "$(calc_download_path "$download_url")" "$link_path"
    fi
}

create_download_links() {
    local download_url="$1"
    local download_id
    download_id=$(get_download_id "$download_url")
    if [[ -z "$download_id" ]]; then
        log_error "Download does not exist: $download_url"
        return 1
    fi

    if ! was_downloaded "$download_url"; then
        log_error "Download not completed: $download_url, cannot create links."
        return 1
    fi

    download_path=$(calc_download_path "$download_url")

    # Get all link paths for the download and create the links
    local link_paths
    link_paths=$(run_duckdb_csv_query "SELECT link_path FROM downloads_links WHERE download_id=$download_id;" | tail -n +2)
    for link_path in $link_paths; do
        log_debug "Linking download: $download_url -> $link_path"
        ln -s "$download_path" "$link_path"
    done
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
