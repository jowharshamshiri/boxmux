#!/usr/bin/env bash

source ../lib/list_lib.sh # Adjust the path to where your list functions are located

# Function to reset the database
reset() {
    rm -f "$DUCKDB_FILE_NAME"
    initialize_db
}

# Test list initialization and retrieval of list_id
test_list_init() {
    reset
    echo "Testing list initialization..." >&2
    local list_name="test_list"
    local list_id
    list_id=$(list_init "$list_name")
    if [ -n "$list_id" ]; then
        echo "List initialized with id: $list_id" >&2
    else
        echo "List initialization failed" >&2
        exit 1
    fi
}

# Test adding and retrieving a value in the list
test_list_add_get() {
    reset
    echo "Testing add and get functions..." >&2
    local list_name="test_list"
    local list_id
    list_id=$(list_init "$list_name")
    list_add "$list_id" "value1"
    local value
    value=$(list_get "$list_id" 1)
    if [ "$value" == "value1" ]; then
        echo "Add and get functions work correctly" >&2
    else
        echo "Add and get functions failed" >&2
        exit 1
    fi
}

# Test list clear function
test_list_clear() {
    reset
    echo "Testing list clear function..." >&2
    local list_name="test_list"
    local list_id
    list_id=$(list_init "$list_name")
    list_add "$list_id" "value1"
    list_clear "$list_id"
    local value
    value=$(list_get "$list_id" 1)
    if [ -z "$value" ]; then
        echo "List clear function works correctly" >&2
    else
        echo "List clear function failed" >&2
        exit 1
    fi
}

# Test list remove function
test_list_remove() {
    reset
    echo "Testing list remove function..." >&2
    local list_name="test_list"
    local list_id
    list_id=$(list_init "$list_name")
    list_add "$list_id" "value1"
    list_remove "$list_id" 1
    local value
    value=$(list_get "$list_id" 1)
    if [ -z "$value" ]; then
        echo "List remove function works correctly" >&2
    else
        echo "List remove function failed" >&2
        exit 1
    fi
}

# Test list print function
test_list_print() {
    reset
    echo "Testing list print function..." >&2
    local list_name="test_list"
    local list_id
    list_id=$(list_init "$list_name")
    list_add "$list_id" "value1"
    list_add "$list_id" "value2"
    list_print "$list_id"
}

# Test list sort function
test_list_sort() {
    reset
    echo "Testing list sort function..." >&2
    local list_name="test_list"
    local list_id
    list_id=$(list_init "$list_name")
    list_add "$list_id" "20"
    list_add "$list_id" "10"
    list_sort "$list_id"
    list_print "$list_id"
}

# Test list cascade subtract function
test_list_cascade_subtract() {
    reset
    echo "Testing list cascade subtract function..." >&2
    local list_name="test_list"
    local list_id
    list_id=$(list_init "$list_name")
    list_add "$list_id" "15"
    list_add "$list_id" "10"
    list_add "$list_id" "25"
    list_cascade_subtract "$list_id"
    list_print "$list_id"
}

test_list_values() {
    reset
    echo "Testing list values function..." >&2
    local list_name="test_list"
    local list_id
    list_id=$(list_init "$list_name")
    list_add "$list_id" "value1"
    list_add "$list_id" "value2"
    local values
    values=($(list_values "$list_id"))
    if [[ "${values[@]}" == "value1 value2" ]]; then
        echo "List values function works correctly" >&2
    else
        echo "List values function failed" >&2
        exit 1
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
run_test test_list_init
run_test test_list_add_get
run_test test_list_clear
run_test test_list_remove
run_test test_list_print
run_test test_list_sort
run_test test_list_cascade_subtract
run_test test_list_values
