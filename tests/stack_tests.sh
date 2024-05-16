#!/usr/bin/env bash

DUCKDB_FILE_NAME="crossbash.duckdb"
source ../lib/stack_lib.sh # Adjust the path to your stack_lib.sh script

# Function to reset the database
reset() {
    rm -f "$DUCKDB_FILE_NAME"
    initialize_db
}

test_stack_new() {
    reset
    local stack_name="test_stack"
    local data_type_id=$DATATYPE_ID_INTEGER

    stack_new "$stack_name" "$data_type_id" || {
        echo "stack_new test failed"
        return 1
    }

    local result=$(run_duckdb_csv_query "SELECT COUNT(*) AS count FROM stacks WHERE stack_name = '$stack_name' AND data_type_id = $data_type_id;")
    result=$(echo "$result" | tail -n 1) # Get the actual count result
    if [[ "$result" -eq 1 ]]; then
        echo "stack_new test passed" >&2
        return 0
    else
        echo "stack_new test failed" >&2
        return 1
    fi
}

test_stack_push() {
    reset
    local stack_name="test_stack"
    local data_type_id=$DATATYPE_ID_TEXT

    local stack_id=$(stack_new "$stack_name" "$data_type_id")
    stack_push "$stack_id" "value1"

    local result=$(run_duckdb_csv_query "SELECT COUNT(*) AS count FROM stacks_data WHERE stack_id = $stack_id AND value = 'value1';" | tail -n 1)
    if [[ "$result" -eq 1 ]]; then
        echo "stack_push test passed" >&2
        return 0
    else
        echo "stack_push test failed" >&2
        return 1
    fi
}

test_stack_pop() {
    reset
    local stack_name="test_stack"
    local data_type_id=$DATATYPE_ID_TEXT

    local stack_id=$(stack_new "$stack_name" "$data_type_id")
    stack_push "$stack_id" "value1"
    stack_push "$stack_id" "value2"

    local value=$(stack_pop "$stack_id")
    if [[ "$value" == "value2" ]]; then
        echo "stack_pop test passed" >&2
        return 0
    else
        echo "stack_pop test failed" >&2
        return 1
    fi
}

test_stack_top() {
    reset
    local stack_name="test_stack"
    local data_type_id=$DATATYPE_ID_TEXT

    local stack_id=$(stack_new "$stack_name" "$data_type_id")
    stack_push "$stack_id" "value1"
    stack_push "$stack_id" "value2"

    local value=$(stack_top "$stack_id")
    if [[ "$value" == "value2" ]]; then
        echo "stack_top test passed" >&2
        return 0
    else
        echo "stack_top test failed" >&2
        return 1
    fi
}

test_stack_clear() {
    reset
    local stack_name="test_stack"
    local data_type_id=$DATATYPE_ID_TEXT

    local stack_id=$(stack_new "$stack_name" "$data_type_id")
    stack_push "$stack_id" "value1"
    stack_clear "$stack_id"

    local result=$(run_duckdb_csv_query "SELECT COUNT(*) AS count FROM stacks_data WHERE stack_id = $stack_id;" | tail -n 1)
    if [[ "$result" -eq 0 ]]; then
        echo "stack_clear test passed" >&2
        return 0
    else
        echo "stack_clear test failed" >&2
        return 1
    fi
}

test_stack_print() {
    reset
    local stack_name="test_stack"
    local data_type_id=$DATATYPE_ID_TEXT

    local stack_id=$(stack_new "$stack_name" "$data_type_id")
    stack_push "$stack_id" "value1"
    stack_push "$stack_id" "value2"

    stack_print "$stack_id" || {
        echo "stack_print test failed" >&2
        return 1
    }
    echo "stack_print test passed" >&2
    return 0
}

test_stack_size() {
    reset
    local stack_name="test_stack"
    local data_type_id=$DATATYPE_ID_TEXT

    local stack_id=$(stack_new "$stack_name" "$data_type_id")
    stack_push "$stack_id" "value1"
    stack_push "$stack_id" "value2"

    local size=$(stack_size "$stack_id")
    if [[ "$size" -eq 2 ]]; then
        echo "stack_size test passed" >&2
        return 0
    else
        echo "stack_size test failed" >&2
        return 1
    fi
}

test_stack_is_empty() {
    reset
    local stack_name="test_stack"
    local data_type_id=$DATATYPE_ID_TEXT

    local stack_id=$(stack_new "$stack_name" "$data_type_id")

    if stack_is_empty "$stack_id"; then
        echo "stack_is_empty test passed" >&2
        return 0
    else
        echo "stack_is_empty test failed" >&2
        return 1
    fi
}

test_stack_duplicate() {
    reset
    local stack_name="test_stack"
    local data_type_id=$DATATYPE_ID_TEXT

    local stack_id=$(stack_new "$stack_name" "$data_type_id")
    stack_push "$stack_id" "value1"
    stack_push "$stack_id" "value2"

    local new_stack_id=$(stack_duplicate "$stack_id")

    local result=$(run_duckdb_csv_query "SELECT COUNT(*) AS count FROM stacks_data WHERE stack_id = $new_stack_id;" | tail -n 1)
    if [[ "$result" -eq 2 ]]; then
        echo "stack_duplicate test passed" >&2
        return 0
    else
        echo "stack_duplicate test failed" >&2
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
run_test test_stack_new
run_test test_stack_push
run_test test_stack_pop
run_test test_stack_top
run_test test_stack_clear
run_test test_stack_print
run_test test_stack_size
run_test test_stack_is_empty
run_test test_stack_duplicate
