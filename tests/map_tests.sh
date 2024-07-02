#!/usr/bin/env bash

DUCKDB_FILE_NAME="boxmux.duckdb"
source ../lib/map_lib.sh # Source the script with your functions

# Function to reset the database
reset() {
    rm -f "$DUCKDB_FILE_NAME"
    initialize_db
}

test_map_new() {
    reset
    local map_name="test_map"
    local data_type_id=$DATATYPE_ID_INTEGER

    map_new "$map_name" "$data_type_id" || {
        echo "map_new test failed"
        return 1
    }

    local result=$(run_duckdb_csv_query "SELECT COUNT(*) AS count FROM maps WHERE map_name = '$map_name' AND data_type_id = $data_type_id;")
    result=$(echo "$result" | tail -n 1) # Get the actual count result
    if [[ "$result" -eq 1 ]]; then
        echo "map_new test passed" >&2
        return 0
    else
        echo "map_new test failed" >&2
        return 1
    fi
}

test_map_exists() {
    reset
    local map_name="test_map"
    local data_type_id=$DATATYPE_ID_INTEGER
    map_new "$map_name" "$data_type_id"

    local map_id=$(run_duckdb_csv_query "SELECT id FROM maps WHERE map_name = '$map_name';" | tail -n 1)

    if [[ $(map_exists "$map_id") -eq 0 ]]; then
        echo "map_exists test passed" >&2
        return 0
    else
        echo "map_exists test failed" >&2
        return 1
    fi
}

test_map_add_or_set() {
    reset
    local map_name="test_map"
    local data_type_id=$DATATYPE_ID_TEXT

    local map_id=$(map_new "$map_name" "$data_type_id")
    echo "Debug: map_id is '$map_id'" >&2 # Debug output

    if [[ -z "$map_id" ]]; then
        echo "Failed to retrieve map_id" >&2
        return 1
    fi

    map_add_or_set "$map_id" "key1" "value1"

    local result=$(run_duckdb_csv_query "SELECT value FROM maps_data WHERE map_id = $map_id AND key = 'key1';" | tail -n 1)
    if [[ "$result" == "value1" ]]; then
        echo "map_add_or_set test passed" >&2
        return 0
    else
        echo "map_add_or_set test failed" >&2
        return 1
    fi
}

test_map_clear() {
    reset
    local map_name="test_map"
    local data_type_id=$DATATYPE_ID_TEXT

    local map_id=$(map_new "$map_name" "$data_type_id")
    map_add_or_set "$map_id" "key1" "value1"
    map_clear "$map_id"

    local result=$(run_duckdb_csv_query "SELECT COUNT(*) AS count FROM maps_data WHERE map_id = $map_id;" | tail -n 1)
    if [[ "$result" -eq 0 ]]; then
        echo "map_clear test passed" >&2
        return 0
    else
        echo "map_clear test failed" >&2
        return 1
    fi
}

# Test map_get
test_map_get() {
    reset
    local map_name="test_map"
    local data_type_id=$DATATYPE_ID_TEXT

    local map_id=$(map_new "$map_name" "$data_type_id")

    map_add_or_set "$map_id" "key1" "value1"

    local value
    value=$(map_get "$map_id" "key1")

    echo "Debug: value is '$value'" >&2 # Debug output
    if [[ "$value" == "value1" ]]; then
        echo "map_get test passed" >&2
        return 0
    else
        echo "map_get test failed" >&2
        return 1
    fi
}

# Test map_contains_key
test_map_contains_key() {
    reset
    local map_name="test_map"
    local data_type_id=$DATATYPE_ID_TEXT

    local map_id=$(map_new "$map_name" "$data_type_id")
    map_add_or_set "$map_id" "key1" "value1"

    if [[ $(map_contains_key "$map_id" "key1") -eq 0 ]]; then
        echo "map_contains_key test passed" >&2
        return 0
    else
        echo "map_contains_key test failed" >&2
        return 1
    fi
}

# Test map_remove function
test_map_remove() {
    reset
    local map_name="test_map"
    local data_type_id=$DATATYPE_ID_TEXT

    local map_id=$(map_new "$map_name" "$data_type_id")
    map_add_or_set "$map_id" "key1" "value1"
    map_remove "$map_id" "key1"

    local result=$(run_duckdb_csv_query "SELECT COUNT(*) AS count FROM maps_data WHERE map_id = $map_id AND key = 'key1';" | tail -n 1)
    if [[ "$result" -eq 0 ]]; then
        echo "map_remove test passed" >&2
        return 0
    else
        echo "map_remove test failed" >&2
        return 1
    fi
}

# Test map_print function
test_map_print() {
    reset
    local map_name="test_map"
    local data_type_id=$DATATYPE_ID_TEXT

    local map_id=$(map_new "$map_name" "$data_type_id")
    map_add_or_set "$map_id" "key1" "value1"
    map_add_or_set "$map_id" "key2" "value2"

    map_print "$map_id" || {
        echo "map_print test failed" >&2
        return 1
    }
    echo "map_print test passed" >&2
    return 0
}

# Test map_sort_by_value function
test_map_sort_by_value() {
    reset
    local map_name="test_map"
    local data_type_id=$DATATYPE_ID_INTEGER

    local map_id=$(map_new "$map_name" "$data_type_id")
    map_add_or_set "$map_id" "key1" "3"
    map_add_or_set "$map_id" "key2" "1"
    map_add_or_set "$map_id" "key3" "2"

    map_sort_by_value "$map_id" || {
        echo "map_sort_by_value test failed" >&2
        return 1
    }

    local keys=($(run_duckdb_csv_query "SELECT key FROM maps_data WHERE map_id = $map_id ORDER BY idx;" | tail -n +2))
    if [[ "${keys[0]}" == "key2" && "${keys[1]}" == "key3" && "${keys[2]}" == "key1" ]]; then
        echo "map_sort_by_value test passed" >&2
        return 0
    else
        echo "map_sort_by_value test failed" >&2
        return 1
    fi
}

test_map_keys() {
    reset
    local map_name="test_map"
    local data_type_id=$DATATYPE_ID_TEXT

    local map_id=$(map_new "$map_name" "$data_type_id")
    map_add_or_set "$map_id" "key1" "value1"
    map_add_or_set "$map_id" "key2" "value2"

    local keys=$(map_keys "$map_id")
    if [[ "$keys" == "key1,key2" ]]; then
        echo "map_keys test passed" >&2
        return 0
    else
        echo "map_keys test failed" >&2
        return 1
    fi
}

test_map_cascade_subtract() {
    reset
    local map_name="test_map"
    local data_type_id=$DATATYPE_ID_INTEGER

    local map_id=$(map_new "$map_name" "$data_type_id")
    map_add_or_set "$map_id" "key1" "3"
    map_add_or_set "$map_id" "key2" "1"
    map_add_or_set "$map_id" "key3" "2"

    map_cascade_subtract "$map_id" || {
        echo "map_cascade_subtract test failed" >&2
        return 1
    }

    local values=($(run_duckdb_csv_query "SELECT value FROM maps_data WHERE map_id = $map_id ORDER BY idx;" | tail -n +2))
    if [[ "${values[0]}" == "1" && "${values[1]}" == "1" && "${values[2]}" == "1" ]]; then
        echo "map_cascade_subtract test passed" >&2
        return 0
    else
        echo "map_cascade_subtract test failed" >&2
        return 1
    fi
}

run_test() {
    local test_name="$1"
    echo "Running test: $test_name" >&2
    $test_name
    result=$?
    if [ $result -eq 0 ]; then
        echo "PASSED: $test_name" >&2
    else
        echo "FAILED: $test_name" >&2
        exit 1
    fi
}

# Run all tests
run_test test_map_new
run_test test_map_exists
run_test test_map_add_or_set
run_test test_map_get
run_test test_map_clear
run_test test_map_contains_key
run_test test_map_remove
run_test test_map_print
run_test test_map_sort_by_value
run_test test_map_keys
run_test test_map_cascade_subtract
