#!/bin/bash

backup_file() {
    local filepath=$1
    local basepath=$(dirname "$filepath")
    local filename=$(basename "$filepath")

    # Find the next backup number
    local n=0
    while [ -e "$basepath/$filename.bak$n" ]; do
        let n++
    done

    # Copy the current file to the next backup number
    sudo cp "$filepath" "$basepath/$filename.bak$n"
}

fix_virtualbox() {
    sudo chown root:root /usr
    sudo chown root:root /usr/lib
    sudo chown root:root /usr/lib/virtualbox
}

function instantiate_template() {
    local template_file="$1"
    local output_file="$2"

    # Immediately check for the minimum number of arguments
    if [ $# -lt 4 ] || [ $(($# % 2)) -ne 0 ]; then
        log_fatal "Usage: instantiate_template <template-file> <output-file> <placeholder_1> <value_1> [<placeholder_2> <value_2> ...]"
        return 1
    fi

    if [ -z "$template_file" ] || [ -z "$output_file" ]; then
        log_fatal "Usage: instantiate_template <template-file> <output-file> <placeholder_1> <value_1> [<placeholder_2> <value_2> ...]"
        return 1
    fi

    if [ ! -f "$template_file" ]; then
        log_fatal "Template file '$template_file' not found."
        return 1
    fi

    if [ -f "$output_file" ]; then
        log_debug "Output file '$output_file' already exists. It will be overwritten."
    fi

    cp "$template_file" "$output_file" || {
        log_fatal "Failed to copy template file '$template_file' to '$output_file'."
        return 1
    }

    # Process additional arguments for placeholders and values
    shift 2
    while [ $# -gt 0 ]; do
        local placeholder=$1
        local value=$2

        if [ -z "$placeholder" ]; then
            log_fatal "Missing placeholder. Usage: instantiate_template <template-file> <output-file> <placeholder_1> <value_1> [<placeholder_2> <value_2> ...]"
            return 1
        fi

        shift 2 # Move past the processed pair
        log_debug "Replacing '$placeholder' with '$value' in '$output_file'..."
        # Use a command that works both in GNU and BSD sed
        sed -i'' -e "s|$placeholder|$value|g" "$output_file"
    done

    log_debug "Template '$template_file' instantiated to '$output_file'."

    return 0
}

download_file() {

    # Download tables
    # run_duckdb_csv_query "CREATE TABLE IF NOT EXISTS downloads (id INTEGER PRIMARY KEY DEFAULT nextval('seq_a'), url TEXT, expected_checksum TEXT, checksum TEXT, hash_type TEXT, downloaded BOOLEAN, download_time TIMESTAMP);"
    # run_duckdb_csv_query "CREATE TABLE IF NOT EXISTS downloads_links (download_id INTEGER REFERENCES downloads(id), link_path TEXT, PRIMARY KEY (download_id, link_path));"
    # }
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

check_file_hash() {
    local file_path="$1"
    local hash_type="$2"
    local expected_hash="$3"

    # Check for mandatory parameters
    if [ -z "$file_path" ] || [ -z "$hash_type" ] || [ -z "$expected_hash" ]; then
        log_fatal "Usage: check_file_hash <file-path> <hash-type> <expected-hash>"
        return 1
    fi

    # Verify that the provided hash type is supported
    if ! [[ "$hash_type" =~ ^(md5|sha1|sha256)$ ]]; then
        log_fatal "Unsupported hash type '$hash_type'. Supported types: md5, sha1, sha256."
        return 1
    fi

    # Check if the file exists
    if [ ! -f "$file_path" ]; then
        log_error "File '$file_path' not found."
        return 1
    fi

    # Check the hash of the file
    local hash_command="${hash_type}sum"
    if $hash_command -c <(echo "$expected_hash  $file_path") >/dev/null 2>&1; then
        log_debug "File '$file_path' has the correct $hash_type hash."
        return 0
    else
        log_error "File '$file_path' has an incorrect $hash_type hash."
        return 1
    fi
}

get_file_hash() {
    local file_path="$1"
    local hash_type="$2"

    # Check for mandatory parameters
    if [ -z "$file_path" ] || [ -z "$hash_type" ]; then
        log_fatal "Usage: get_file_hash <file-path> <hash-type>"
        return 1
    fi

    # Verify that the provided hash type is supported
    if ! [[ "$hash_type" =~ ^(md5|sha1|sha256)$ ]]; then
        log_fatal "Unsupported hash type '$hash_type'. Supported types: md5, sha1, sha256."
        return 1
    fi

    # Check if the file exists
    if [ ! -f "$file_path" ]; then
        log_error "File '$file_path' not found."
        return 1
    fi

    # Calculate the hash of the file
    local hash_command="${hash_type}sum"
    local file_hash=$($hash_command "$file_path" | awk '{print $1}')

    if [ -z "$file_hash" ]; then
        log_error "Failed to calculate the $hash_type hash of file '$file_path'."
        return 1
    fi

    echo "$file_hash"
}

hash_string() {
    local input_string="$1"
    local hash_type="$2"

    # Check for mandatory parameters
    if [ -z "$input_string" ] || [ -z "$hash_type" ]; then
        log_fatal "Usage: hash_string <input-string> <hash-type>"
        return 1
    fi

    # Verify that the provided hash type is supported
    if ! [[ "$hash_type" =~ ^(md5|sha1|sha256)$ ]]; then
        log_fatal "Unsupported hash type '$hash_type'. Supported types: md5, sha1, sha256."
        return 1
    fi

    # Calculate the hash of the input string
    local hash_command="${hash_type}sum"
    local string_hash=$(echo -n "$input_string" | $hash_command | awk '{print $1}')

    if [ -z "$string_hash" ]; then
        log_error "Failed to calculate the $hash_type hash of the input string."
        return 1
    fi

    echo "$string_hash"
}

update_file_section() {
    local section_file="$1"
    local output_file="$2"
    local start_marker="${3:-$MG_BASHRC_SECTION_START}"
    local end_marker="${4:-$MG_BASHRC_SECTION_END}"
    local comment="${5:-$MG_BASHRC_SECTION_COMMENT}"

    # Set default markers and comment if not provided
    start_marker="${start_marker:-'# BEGIN MANAGED SECTION'}"
    end_marker="${end_marker:-'# END MANAGED SECTION'}"
    comment="${comment:-'# This section is managed by a script'}"

    # Check if the input and output files have been specified
    if [ -z "$section_file" ] || [ -z "$output_file" ]; then
        log_fatal "Missing section file or output file."
        log_fatal "Usage: update_file_section <section-file> <output-file> [start-marker] [end-marker] [comment]"
        return 1
    fi

    if grep -qF "$start_marker" "$output_file"; then
        # The section exists, replace it
        cp "$output_file" "${output_file}.bak" # Create a backup of the original output file

        awk -v start="$start_marker" -v end="$end_marker" -v comment="$comment" -v file="$section_file" '
            BEGIN {print_section=1}
            $0 ~ start {
                print start  # Print the start marker
                print comment  # Print the comment
                print_section=0
                while ((getline line < file) > 0) {
                    print line
                }
                next
            }
            $0 ~ end {print $0; print_section=1; next}
            print_section {print}
        ' "${output_file}.bak" >"$output_file"
    else
        # The section does not exist, append it
        echo "$start_marker" >>"$output_file"
        echo "$comment" >>"$output_file"
        cat "$section_file" >>"$output_file"
        echo "$end_marker" >>"$output_file"
    fi
}

count_occurrences() {
    local string=$1
    local substring=$2
    local count=0
    local tmp=${string}

    while [[ $tmp == *"$substring"* ]]; do
        tmp=${tmp#*"$substring"}
        ((count++))
    done

    echo $count
}

sort_arrays() {
    # Store the first array name separately
    local array1_name=$1
    shift
    local array_names=("$@")

    # Use eval to retrieve the first array by its name
    eval "array1=(\"\${${array1_name}[@]}\")"

    # Retrieve all other arrays
    for name in "${array_names[@]}"; do
        eval "arrays_$name=(\"\${${name}[@]}\")"
    done

    # Get the length of the first array
    local length=${#array1[@]}

    # Bubble sort implementation
    for ((i = 0; i < length; i++)); do
        for ((j = 0; j < length - i - 1; j++)); do
            if ((array1[j] > array1[j + 1])); then
                # Swap elements in array1
                temp=${array1[j]}
                array1[j]=${array1[j + 1]}
                array1[j + 1]=$temp

                # Swap corresponding elements in all other arrays
                for name in "${array_names[@]}"; do
                    eval "temp=\${arrays_$name[j]}"
                    eval "arrays_$name[j]=\${arrays_$name[j + 1]}"
                    eval "arrays_$name[j + 1]=\$temp"
                done
            fi
        done
    done

    # Use eval to update the original arrays
    eval "${array1_name}=(\"\${array1[@]}\")"
    for name in "${array_names[@]}"; do
        eval "${name}=(\"\${arrays_$name[@]}\")"
    done
}

index_of() {
    local value=$1
    shift
    local array=("$@")

    for i in "${!array[@]}"; do
        if [[ "${array[$i]}" == "$value" ]]; then
            echo "$i"
            return 0
        fi
    done

    # echo "-1"
    return 1
}

function update_coredns_forward() {
    local new_dns_ip=$1
    if [[ -z "$new_dns_ip" ]]; then
        log_fatal "Usage: update_coredns_forward <new-dns-ip>"
        return 1
    fi

    # Fetch the current CoreDNS ConfigMap to a local file
    kubectl -n kube-system get configmap coredns -o yaml >/tmp/coredns-config.yaml

    # Check if the file was fetched successfully
    if [[ ! -s /tmp/coredns-config.yaml ]]; then
        log_error "Failed to fetch CoreDNS ConfigMap."
        return 1
    fi

    # Replace the existing forward configuration with the new DNS IP
    awk -v new_dns_ip="$new_dns_ip" '
        /forward ./ { print "        forward . " new_dns_ip; next }
        { print }
    ' /tmp/coredns-config.yaml >/tmp/coredns-config-modified.yaml

    # Apply the modified ConfigMap
    if ! kubectl apply -f /tmp/coredns-config-modified.yaml; then
        log_error "Failed to apply the modified CoreDNS ConfigMap."
        return 1
    fi

    # Restart CoreDNS to apply changes
    if ! kubectl -n kube-system rollout restart deployment/coredns; then
        log_error "Failed to restart CoreDNS deployment."
        return 1
    fi

    log_debug "CoreDNS configuration updated and service restarted successfully."
}

create_link() {
    local target="$1"
    local link_name="$2"

    if [ -z "$target" ] || [ -z "$link_name" ]; then
        log_fatal "Usage: create_link <target> <link-name>"
        return 1
    fi

    if [ -e "$link_name" ]; then
        log_debug "Removing existing link '$link_name'."
        rm -f "$link_name"
    fi

    ln -s "$target" "$link_name" || {
        log_error "Failed to create link '$link_name' pointing to '$target'."
        return 1
    }

    log_debug "Created link '$link_name' pointing to '$target'."
    return 0
}

replace_with_lines() {
    local file=$1
    local pattern=$2
    shift 2
    local replacements=("$@")

    if [ -z "$file" ] || [ -z "$pattern" ] || [ ${#replacements[@]} -eq 0 ]; then
        log_fatal "Usage: replace_with_lines <file> <pattern> <replacement-line-1> [<replacement-line-2> ...]"
        return 1
    fi

    if [ ! -f "$file" ]; then
        log_fatal "File '$file' not found."
        return 1
    fi

    if ! grep -q "$pattern" "$file"; then
        log_error "Pattern '$pattern' not found in file '$file'."
        return 1
    fi

    # Create a temporary file for safer in-place editing
    local tmp_file=$(mktemp)

    # Prepare the replacement text as a single block
    local replacement_text=""
    for line in "${replacements[@]}"; do
        replacement_text+="$line"$'\n'
    done

    # Use sed to find the pattern and replace the whole line with new lines
    sed "/$pattern/c\\
$replacement_text
" "$file" >"$tmp_file"

    # Overwrite the original file with the modified temporary file
    mv "$tmp_file" "$file"
}

setup_ssh_keys_for_user() {
    local user_name="$1"
    local ip_address="$2"

    if [ -z "$user_name" ] || [ -z "$ip_address" ]; then
        log_fatal "Usage: setup_ssh_keys_for_user <user-name> <ip-address>"
        return 1
    fi

    log_debug "Setting up SSH keys for $user_name@$ip_address..."

    local user_home=$(eval echo ~"$user_name")

    if [ -f "$user_home"/.ssh/known_hosts ]; then
        sudo touch "$user_home"/.ssh/known_hosts
        sudo chown "$user_name":"$user_name" "$user_home"/.ssh/known_hosts
        sudo chmod 600 "$user_home"/.ssh/known_hosts
    fi

    sudo ssh-keygen -f "$user_home/.ssh/known_hosts" -R "$ip_address"

    sudo ssh-keyscan -H "$ip_address" | sudo tee -a "$user_home"/.ssh/known_hosts >/dev/null
}

is_loopback_address() {
    local ip_address="$1"

    if [ -z "$ip_address" ]; then
        log_fatal "Usage: is_loopback_address <ip-address>"
        return 1
    fi

    if ! is_ip_valid "$ip_address"; then
        return 1
    fi

    # Check if the IP address starts with "127."
    if [[ "$ip_address" == 127.* ]]; then
        return 0
    else
        return 1
    fi
}

is_ip_valid() {
    local ip_address="$1"

    if [ -z "$ip_address" ]; then
        log_fatal "Usage: is_ip_valid <ip-address>"
        return 1
    fi

    if [[ "$ip_address" =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        log_trace "IP address $ip_address is valid."
        return 0 # IP address is valid
    else
        log_error "IP address $ip_address is not valid."
        return 1 # IP address is not valid
    fi
}

is_ip_reachable() {
    local ip_address="$1"

    if [ -z "$ip_address" ]; then
        log_fatal "Usage: is_ip_reachable <ip-address>"
        return 1
    fi

    if ! is_ip_valid "$ip_address"; then
        return 1
    fi

    if ping -c 1 -W 1 "$ip_address" >/dev/null 2>&1; then
        log_trace "IP address $ip_address is reachable."
        return 0 # IP address is reachable
    else
        log_trace "IP address $ip_address is not reachable."
        return 1 # IP address is not reachable
    fi
}

tcp_port_open_at_ip() {
    local ip_address="$1"
    local port="$2"

    if [ -z "$ip_address" ] || [ -z "$port" ]; then
        log_fatal "Usage: tcp_port_open_at_ip <ip-address> <port>"
        return 1
    fi

    if ! is_ip_reachable "$ip_address"; then
        return 1
    fi

    if nc -z -w1 "$ip_address" "$port"; then
        return 0 # Port is open
    else
        return 1 # Port is closed
    fi
}

udp_port_open_at_ip() {
    local ip_address="$1"
    local port="$2"

    if [ -z "$ip_address" ] || [ -z "$port" ]; then
        log_fatal "Usage: tcp_port_open_at_ip <ip-address> <port>"
        return 1
    fi

    if ! is_ip_reachable "$ip_address"; then
        return 1
    fi

    if nc -z -w1 -u "$ip_address" "$port"; then
        return 0 # Port is open
    else
        return 1 # Port is closed
    fi
}

wait_until_local_port_free() {
    local port="$1"
    local ip_address="${2:-$LOCALHOST_IP}"
    local sleep_seconds="${3:-$DEFAULT_SLEEP_INTERVAL}"
    local timeout="${4:-$DEFAULT_TIMEOUT}"
    local start_time=$(date +%s)

    # Check if port number is provided
    if [[ -z "$port" ]]; then
        log_fatal "Usage: wait_until_local_port_free <port> [ip-address] [interval-in-seconds] [timeout-in-seconds]"
        return 1
    fi

    if ! is_loopback_address "$ip_address"; then
        log_error "IP address $ip_address is not a loopback address."
        return 1
    fi

    # Prepare the grep pattern to handle both regular and interface-specific loopback addresses
    local grep_pattern="${ip_address//./\\.}(:|%).*:$port "

    # Loop until the port is not in use or timeout is reached
    while true; do
        local current_time=$(date +%s)
        local elapsed_time=$((current_time - start_time))

        if [[ "$elapsed_time" -ge "$timeout" ]]; then
            log_error "Port $port at $ip_address is still in use. Timeout reached."
            return 1
        fi

        # Check if the port is free
        local listening_info=$(sudo ss -ltnp | grep -E "$grep_pattern")
        if [[ -z "$listening_info" ]]; then
            log_debug "Port $port at $ip_address is now free."
            return 0
        else
            log_alert "Port $port at $ip_address is still in use. Checking again in $sleep_seconds seconds..."
        fi

        sleep "$sleep_seconds"
    done
}

wait_until_local_port_open() {
    local port="$1"
    local ip_address="${2:-$LOCALHOST_IP}"
    local sleep_seconds="${3:-$DEFAULT_SLEEP_INTERVAL}"
    local timeout="${4:-$DEFAULT_TIMEOUT}"
    local start_time=$(date +%s)

    # Check if port number is provided
    if [[ -z "$port" ]]; then
        log_fatal "Usage: wait_until_local_port_open <port> [ip-address] [interval-in-seconds] [timeout-in-seconds]"
        return 1
    fi

    if ! is_loopback_address "$ip_address"; then
        log_error "IP address $ip_address is not a loopback address."
        return 1
    fi

    # Prepare the grep pattern to handle both regular and interface-specific loopback addresses
    local grep_pattern="${ip_address//./\\.}(:|%).*:$port "

    # Loop until the port is open or timeout is reached
    while true; do
        local current_time=$(date +%s)
        local elapsed_time=$((current_time - start_time))

        if [[ "$elapsed_time" -ge "$timeout" ]]; then
            log_error "Port $port at $ip_address is still closed. Timeout reached."
            return 1
        fi

        local listening_info=$(sudo ss -ltnp | grep -E "$grep_pattern")
        if [[ -n "$listening_info" ]]; then
            log_debug "Port $port at $ip_address is now open."
            return 0
        else
            log_alert "Port $port at $ip_address is still closed. Checking again in $sleep_seconds seconds..."
        fi

        sleep "$sleep_seconds"
    done
}

wait_until_local_port_occupied_by() {
    local port="$1"
    local app_name="$2"
    local ip_address="${3:-$LOCALHOST_IP}"
    local sleep_seconds="${4:-$DEFAULT_SLEEP_INTERVAL}"
    local timeout="${5:-$DEFAULT_TIMEOUT}"
    local start_time=$(date +%s)

    # Check if port number and app name are provided
    if [[ -z "$port" || -z "$app_name" ]]; then
        log_fatal "Usage: wait_until_local_port_occupied_by <port> <app-name> [ip-address] [interval-in-seconds] [timeout-in-seconds]"
        return 1
    fi

    if ! is_loopback_address "$ip_address"; then
        log_error "IP address $ip_address is not a loopback address."
        return 1
    fi

    # Prepare the grep pattern to handle both regular and interface-specific loopback addresses
    local grep_pattern="${ip_address//./\\.}(:|%).*:$port "

    # Loop until the port is open and occupied by the specified app or timeout is reached
    while true; do
        local current_time=$(date +%s)
        local elapsed_time=$((current_time - start_time))

        if [[ "$elapsed_time" -ge "$timeout" ]]; then
            log_error "Port $port at $ip_address is still closed or not occupied by $app_name. Timeout reached."
            return 1
        fi

        local listening_info=$(sudo ss -ltnp | grep -E "$grep_pattern")
        if [[ -n "$listening_info" ]]; then
            local current_app=$(echo "$listening_info" | awk -F'"' '{print $2}')
            if [[ "$current_app" == "$app_name" ]]; then
                log_debug "Port $port at $ip_address is now occupied by $app_name."
                return 0
            fi
        fi

        log_alert "Port $port at $ip_address is not yet occupied by $app_name. Checking again in $sleep_seconds seconds..."
        sleep "$sleep_seconds"
    done
}

wait_until_port_open_at_ip() {
    local ip_address="$1"
    local port="$2"
    local sleep_seconds="${3:-$DEFAULT_SLEEP_INTERVAL}"
    local timeout="${4:-$DEFAULT_TIMEOUT}"
    local start_time=$(date +%s)

    # Check if IP address and port number are provided
    if [[ -z "$ip_address" || -z "$port" ]]; then
        log_fatal "Usage: wait_until_port_open_at_ip <ip-address> <port> [interval-in-seconds] [timeout-in-seconds]"
        return 1
    fi

    # Loop until the port is open or timeout is reached
    while true; do
        local current_time=$(date +%s)
        local elapsed_time=$((current_time - start_time))

        if [[ "$elapsed_time" -ge "$timeout" ]]; then
            log_error "Port $port is still closed at $ip_address. Timeout reached."
            return 1
        fi

        # Check if the port is open at the IP address
        if tcp_port_open_at_ip "$ip_address" "$port"; then
            log_debug "Port $port is now open at $ip_address."
            return 0
        else
            log_alert "Port $port is still closed at $ip_address. Checking again in $sleep_seconds seconds..."
        fi

        sleep "$sleep_seconds"
    done
}

wait_until_port_closed_at_ip() {
    local ip_address="$1"
    local port="$2"
    local sleep_seconds="${3:-$DEFAULT_SLEEP_INTERVAL}"
    local timeout="${4:-$DEFAULT_TIMEOUT}"
    local start_time=$(date +%s)

    # Check if IP address and port number are provided
    if [[ -z "$ip_address" || -z "$port" ]]; then
        log_fatal "Usage: wait_until_port_closed_at_ip <ip-address> <port> [interval-in-seconds] [timeout-in-seconds]"
        return 1
    fi

    # Loop until the port is closed or timeout is reached
    while true; do
        local current_time=$(date +%s)
        local elapsed_time=$((current_time - start_time))

        if [[ "$elapsed_time" -ge "$timeout" ]]; then
            log_error "Port $port is still open at $ip_address. Timeout reached."
            return 1
        fi

        # Check if the port is closed at the IP address
        if ! tcp_port_open_at_ip "$ip_address" "$port"; then
            log_debug "Port $port is now closed at $ip_address."
            return 0
        else
            log_alert "Port $port is still open at $ip_address. Checking again in $sleep_seconds seconds..."
        fi

        sleep "$sleep_seconds"
    done
}

check_domain_resolution() {
    local domain="$1"

    if [ -z "$domain" ]; then
        log_fatal "Usage: check_domain_resolution <domain-name>"
        return 1
    fi

    if nslookup "$domain" >/dev/null 2>&1; then
        return 0 # Domain resolves
    else
        return 1 # Domain does not resolve
    fi
}

wait_until_domain_resolves() {
    local domain="$1"
    local sleep_seconds="${2:-$DEFAULT_SLEEP_INTERVAL}"
    local timeout="${3:-$DEFAULT_TIMEOUT}"
    local start_time=$(date +%s)

    # Check if domain name is provided
    if [[ -z "$domain" ]]; then
        log_fatal "Usage: wait_until_domain_resolves <domain-name> [interval-in-seconds] [timeout-in-seconds]"
        return 1
    fi

    # Loop until the domain resolves or timeout is reached
    while true; do
        local current_time=$(date +%s)
        local elapsed_time=$((current_time - start_time))

        if [[ "$elapsed_time" -ge "$timeout" ]]; then
            log_error "Could not resolve $domain. Timeout reached."
            return 1
        fi

        # Check if the domain resolves
        if check_domain_resolution "$domain"; then
            log_debug "Domain $domain is now resolvable."
            return 0
        else
            log_alert "Domain $domain is not yet resolvable. Checking again in $sleep_seconds seconds..."
        fi

        sleep "$sleep_seconds"
    done
}

check_dns_over_tls_at_ip() {
    local ip_address="$1"
    local domain="${2:-$REFERENCE_DOMAIN}"
    local dot_port="${3:-853}"

    if [ -z "$ip_address" ] || [ -z "$domain" ]; then
        log_fatal "Usage: check_dns_over_tls_at_ip <ip-address> [domain] [dot_port]"
        return 1
    fi

    if kdig +tls -d @"$ip_address" -p "$dot_port" "$domain" >/dev/null 2>&1; then
        log_debug "DNS over TLS at $ip_address:$dot_port is working."
        return 0
    else
        log_error "DNS over TLS at $ip_address:$dot_port is not working."
        return 1
    fi
}

check_domain_resolution_by_dot() {
    local dot_ip="$1"
    local reference_domain="${2:-$REFERENCE_DOMAIN}"
    local dot_port="${3:-853}"

    if [ -z "$dot_ip" ]; then
        log_fatal "Usage: check_domain_resolution_by_dot <dot-ip> [reference-domain] [dot_port]"
        return 1
    fi

    if check_dns_over_tls_at_ip "$dot_ip" "$reference_domain" "$dot_port"; then
        log_debug "Can resolve $reference_domain using DNS over TLS at $dot_ip:$dot_port."
        return 0
    else
        log_alert "Cannot resolve $reference_domain using DNS over TLS at $dot_ip:$dot_port."
        return 1
    fi
}

wait_for_domain_resolution_by_dot() {
    local dot_ip="$1"
    local reference_domain="${2:-$REFERENCE_DOMAIN}"
    local dot_port="${3:-853}"
    local sleep_seconds="${4:-$DEFAULT_SLEEP_INTERVAL}"
    local timeout="${5:-$DEFAULT_TIMEOUT}"
    local start_time=$(date +%s)

    if [ -z "$dot_ip" ]; then
        log_fatal "Usage: wait_for_domain_resolution_by_dot <dot-ip> [reference-domain] [dot_port] [interval-in-seconds] [timeout-in-seconds]"
        return 1
    fi

    while true; do
        local current_time=$(date +%s)
        local elapsed_time=$((current_time - start_time))

        if [[ "$elapsed_time" -ge "$timeout" ]]; then
            log_error "Could not resolve $reference_domain using DNS over TLS at $dot_ip. Timeout reached."
            return 1
        fi

        if check_domain_resolution_by_dot "$dot_ip" "$reference_domain" "$dot_port"; then
            log_debug "Can resolve $reference_domain using DNS over TLS at $dot_ip."
            return 0
        else
            log_alert "Cannot resolve $reference_domain using DNS over TLS at $dot_ip. Checking again in $sleep_seconds seconds..."
        fi

        sleep "$sleep_seconds"
    done
}

check_internet_connection_by_dot() {
    local dot_ip="$1"
    local dot_port="${2:-853}"

    if [ -z "$dot_ip" ]; then
        log_fatal "Usage: check_internet_connection_by_dot <dot-ip> [dot_port]"
        return 1
    fi

    return check_domain_resolution_by_dot "$dot_ip" "$REFERENCE_DOMAIN" "$dot_port"
}

wait_for_internet_connection_by_dot() {
    local dot_ip="$1"
    local dot_port="${2:-853}"
    local sleep_seconds="${3:-$DEFAULT_SLEEP_INTERVAL}"
    local timeout="${4:-$DEFAULT_TIMEOUT}"
    local start_time=$(date +%s)

    if [ -z "$dot_ip" ]; then
        log_fatal "Usage: wait_for_internet_connection_by_dot <dot-ip> [dot_port] [interval-in-seconds] [timeout-in-seconds]"
        return 1
    fi

    # Loop until the internet connection is working or timeout is reached
    while true; do
        local current_time=$(date +%s)
        local elapsed_time=$((current_time - start_time))

        if [[ "$elapsed_time" -ge "$timeout" ]]; then
            log_error "Timeout reached. Stopping check."
            return 1
        fi

        # Check if the internet connection is working
        if check_internet_connection_by_dot "$dot_ip" "$dot_port"; then
            log_debug "Internet connection is now working."
            return 0
        else
            log_alert "Internet connection is not yet working. Checking again in $sleep_seconds seconds..."
        fi

        sleep "$sleep_seconds"
    done
}

check_dns_at_ip() {
    local dns_ip="$1"
    local reference_domain="${2:-$REFERENCE_DOMAIN}"

    if [ -z "$dns_ip" ]; then
        log_fatal "Usage: check_dns_at_ip <dns-ip> [reference-domain]"
        return 1
    fi

    if dig +short @"$dns_ip" "$reference_domain" >/dev/null 2>&1; then
        log_debug "Can resolve $reference_domain using DNS at $dns_ip."
        return 0
    else
        log_alert "Cannot resolve $reference_domain using DNS at $dns_ip."
        return 1
    fi
}

check_domain_resolution_by_dns() {
    local dns_ip="$1"
    local reference_domain="${2:-$REFERENCE_DOMAIN}"

    if [ -z "$dns_ip" ]; then
        log_fatal "Usage: check_domain_resolution_by_dns <dns-ip> [reference-domain]"
        return 1
    fi

    if check_dns_at_ip "$dns_ip" "$reference_domain"; then
        log_debug "Can resolve $reference_domain using DNS at $dns_ip."
        return 0
    else
        log_alert "Cannot resolve $reference_domain using DNS at $dns_ip."
        return 1
    fi
}

wait_for_domain_resolution_by_dns() {
    local dns_ip="$1"
    local reference_domain="${2:-$REFERENCE_DOMAIN}"
    local sleep_seconds="${3:-$DEFAULT_SLEEP_INTERVAL}"
    local timeout="${4:-$DEFAULT_TIMEOUT}"
    local start_time=$(date +%s)

    if [ -z "$dns_ip" ]; then
        log_fatal "Usage: wait_for_domain_resolution_by_dns <dns-ip> [reference-domain] [interval-in-seconds] [timeout-in-seconds]"
        return 1
    fi

    # Loop until the internet connection is working or timeout is reached
    while true; do
        local current_time=$(date +%s)
        local elapsed_time=$((current_time - start_time))

        if [[ "$elapsed_time" -ge "$timeout" ]]; then
            log_error "Could not resolve $reference_domain using DNS at $dns_ip. Timeout reached."
            return 1
        fi

        # Check if the internet connection is working
        if check_domain_resolution_by_dns "$dns_ip" "$reference_domain"; then
            log_debug "Can resolve $reference_domain using DNS at $dns_ip."
            return 0
        else
            log_alert "Cannot resolve $reference_domain using DNS at $dns_ip. Checking again in $sleep_seconds seconds..."
        fi

        sleep "$sleep_seconds"
    done
}

check_internet_connection_by_dns() {
    local dns_ip="$1"

    if [ -z "$dns_ip" ]; then
        log_fatal "Usage: check_internet_connection_by_dns <dns-ip>"
        return 1
    fi

    return check_domain_resolution_by_dns "$dns_ip" "$REFERENCE_DOMAIN"
}

wait_for_internet_connection_by_dns() {
    local dns_ip="$1"
    local sleep_seconds="${2:-$DEFAULT_SLEEP_INTERVAL}"
    local timeout="${3:-$DEFAULT_TIMEOUT}"
    local start_time=$(date +%s)

    if [ -z "$dns_ip" ]; then
        log_fatal "Usage: wait_for_internet_connection_by_dns <dns-ip> [interval-in-seconds] [timeout-in-seconds]"
        return 1
    fi

    # Loop until the internet connection is working or timeout is reached
    while true; do
        local current_time=$(date +%s)
        local elapsed_time=$((current_time - start_time))

        if [[ "$elapsed_time" -ge "$timeout" ]]; then
            log_error "Timeout reached. Stopping check."
            return 1
        fi

        # Check if the internet connection is working
        if check_internet_connection_by_dns "$dns_ip"; then
            log_debug "Internet connection is now working."
            return 0
        else
            log_alert "Internet connection is not yet working. Checking again in $sleep_seconds seconds..."
        fi

        sleep "$sleep_seconds"
    done
}

check_local_systemd_domain_resolution() {
    local reference_domain="${1:-$REFERENCE_DOMAIN}"

    if [ -z "$reference_domain" ]; then
        log_fatal "Usage: check_local_systemd_domain_resolution [reference-domain]"
        return 1
    fi

    if check_domain_resolution_by_dns "127.0.0.53" "$reference_domain"; then
        log_debug "Systemd-resolved is resolving $reference_domain correctly."
        return 0
    else
        log_fatal "Systemd-resolved is not resolving $reference_domain correctly."
        return 1
    fi
}

check_systemd_using_correct_dns() {
    local dns_ip="$1"

    if [ -z "$dns_ip" ]; then
        log_fatal "Usage: check_systemd_using_correct_dns <dns-ip>"
        return 1
    fi

    # Correctly specify the systemd service and the property to check
    if systemctl show -p DNS systemd-resolved | grep -q "DNS=$dns_ip"; then
        log_debug "Systemd-resolved is using the correct DNS server $dns_ip."
        return 0
    else
        log_error "Systemd-resolved is not using the correct DNS server $dns_ip."
        return 1
    fi
}

check_systemd_using_correct_dot() {
    local dot_server="$1"

    if [ -z "$dot_server" ]; then
        log_fatal "Usage: check_systemd_using_correct_dot <dot-server>"
        return 1
    fi

    # Check if DNSOverTLS is enabled
    local tls_setting=$(systemctl show -p DNSOverTLS systemd-resolved | sed 's/DNSOverTLS=//')

    if [ "$tls_setting" != "yes" ] && [ "$tls_setting" != "opportunistic" ]; then
        log_error "DNSOverTLS is not enabled properly. Current setting: $tls_setting"
        return 1
    fi

    # Check if the specified DNS server is configured
    if systemctl show -p DNS systemd-resolved | grep -q "DNS=$dot_server"; then
        log_debug "Systemd-resolved is configured to use the correct DoT server: $dot_server"
        return 0
    else
        log_error "Systemd-resolved is not using the specified DoT server: $dot_server"
        return 1
    fi
}

contains() {
    local e match="$1"
    shift
    for e; do [[ "$e" == "$match" ]] && return 0; done
    return 1
}

print_env() {
    if [ -z "$ENV_FILE" ]; then
        log_fatal "Environment variable 'ENV_FILE' is not set."
        return 1
    fi

    if [ ! -f "$ENV_FILE" ]; then
        log_fatal "File '$env_file' not found."
        return 1
    fi

    log_state "Printing environment variables from '$ENV_FILE':"

    while IFS= read -r line; do
        # Skip empty lines and lines starting with '#'
        [[ "$line" == "" || "$line" =~ ^# ]] && continue

        log_state "$line"
    done <"$ENV_FILE"

    return 0
}

install_yq_with_xml_support() {
    local yq_binary="/usr/local/bin/yq"

    # Check if yq is already installed
    if [ -f "$yq_binary" ] || command -v yq &>/dev/null; then
        log_debug "yq is already installed."
        return 0
    fi

    # Fetch the latest version of yq
    local latest_version=$(curl --silent "https://api.github.com/repos/mikefarah/yq/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' | sed 's/^v//')

    # Determine the yq version to install
    local yq_version="${1:-$latest_version}"

    log_state "Installing yq v$yq_version with XML support..."

    # Temporary directory for download
    local tmp_dir=$(mktemp -d)
    pushd "$tmp_dir" || return 1

    # Download and install yq binary
    wget "https://github.com/mikefarah/yq/releases/download/v${yq_version}/yq_linux_amd64.tar.gz" -O yq.tar.gz -q
    tar -zxf yq.tar.gz
    sudo cp yq_linux_amd64 "$yq_binary"

    # Make yq executable
    sudo chmod +x "$yq_binary"

    popd || exit
    rm -rf "$tmp_dir"

    # Check if yq is installed correctly
    if [ -f "$yq_binary" ]; then
        log_state "yq v$yq_version installed successfully."
    else
        log_fatal "Failed to install yq."
        return 1
    fi
}

manual_setup() {
    install_yq_with_xml_support
}

is_valid_json() {
    local json_string="$1"

    if [ -z "$json_string" ]; then
        log_fatal "Usage: is_valid_json <json-string>"
        return 1
    fi

    if jq -e . >/dev/null 2>&1 <<<"$json_string"; then
        log_trace "JSON string is valid."
        return 0
    else
        log_error "JSON string is not valid."
        return 1
    fi
}

ensure_json_file_exists() {
    local file_path="$1"

    if [ -z "$file_path" ]; then
        log_fatal "Usage: ensure_json_file_exists <file-path>"
        return 1
    fi

    if [ ! -f "$file_path" ]; then
        log_trace "Creating JSON file $file_path..."
        echo "{}" >"$file_path"
    else
        log_trace "JSON file $file_path already exists."

        if [ ! -s "$file_path" ]; then
            log_trace "JSON file $file_path is empty. Adding default content..."
            echo "{}" >"$file_path"
        fi

        if ! is_valid_json "$(cat "$file_path")"; then
            log_error "JSON file $file_path is not valid."
            return 1
        fi
    fi
}

update_json_file() {
    local file_path="$1"
    local key="$2"
    local new_value="$3"
    local create_if_not_exists="${4:-true}"

    if [ -z "$file_path" ] || [ -z "$key" ]; then
        log_fatal "Usage: update_json_file <file-path> <key> [new-value] [create-if-not-exists]"
        return 1
    fi

    if [ "$create_if_not_exists" = "true" ]; then
        ensure_json_file_exists "$file_path"
    fi

    # Check if the JSON file exists
    if [[ ! -f "$file_path" ]]; then
        log_error "Error: JSON file does not exist."
        return 1
    fi

    if ! is_valid_json "$(cat "$file_path")"; then
        log_error "Error: JSON file is not valid."
        return 1
    fi

    if [ -z "$new_value" ]; then
        log_trace "Removing key '$key' from the JSON file."
        # If the new value is not provided, remove the key from the JSON file
        jq --arg key "$key" 'del(.[$key])' "$file_path" >"$file_path.tmp" && mv "$file_path.tmp" "$file_path"
    else
        log_trace "Updating key '$key' in the JSON file $file_path with value '$new_value'."
        # Update the JSON file in-place using jq
        jq --arg key "$key" --arg value "$new_value" \
            '(.[$key] = $value)' "$file_path" >"$file_path.tmp" && mv "$file_path.tmp" "$file_path"
    fi

    if [ $? -eq 0 ]; then
        log_debug "Successfully updated the JSON file $file_path."
        log_trace "Updated JSON file content: \n$(cat "$file_path")"
    else
        log_error "Failed to update the JSON file $file_path."
        return 1
    fi
}

concat_with_separator() {
    local separator="$1"
    shift

    local result=""
    local first=true

    for arg in "$@"; do
        if [ "$first" = true ]; then
            result="$arg"
            first=false
        else
            result="${result}${separator}${arg}"
        fi
    done

    echo -e "$result"
}

split_into_array() {
    local input="$1"
    local separator="$2"
    local -a array=()

    while [[ "$input" ]]; do
        # Extract the part before the separator
        if [[ "$input" == *"$separator"* ]]; then
            array+=("${input%%"$separator"*}")
            input="${input#*"$separator"}"
        else
            array+=("$input")
            break
        fi
    done

    echo "${array[@]}"
}

read_json_file() {
    local file_path="$1"
    local key="$2"

    if [ -z "$file_path" ] || [ -z "$key" ]; then
        log_fatal "Usage: read_json_file <file-path> <key>"
        return 1
    fi

    if [[ ! -f "$file_path" ]]; then
        log_error "Error: JSON file does not exist."
        return 1
    fi

    if ! is_valid_json "$(cat "$file_path")"; then
        log_error "Error: JSON file is not valid."
        return 1
    fi

    # Read the value of the key from the JSON file
    local value=$(jq -r ".$key" "$file_path")

    if [ -z "$value" ] || [ "$value" == "null" ]; then
        log_error "Error: Key '$key' not found in the JSON file."
        return 1
    fi

    if [ "$value" == "null" ]; then
        value=""
    fi

    echo "$value"
}

write_to_run_state() {
    local key="$1"
    local value="$2"

    log_trace "Writing to run state file $RUN_STATE_FILE: $key=$value"
    update_json_file "$RUN_STATE_FILE" "$key" "$value"

    if [ $? -eq 0 ]; then
        log_debug "Successfully wrote to run state file."
    else
        log_error "Failed to write to run state file."
        return 1
    fi
}

read_from_run_state() {
    local key="$1"
    result=$(read_json_file "$RUN_STATE_FILE" "$key")
    if [ $? -eq 0 ]; then
        log_trace "Successfully read from run state file."
        echo "$result"
    else
        log_trace "Failed to read from run state file."
        return 1
    fi
}

remove_whitespace_and_control_chars() {
    local input="$1"
    echo "$input" | tr -d '[:space:]' | tr -d '[:cntrl:]'
}

is_set_in_run_state() {
    local key="$1"

    if [ -z "$key" ]; then
        log_fatal "Usage: is_set_in_run_state <key>"
        return 1
    fi

    local value=$(read_from_run_state "$key")

    if [ $? -eq 0 ] && [ -n "$value" ] && [ "$value" != "null" ]; then
        return 0
    else
        return 1
    fi
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
