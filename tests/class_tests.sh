#!/usr/bin/env bash

DUCKDB_FILE_NAME="crossbash.duckdb"
source "../lib/class_lib.sh" # Source the script with your functions

# Function to reset the database
reset() {
    rm -f "$DUCKDB_FILE_NAME"
    initialize_db
}

test_class_new() {
    reset
    local class_name="test_class"

    class_new "$class_name" || {
        echo "class_new test failed"
        return 1
    }

    local result=$(./duckdb "$DUCKDB_FILE_NAME" -csv "SELECT COUNT(*) AS count FROM classes WHERE class_name = '$class_name';")
    result=$(echo "$result" | tail -n 1) # Get the actual count result
    if [[ "$result" -eq 1 ]]; then
        echo "class_new test passed" >&2
        return 0
    else
        echo "class_new test failed" >&2
        return 1
    fi
}

test_class_exists() {
    reset
    local class_name="test_class"
    class_new "$class_name"

    local class_id=$(./duckdb "$DUCKDB_FILE_NAME" -csv "SELECT id FROM classes WHERE class_name = '$class_name';" | tail -n 1)

    if [[ $(class_exists "$class_id") -eq 0 ]]; then
        echo "class_exists test passed" >&2
        return 0
    else
        echo "class_exists test failed" >&2
        return 1
    fi
}

test_class_add_property() {
    reset
    local class_name="test_class"
    local property="test_property"
    local data_type_id=$DATATYPE_ID_TEXT

    local class_id=$(class_new "$class_name")

    if [[ -z "$class_id" ]]; then
        echo "Failed to retrieve class_id" >&2
        return 1
    fi

    class_add_property "$class_id" "$property" "$data_type_id"

    local result=$(./duckdb "$DUCKDB_FILE_NAME" -csv "SELECT COUNT(*) AS count FROM classes_properties WHERE class_id = $class_id AND property = '$property';" | tail -n 1)
    if [[ "$result" -eq 1 ]]; then
        echo "class_add_property test passed" >&2
        return 0
    else
        echo "class_add_property test failed" >&2
        return 1
    fi
}
test_class_create_instance() {
    reset
    local class_name="test_class"

    local class_id=$(class_new "$class_name")

    if [[ -z "$class_id" ]]; then
        echo "Failed to retrieve class_id" >&2
        return 1
    fi

    class_create_instance "$class_id"

    local result=$(./duckdb "$DUCKDB_FILE_NAME" -csv "SELECT COUNT(*) AS count FROM classes_instances WHERE class_id = $class_id;" | tail -n 1)
    if [[ "$result" -eq 1 ]]; then
        echo "class_create_instance test passed" >&2
        return 0
    else
        echo "class_create_instance test failed" >&2
        return 1
    fi
}
test_instance_set_property() {
    reset
    local class_name="test_class"
    local property="test_property"
    local data_type_id=$DATATYPE_ID_TEXT

    local class_id=$(class_new "$class_name")
    local property_id=$(class_add_property "$class_id" "$property" "$data_type_id")
    local instance_id=$(class_create_instance "$class_id")

    instance_set_property "$class_id" "$instance_id" "$property_id" "test_value"

    local result=$(./duckdb "$DUCKDB_FILE_NAME" -csv "SELECT value FROM classes_instances_data WHERE instance_id = $instance_id AND property_id = $property_id ORDER BY instance_id;" | tail -n 1)
    if [[ "$result" == "test_value" ]]; then
        echo "instance_set_property test passed" >&2
        return 0
    else
        echo "instance_set_property test failed" >&2
        return 1
    fi
}
test_instance_get_property() {
    reset
    local class_name="test_class"
    local property="test_property"
    local data_type_id=$DATATYPE_ID_TEXT

    local class_id=$(class_new "$class_name")
    local property_id=$(class_add_property "$class_id" "$property" "$data_type_id")
    local instance_id=$(class_create_instance "$class_id")

    instance_set_property "$class_id" "$instance_id" "$property_id" "test_value"

    local value
    value=$(instance_get_property "$class_id" "$instance_id" "$property_id")

    if [[ "$value" == "test_value" ]]; then
        echo "instance_get_property test passed" >&2
        return 0
    else
        echo "instance_get_property test failed" >&2
        return 1
    fi
}

test_class_get_by_property() {
    reset
    local class_name="test_class"
    local property="test_property"
    local data_type_id=$DATATYPE_ID_TEXT

    local class_id=$(class_new "$class_name")
    local property_id=$(class_add_property "$class_id" "$property" "$data_type_id")
    local instance_id_1=$(class_create_instance "$class_id")
    local instance_id_2=$(class_create_instance "$class_id")

    instance_set_property "$class_id" "$instance_id_1" "$property_id" "value1"
    instance_set_property "$class_id" "$instance_id_2" "$property_id" "value2"

    local instances=$(class_get_by_property "$class_id" "$property_id" "value1")

    if [[ "$instances" == "$instance_id_1" ]]; then
        echo "class_get_by_property test passed" >&2
        return 0
    else
        echo "class_get_by_property test failed" >&2
        return 1
    fi
}

test_class_list_instances() {
    reset
    local class_name="test_class"

    local class_id=$(class_new "$class_name")
    local instance_id_1=$(class_create_instance "$class_id")
    local instance_id_2=$(class_create_instance "$class_id")

    local instances=$(class_list_instances "$class_id")

    if [[ "$instances" == "$instance_id_1"$'\n'"$instance_id_2" ]]; then
        echo "class_list_instances test passed" >&2
        return 0
    else
        echo "class_list_instances test failed" >&2
        return 1
    fi
}

test_class_sort_by_property() {
    reset
    local class_name="test_class"
    local property="test_property"
    local data_type_id=$DATATYPE_ID_INTEGER

    local class_id=$(class_new "$class_name")
    local property_id=$(class_add_property "$class_id" "$property" "$data_type_id")
    local instance_id_1=$(class_create_instance "$class_id")
    local instance_id_2=$(class_create_instance "$class_id")
    local instance_id_3=$(class_create_instance "$class_id")

    instance_set_property "$class_id" "$instance_id_1" "$property_id" "3"
    instance_set_property "$class_id" "$instance_id_2" "$property_id" "1"
    instance_set_property "$class_id" "$instance_id_3" "$property_id" "2"

    class_sort_by_property "$class_id" "$property_id" || {
        echo "class_sort_by_property test failed" >&2
        return 1
    }

    local instances=($(./duckdb "$DUCKDB_FILE_NAME" -csv "SELECT instance_id FROM classes_instances WHERE class_id = $class_id ORDER BY idx;" | tail -n +2))
    if [[ "${instances[0]}" == "$instance_id_2" && "${instances[1]}" == "$instance_id_3" && "${instances[2]}" == "$instance_id_1" ]]; then
        echo "class_sort_by_property test passed" >&2
        return 0
    else
        echo "class_sort_by_property test failed" >&2
        return 1
    fi
}
test_class_cascade_subtract_property() {
    reset
    local class_name="test_class"
    local property="test_property"
    local data_type_id=$DATATYPE_ID_INTEGER

    local class_id=$(class_new "$class_name")
    local property_id=$(class_add_property "$class_id" "$property" "$data_type_id")
    local instance_id_1=$(class_create_instance "$class_id")
    local instance_id_2=$(class_create_instance "$class_id")
    local instance_id_3=$(class_create_instance "$class_id")

    instance_set_property "$class_id" "$instance_id_1" "$property_id" "3"
    instance_set_property "$class_id" "$instance_id_2" "$property_id" "1"
    instance_set_property "$class_id" "$instance_id_3" "$property_id" "2"

    class_cascade_subtract_property "$class_id" "$property_id" || {
        echo "class_cascade_subtract_property test failed" >&2
        return 1
    }

    local values=($(./duckdb "$DUCKDB_FILE_NAME" -csv "SELECT value FROM classes_instances_data WHERE property_id = $property_id ORDER BY instance_id;" | tail -n +2))

    echo "Values: ${values[*]}" >&2

    if [[ "${values[0]}" == "1" && "${values[1]}" == "1" && "${values[2]}" == "1" ]]; then
        echo "class_cascade_subtract_property test passed" >&2
        return 0
    else
        echo "class_cascade_subtract_property test failed" >&2
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
run_test test_class_new
run_test test_class_exists
run_test test_class_add_property
run_test test_class_create_instance
run_test test_instance_set_property
run_test test_instance_get_property
run_test test_class_list_instances
run_test test_class_get_by_property
run_test test_class_sort_by_property
run_test test_class_cascade_subtract_property
