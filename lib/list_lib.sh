#!/usr/bin/env bash

source "/Users/bahram/ws/prj/machinegenesis/crossbash/lib/duckdb_lib.sh"

list_init() {
    local list_name="$1"
    local list_id=$(./duckdb "$DUCKDB_FILE_NAME" -csv "INSERT INTO lists (id, list_name) VALUES (nextval('seq_list_id'), '$list_name') ON CONFLICT DO NOTHING RETURNING id;" | tail -n +2)
    echo "$list_id"
}

# Function to check if a list exists
list_exists() {
    local list_id="$1"
    local count
    count=$(./duckdb "$DUCKDB_FILE_NAME" -csv "SELECT COUNT(*) FROM lists WHERE id=$list_id;" | tail -n +2)
    if [ "$count" -eq 0 ]; then
        echo "List with id '$list_id' does not exist. Please initialize it first." >&2
        return 1
    else
        return 0
    fi
}

# Function to clear all entries from a list
list_clear() {
    local list_id="$1"
    list_exists "$list_id" || return 1
    ./duckdb "$DUCKDB_FILE_NAME" -csv "DELETE FROM lists_data WHERE list_id=$list_id;"
}

# Function to add a value to the list
list_add() {
    local list_id="$1"
    local value="$2"
    list_exists "$list_id" || return 1
    local idx
    idx=$(./duckdb "$DUCKDB_FILE_NAME" -csv "SELECT COALESCE(MAX(idx), 0) + 1 FROM lists_data WHERE list_id=$list_id;" | tail -n +2)
    ./duckdb "$DUCKDB_FILE_NAME" -csv "INSERT INTO lists_data (list_id, value, idx) VALUES ($list_id, '$value', $idx);"
}

# Function to get a value at a specific index in the list
list_get() {
    local list_id="$1"
    local idx="$2"
    list_exists "$list_id" || return 1
    local value
    value=$(./duckdb "$DUCKDB_FILE_NAME" -csv "SELECT value FROM lists_data WHERE list_id=$list_id AND idx=$idx;" | tail -n +2)
    if [ -n "$value" ]; then
        echo "$value"
    else
        return 1
    fi
}

# Function to remove a value at a specific index from the list
list_remove() {
    local list_id="$1"
    local idx="$2"
    list_exists "$list_id" || return 1
    ./duckdb "$DUCKDB_FILE_NAME" -csv "DELETE FROM lists_data WHERE list_id=$list_id AND idx=$idx;"
}

# Function to print all values in the list
list_print() {
    local list_id="$1"
    list_exists "$list_id" || return 1
    ./duckdb "$DUCKDB_FILE_NAME" -csv "SELECT idx, value FROM lists_data WHERE list_id=$list_id ORDER BY idx;" | tail -n +2 | while IFS=, read -r idx value; do
        echo "$idx: $value"
    done
}

# Function to sort the list by values
list_sort() {
    local list_id="$1"
    list_exists "$list_id" || return 1

    ./duckdb "$DUCKDB_FILE_NAME" -csv "
        CREATE TEMP TABLE list_temp_sorted AS
        SELECT list_id, value, ROW_NUMBER() OVER (ORDER BY CAST(value AS INTEGER)) AS new_idx
        FROM lists_data
        WHERE list_id = $list_id;
    "

    ./duckdb "$DUCKDB_FILE_NAME" -csv "
        DELETE FROM lists_data WHERE list_id = $list_id;
    "

    ./duckdb "$DUCKDB_FILE_NAME" -csv "
        INSERT INTO lists_data (list_id, value, idx)
        SELECT list_id, value, new_idx FROM list_temp_sorted;
    "

    ./duckdb "$DUCKDB_FILE_NAME" -csv "DROP TABLE list_temp_sorted;"
}

# Function to perform cascade subtraction on the list values
list_cascade_subtract() {
    local list_id="$1"
    list_exists "$list_id" || return 1

    # Sort the list by values
    list_sort "$list_id"

    # Fetch sorted values
    local values
    values=($(./duckdb "$DUCKDB_FILE_NAME" -csv "SELECT value FROM lists_data WHERE list_id=$list_id ORDER BY idx;" | tail -n +2))

    echo "Initial values: ${values[@]}" >&2

    local num_values=${#values[@]}
    for ((i = 1; i < num_values; i++)); do
        for ((j = i; j < num_values; j++)); do
            values[j]=$((values[j] - values[i - 1]))
        done
    done

    echo "Updated values after subtraction: ${values[@]}" >&2

    ./duckdb "$DUCKDB_FILE_NAME" -csv "DELETE FROM lists_data WHERE list_id=$list_id;"

    for i in "${!values[@]}"; do
        local idx=$((i + 1))
        local value="${values[i]}"
        ./duckdb "$DUCKDB_FILE_NAME" -csv "INSERT INTO lists_data (list_id, value, idx) VALUES ($list_id, '$value', $idx);"
    done
}

list_values() {
    local list_id="$1"
    list_exists "$list_id" || return 1
    ./duckdb "$DUCKDB_FILE_NAME" -csv "SELECT value FROM lists_data WHERE list_id=$list_id ORDER BY idx;" | tail -n +2
}
