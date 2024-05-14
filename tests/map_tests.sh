#!/usr/bin/env bash

# Load the functions
source ../map.sh # Replace with the actual path to your script

# Function to run a test and print the result
run_test() {
    local test_name="$1"
    local result="$2"
    if [ "$result" -eq 0 ]; then
        echo "$test_name: PASSED"
    else
        echo "$test_name: FAILED"
    fi
}

# Test initialization
test_map_init() {
    local map_name="testmap"
    map_init "$map_name"
    local count
    count=$(./duckdb mapdata.db -csv "SELECT COUNT(*) FROM map_names WHERE map_name='$map_name';" | tail -n +2)
    if [ "$count" -eq 1 ]; then
        return 0
    else
        return 1
    fi
}

# Test add or set
test_map_add_or_set() {
    local map_name="testmap"
    map_init "$map_name"
    map_clear "$map_name"
    map_add_or_set "$map_name" "key1" "value1"
    local value
    value=$(map_get "$map_name" "key1")
    if [ "$value" == "value1" ]; then
        return 0
    else
        return 1
    fi
}

# Test get
test_map_get() {
    local map_name="testmap"
    map_init "$map_name"
    map_clear "$map_name"
    map_add_or_set "$map_name" "key1" "value1"
    local value
    value=$(map_get "$map_name" "key1")
    if [ "$value" == "value1" ]; then
        return 0
    else
        return 1
    fi
}

# Test contains key
test_map_contains_key() {
    local map_name="testmap"
    map_init "$map_name"
    map_clear "$map_name"
    map_add_or_set "$map_name" "key1" "value1"
    map_contains_key "$map_name" "key1"
    local contains=$?
    if [ $contains -eq 0 ]; then
        return 0
    else
        return 1
    fi
}

# Test remove
test_map_remove() {
    local map_name="testmap"
    map_init "$map_name"
    map_clear "$map_name"
    map_add_or_set "$map_name" "key1" "value1"
    map_remove "$map_name" "key1"
    map_contains_key "$map_name" "key1"
    local contains=$?
    if [ $contains -eq 1 ]; then
        return 0
    else
        return 1
    fi
}

# Test print
test_map_print() {
    local map_name="testmap"
    map_init "$map_name"
    map_clear "$map_name"
    map_add_or_set "$map_name" "key1" "value1"
    map_add_or_set "$map_name" "key2" "value2"
    local output
    output=$(map_print "$map_name")
    if [[ "$output" == *"key1: value1"* ]] && [[ "$output" == *"key2: value2"* ]]; then
        return 0
    else
        return 1
    fi
}

# Test sort by value
test_map_sort_by_value() {
    local map_name="testmap"
    map_init "$map_name"
    map_clear "$map_name"
    map_add_or_set "$map_name" "key2" "2"
    map_add_or_set "$map_name" "key1" "1"
    local output
    output=$(map_sort_by_value "$map_name")
    echo "Output of map_sort_by_value: $output" >&2
    if [[ "$output" == "key1: 1"* ]] && [[ "$output" == *"key2: 2" ]]; then
        return 0
    else
        return 1
    fi
}

# Test cascade subtract
test_map_cascade_subtract() {
    local map_name="testmap"
    map_init "$map_name"
    map_clear "$map_name"
    map_add_or_set "$map_name" "key1" "10"
    map_add_or_set "$map_name" "key2" "30"
    map_add_or_set "$map_name" "key3" "60"
    map_cascade_subtract "$map_name"
    local value1
    local value2
    local value3
    value1=$(map_get "$map_name" "key1")
    value2=$(map_get "$map_name" "key2")
    value3=$(map_get "$map_name" "key3")
    if [ "$value1" == "10" ] && [ "$value2" == "20" ] && [ "$value3" == "30" ]; then
        return 0
    else
        return 1
    fi
}

# Run all tests
run_test "test_map_init" "$(
    test_map_init
    echo $?
)"
run_test "test_map_add_or_set" "$(
    test_map_add_or_set
    echo $?
)"
run_test "test_map_get" "$(
    test_map_get
    echo $?
)"
run_test "test_map_contains_key" "$(
    test_map_contains_key
    echo $?
)"
run_test "test_map_remove" "$(
    test_map_remove
    echo $?
)"
run_test "test_map_print" "$(
    test_map_print
    echo $?
)"
run_test "test_map_sort_by_value" "$(
    test_map_sort_by_value
    echo $?
)"
run_test "test_map_cascade_subtract" "$(
    test_map_cascade_subtract
    echo $?
)"
