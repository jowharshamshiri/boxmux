#!/usr/bin/env bash

# Function to detect OS and architecture, and download the appropriate DuckDB binary
download_duckdb() {
    local url=""
    if [[ "$OSTYPE" == "darwin"* ]]; then
        url="https://github.com/duckdb/duckdb/releases/download/v0.10.2/duckdb_cli-osx-universal.zip"
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        local arch
        arch=$(uname -m)
        if [[ "$arch" == "x86_64" ]]; then
            url="https://github.com/duckdb/duckdb/releases/download/v0.10.2/duckdb_cli-linux-amd64.zip"
        elif [[ "$arch" == "aarch64" ]]; then
            url="https://github.com/duckdb/duckdb/releases/download/v0.10.2/duckdb_cli-linux-aarch64.zip"
        else
            echo "Unsupported architecture: $arch"
            return 1
        fi
    else
        echo "Unsupported OS: $OSTYPE"
        return 1
    fi

    echo "Downloading DuckDB binary from $url..."
    curl -L "$url" -o duckdb.zip
    unzip duckdb.zip -d duckdb_bin
    mv duckdb_bin/duckdb* duckdb
    chmod +x duckdb
    rm -rf duckdb.zip duckdb_bin
    echo "DuckDB binary downloaded and ready to use."
}

# Function to initialize the DuckDB database
initialize_db() {
    ./duckdb mapdata.db -csv "CREATE TABLE IF NOT EXISTS maps (map_name TEXT, key TEXT, value TEXT, PRIMARY KEY (map_name, key));"
    ./duckdb mapdata.db -csv "CREATE TABLE IF NOT EXISTS map_names (map_name TEXT PRIMARY KEY);"
}

# Function to initialize a map
map_init() {
    local map_name="$1"
    ./duckdb mapdata.db -csv "INSERT INTO map_names (map_name) VALUES ('$map_name') ON CONFLICT DO NOTHING;"
}

# Function to check if a map exists
map_exists() {
    local map_name="$1"
    local count
    count=$(./duckdb mapdata.db -csv "SELECT COUNT(*) FROM map_names WHERE map_name='$map_name';" | tail -n +2)
    if [ "$count" -eq 0 ]; then
        echo "Map '$map_name' does not exist. Please initialize it first." >&2
        return 1
    else
        return 0
    fi
}

# Function to clear all entries from a map
map_clear() {
    local map_name="$1"
    ./duckdb mapdata.db -csv "DELETE FROM maps WHERE map_name='$map_name';"
}

# Function to add or set a key-value pair in the map
map_add_or_set() {
    local map_name="$1"
    local key="$2"
    local value="$3"
    map_exists "$map_name" || return 1
    ./duckdb mapdata.db -csv "INSERT INTO maps (map_name, key, value) VALUES ('$map_name', '$key', '$value') ON CONFLICT(map_name, key) DO UPDATE SET value=excluded.value;"
}

# Function to get the value for a key in the map
map_get() {
    local map_name="$1"
    local key="$2"
    map_exists "$map_name" || return 1
    local value
    value=$(./duckdb mapdata.db -csv "SELECT value FROM maps WHERE map_name='$map_name' AND key='$key';" | tail -n +2)
    if [ -n "$value" ]; then
        echo "$value"
    else
        return 1
    fi
}

# Function to check if a key exists in the map
map_contains_key() {
    local map_name="$1"
    local key="$2"
    map_exists "$map_name" || return 1
    local count
    count=$(./duckdb mapdata.db -csv "SELECT COUNT(*) FROM maps WHERE map_name='$map_name' AND key='$key';" | tail -n +2)
    if [ "$count" -gt 0 ]; then
        return 0
    else
        return 1
    fi
}

# Function to remove a key-value pair from the map
map_remove() {
    local map_name="$1"
    local key="$2"
    map_exists "$map_name" || return 1
    ./duckdb mapdata.db -csv "DELETE FROM maps WHERE map_name='$map_name' AND key='$key';"
}

# Function to print all key-value pairs in the map
map_print() {
    local map_name="$1"
    map_exists "$map_name" || return 1
    ./duckdb mapdata.db -csv "SELECT key, value FROM maps WHERE map_name='$map_name';" | tail -n +2 | while IFS=, read -r key value; do
        echo "$key: $value"
    done
}

# Function to sort the map by values
map_sort_by_value() {
    local map_name="$1"
    map_exists "$map_name" || return 1
    ./duckdb mapdata.db -csv "SELECT key, value FROM maps WHERE map_name='$map_name' ORDER BY CAST(value AS INTEGER);" | tail -n +2 | while IFS=, read -r key value; do
        echo "$key: $value"
    done
}

map_cascade_subtract() {
    local map_name="$1"
    map_exists "$map_name" || return 1

    local keys
    local values
    keys=($(./duckdb mapdata.db -csv "SELECT key FROM maps WHERE map_name='$map_name' ORDER BY key;" | tail -n +2))
    values=($(./duckdb mapdata.db -csv "SELECT value FROM maps WHERE map_name='$map_name' ORDER BY key;" | tail -n +2))

    echo "Initial keys: ${keys[@]}" >&2
    echo "Initial values: ${values[@]}" >&2

    local num_values=${#values[@]}
    for ((i = 1; i < num_values; i++)); do
        local previous_value="${values[i - 1]}"
        values[i]=$((${values[i]} - ${previous_value}))
    done

    echo "Updated values after subtraction: ${values[@]}" >&2

    for i in "${!keys[@]}"; do
        ./duckdb mapdata.db -csv "UPDATE maps SET value='${values[i]}' WHERE map_name='$map_name' AND key='${keys[i]}';"
    done
}

# Main script logic
download_duckdb
initialize_db
