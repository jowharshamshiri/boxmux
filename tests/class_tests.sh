#!/usr/bin/env bash

#temp file
DUCKDB_FILE_NAME=$(mktemp)
echo "Using temp file: $DUCKDB_FILE_NAME" >&2
source "../lib/class_lib.sh"

initialize_test_db() {
    run_duck_db_csv_script "$XB_HOME/assets/db_no_fks.sql"
    setup_data_types
}

# Function to reset the database
reset() {
    rm -f "$DUCKDB_FILE_NAME"
    initialize_test_db
}

test_class_new() {
    reset
    local class_name="test_class"

    class_new "$class_name" || {
        echo "class_new test failed"
        return 1
    }

    local result=$(run_duckdb_csv_query "SELECT COUNT(*) AS count FROM classes WHERE class_name = '$class_name';")
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

    local class_id=$(run_duckdb_csv_query "SELECT id FROM classes WHERE class_name = '$class_name';" | tail -n 1)

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

    local result=$(run_duckdb_csv_query "SELECT COUNT(*) AS count FROM classes_properties WHERE class_id = $class_id AND property = '$property';" | tail -n 1)
    if [[ "$result" -eq 1 ]]; then
        echo "class_add_property test passed" >&2
        return 0
    else
        echo "class_add_property test failed" >&2
        return 1
    fi
}
test_instance_new() {
    reset
    local class_name="test_class"

    local class_id=$(class_new "$class_name")

    if [[ -z "$class_id" ]]; then
        echo "Failed to retrieve class_id" >&2
        return 1
    fi

    instance_new "$class_id"

    local result=$(run_duckdb_csv_query "SELECT COUNT(*) AS count FROM classes_instances WHERE class_id = $class_id;" | tail -n 1)
    if [[ "$result" -eq 1 ]]; then
        echo "instance_new test passed" >&2
        return 0
    else
        echo "instance_new test failed" >&2
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
    local instance_id=$(instance_new "$class_id")

    instance_set_property "$class_id" "$instance_id" "$property_id" "test_value"

    local result=$(run_duckdb_csv_query "SELECT value FROM classes_instances_data WHERE instance_id = $instance_id AND property_id = $property_id ORDER BY instance_id;" | tail -n 1)
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
    local instance_id=$(instance_new "$class_id")

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

test_instance_get_by_property() {
    reset
    local class_name="test_class"
    local property="test_property"
    local data_type_id=$DATATYPE_ID_TEXT

    local class_id=$(class_new "$class_name")
    local property_id=$(class_add_property "$class_id" "$property" "$data_type_id")
    local instance_id_1=$(instance_new "$class_id")
    local instance_id_2=$(instance_new "$class_id")

    instance_set_property "$class_id" "$instance_id_1" "$property_id" "value1"
    instance_set_property "$class_id" "$instance_id_2" "$property_id" "value2"

    local instances=$(instance_get_by_property "$class_id" "$property_id" "value1")

    if [[ "$instances" == "$instance_id_1" ]]; then
        echo "instance_get_by_property test passed" >&2
        return 0
    else
        echo "instance_get_by_property test failed" >&2
        return 1
    fi
}

test_instance_list() {
    reset
    local class_name="test_class"

    local class_id=$(class_new "$class_name")
    local instance_id_1=$(instance_new "$class_id")
    local instance_id_2=$(instance_new "$class_id")

    local instances=$(instance_list "$class_id")

    if [[ "$instances" == "$instance_id_1"$'\n'"$instance_id_2" ]]; then
        echo "instance_list test passed" >&2
        return 0
    else
        echo "instance_list test failed" >&2
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
    local instance_id_1=$(instance_new "$class_id")
    local instance_id_2=$(instance_new "$class_id")
    local instance_id_3=$(instance_new "$class_id")

    instance_set_property "$class_id" "$instance_id_1" "$property_id" "3"
    instance_set_property "$class_id" "$instance_id_2" "$property_id" "1"
    instance_set_property "$class_id" "$instance_id_3" "$property_id" "2"

    class_sort_by_property "$class_id" "$property_id" || {
        echo "class_sort_by_property test failed" >&2
        return 1
    }

    local instances=($(run_duckdb_csv_query "SELECT instance_id FROM classes_instances WHERE class_id = $class_id ORDER BY idx;" | tail -n +2))
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
    local instance_id_1=$(instance_new "$class_id")
    local instance_id_2=$(instance_new "$class_id")
    local instance_id_3=$(instance_new "$class_id")

    instance_set_property "$class_id" "$instance_id_1" "$property_id" "3"
    instance_set_property "$class_id" "$instance_id_2" "$property_id" "1"
    instance_set_property "$class_id" "$instance_id_3" "$property_id" "2"

    class_cascade_subtract_property "$class_id" "$property_id" || {
        echo "class_cascade_subtract_property test failed" >&2
        return 1
    }

    local values=($(run_duckdb_csv_query "SELECT value FROM classes_instances_data WHERE property_id = $property_id ORDER BY instance_id;" | tail -n +2))

    echo "Values: ${values[*]}" >&2

    if [[ "${values[0]}" == "1" && "${values[1]}" == "1" && "${values[2]}" == "1" ]]; then
        echo "class_cascade_subtract_property test passed" >&2
        return 0
    else
        echo "class_cascade_subtract_property test failed" >&2
        return 1
    fi
}

test_cache_get_instance() {
    reset
    local class_name="test_class"
    local property_1="property_1"
    local property_2="property_2"
    local data_type_id_1=$DATATYPE_ID_TEXT
    local data_type_id_2=$DATATYPE_ID_INTEGER

    local class_id=$(class_new "$class_name")
    local property_id_1=$(class_add_property "$class_id" "$property_1" "$data_type_id_1")
    local property_id_2=$(class_add_property "$class_id" "$property_2" "$data_type_id_2")
    local instance_id=$(instance_new "$class_id")

    instance_set_property "$class_id" "$instance_id" "$property_id_1" "test_value_1"
    instance_set_property "$class_id" "$instance_id" "$property_id_2" "123"

    # Fetch properties from the database (should not use cache)
    local properties
    properties=$(cache_get_instance "$class_id" "$instance_id" "true")
    echo "First fetch (forced): $properties"
    if [[ "$properties" != *"$property_id_1,test_value_1"* ]] || [[ "$properties" != *"$property_id_2,123"* ]]; then
        echo "cache_get_instance test (force fetch) failed" >&2
        return 1
    fi

    # Fetch properties again (should use cache)
    properties=$(cache_get_instance "$class_id" "$instance_id" "false")
    echo "Second fetch (cached): $properties"
    if [[ "$properties" != *"$property_id_1,test_value_1"* ]] || [[ "$properties" != *"$property_id_2,123"* ]]; then
        echo "cache_get_instance test (cache fetch) failed" >&2
        return 1
    fi

    # Change a property value and force fetch (should reflect new value)
    instance_set_property "$class_id" "$instance_id" "$property_id_1" "new_test_value_1"
    properties=$(cache_get_instance "$class_id" "$instance_id" "true")
    echo "Third fetch (forced after update): $properties"
    if [[ "$properties" != *"$property_id_1,new_test_value_1"* ]] || [[ "$properties" != *"$property_id_2,123"* ]]; then
        echo "cache_get_instance test (force fetch after update) failed" >&2
        return 1
    fi

    # Fetch properties again (should use cache and reflect new value due to forced update)
    properties=$(cache_get_instance "$class_id" "$instance_id" "false")
    echo "Fourth fetch (cached after update): $properties"
    if [[ "$properties" != *"$property_id_1,new_test_value_1"* ]] || [[ "$properties" != *"$property_id_2,123"* ]]; then
        echo "cache_get_instance test (cache fetch after update) failed" >&2
        return 1
    fi

    echo "cache_get_instance test passed" >&2
    return 0
}

test_cache_get_property() {
    reset
    local class_name="test_class"
    local property="test_property"
    local data_type_id=$DATATYPE_ID_TEXT
    local property_value="test_value"

    local class_id=$(class_new "$class_name")
    local property_id=$(class_add_property "$class_id" "$property" "$data_type_id")
    local instance_id=$(instance_new "$class_id")

    instance_set_property "$class_id" "$instance_id" "$property_id" "$property_value"

    local value
    value=$(cache_get_property "$class_id" "$instance_id" "$property_id")

    if [[ "$value" == "$property_value" ]]; then
        echo "cache_get_property test passed" >&2
        return 0
    else
        echo "cache_get_property test failed" >&2
        return 1
    fi
}

test_class_delete() {
    reset
    local class_name="test_class"
    local property="test_property"
    local data_type_id=$DATATYPE_ID_TEXT

    local class_id=$(class_new "$class_name")
    local property_id=$(class_add_property "$class_id" "$property" "$data_type_id")
    local instance_id=$(instance_new "$class_id")

    instance_set_property "$class_id" "$instance_id" "$property_id" "test_value"

    class_delete "$class_id" || {
        echo "class_delete test failed: class_delete returned error" >&2
        return 1
    }

    local result=$(run_duckdb_csv_query "SELECT COUNT(*) AS count FROM classes WHERE id = $class_id;")
    result=$(echo "$result" | tail -n 1)
    if [[ "$result" -ne 0 ]]; then
        echo "class_delete test failed: class was not deleted" >&2
        return 1
    fi

    result=$(run_duckdb_csv_query "SELECT COUNT(*) AS count FROM classes_properties WHERE class_id = $class_id;")
    result=$(echo "$result" | tail -n 1)
    if [[ "$result" -ne 0 ]]; then
        echo "class_delete test failed: properties were not deleted" >&2
        return 1
    fi

    result=$(run_duckdb_csv_query "SELECT COUNT(*) AS count FROM classes_instances WHERE class_id = $class_id;")
    result=$(echo "$result" | tail -n 1)
    if [[ "$result" -ne 0 ]]; then
        echo "class_delete test failed: instances were not deleted" >&2
        return 1
    fi

    result=$(run_duckdb_csv_query "SELECT COUNT(*) AS count FROM classes_instances_data WHERE instance_id = $instance_id;")
    result=$(echo "$result" | tail -n 1)
    if [[ "$result" -ne 0 ]]; then
        echo "class_delete test failed: instance properties were not deleted" >&2
        return 1
    fi

    echo "class_delete test passed" >&2
    return 0
}

test_instance_exists() {
    reset
    local class_name="test_class"

    local class_id=$(class_new "$class_name")
    local instance_id=$(instance_new "$class_id")

    if [[ $(instance_exists "$class_id" "$instance_id") -eq 0 ]]; then
        echo "instance_exists test passed" >&2
        return 0
    else
        echo "instance_exists test failed" >&2
        return 1
    fi
}

test_instance_delete() {
    reset
    local class_name="test_class"
    local property="test_property"
    local data_type_id=$DATATYPE_ID_TEXT

    local class_id=$(class_new "$class_name")
    local property_id=$(class_add_property "$class_id" "$property" "$data_type_id")
    local instance_id=$(instance_new "$class_id")

    instance_set_property "$class_id" "$instance_id" "$property_id" "test_value"

    instance_delete "$class_id" "$instance_id" || {
        echo "instance_delete test failed: instance_delete returned error" >&2
        return 1
    }

    local result=$(run_duckdb_csv_query "SELECT COUNT(*) AS count FROM classes_instances WHERE instance_id = $instance_id;")
    result=$(echo "$result" | tail -n 1)
    if [[ "$result" -ne 0 ]]; then
        echo "instance_delete test failed: instance was not deleted" >&2
        return 1
    fi

    result=$(run_duckdb_csv_query "SELECT COUNT(*) AS count FROM classes_instances_data WHERE instance_id = $instance_id;")
    result=$(echo "$result" | tail -n 1)
    if [[ "$result" -ne 0 ]]; then
        echo "instance_delete test failed: instance properties were not deleted" >&2
        return 1
    fi

    echo "instance_delete test passed" >&2
    return 0
}

test_instance_get_by_properties() {
    reset
    local class_name="test_class"
    local property_1="property_1"
    local property_2="property_2"
    local data_type_id_1=$DATATYPE_ID_TEXT
    local data_type_id_2=$DATATYPE_ID_TEXT

    local class_id=$(class_new "$class_name")
    local property_id_1=$(class_add_property "$class_id" "$property_1" "$data_type_id_1")
    local property_id_2=$(class_add_property "$class_id" "$property_2" "$data_type_id_2")
    local instance_id_1=$(instance_new "$class_id")
    local instance_id_2=$(instance_new "$class_id")

    instance_set_property "$class_id" "$instance_id_1" "$property_id_1" "value1"
    instance_set_property "$class_id" "$instance_id_1" "$property_id_2" "value2"
    instance_set_property "$class_id" "$instance_id_2" "$property_id_1" "value3"
    instance_set_property "$class_id" "$instance_id_2" "$property_id_2" "value4"

    local result=$(instance_get_by_properties "$class_id" "$property_id_1" "value1" "$property_id_2" "value2")
    if [[ "$result" == "$instance_id_1" ]]; then
        echo "instance_get_by_properties test passed" >&2
        return 0
    else
        echo "instance_get_by_properties test failed" >&2
        return 1
    fi
}

test_instance_list_by_property() {
    reset
    local class_name="test_class"
    local property="test_property"
    local data_type_id=$DATATYPE_ID_TEXT

    local class_id=$(class_new "$class_name")
    local property_id=$(class_add_property "$class_id" "$property" "$data_type_id")
    local instance_id_1=$(instance_new "$class_id")
    local instance_id_2=$(instance_new "$class_id")

    instance_set_property "$class_id" "$instance_id_1" "$property_id" "value1"
    instance_set_property "$class_id" "$instance_id_2" "$property_id" "value1"

    local instances=$(instance_list_by_property "$class_id" "$property_id" "value1")

    if [[ "$instances" == "$instance_id_1"$'\n'"$instance_id_2" ]]; then
        echo "instance_list_by_property test passed" >&2
        return 0
    else
        echo "instance_list_by_property test failed" >&2
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
    local instance_id_1=$(instance_new "$class_id")
    local instance_id_2=$(instance_new "$class_id")
    local instance_id_3=$(instance_new "$class_id")

    instance_set_property "$class_id" "$instance_id_1" "$property_id" "3"
    instance_set_property "$class_id" "$instance_id_2" "$property_id" "1"
    instance_set_property "$class_id" "$instance_id_3" "$property_id" "2"

    class_sort_by_property "$class_id" "$property_id" || {
        echo "class_sort_by_property test failed" >&2
        return 1
    }

    local instances=($(run_duckdb_csv_query "SELECT instance_id FROM classes_instances WHERE class_id = $class_id ORDER BY idx;" | tail -n +2))
    if [[ "${instances[0]}" == "$instance_id_2" && "${instances[1]}" == "$instance_id_3" && "${instances[2]}" == "$instance_id_1" ]]; then
        echo "class_sort_by_property test passed" >&2
        return 0
    else
        echo "class_sort_by_property test failed" >&2
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
run_test test_instance_new
run_test test_instance_set_property
run_test test_instance_get_property
run_test test_instance_list
run_test test_instance_get_by_property
run_test test_class_sort_by_property
run_test test_class_cascade_subtract_property
run_test test_cache_get_instance
run_test test_cache_get_property
run_test test_class_delete
run_test test_instance_exists
run_test test_instance_delete
run_test test_instance_get_by_properties
run_test test_instance_list_by_property
run_test test_class_sort_by_property
echo "All tests passed" >&2
