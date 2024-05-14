#!/usr/bin/env bash

source ../lib/map_lib.sh   # Adjust the path to where your map functions are located
source ../lib/list_lib.sh  # Adjust the path to where your list functions are located
source ../lib/class_lib.sh # Adjust the path to where your class functions are located

# Function to reset the database
reset() {
    rm -f "$DUCKDB_FILE_NAME"
    initialize_db
}

# Test class initialization and property/instance setup
test_class_init() {
    reset
    echo "Testing class initialization..." >&2
    local class_name="TestClass"
    local class_id
    class_id=$(class_init "$class_name")
    if [ -n "$class_id" ]; then
        echo "Class initialized with id: $class_id" >&2
    else
        echo "Class initialization failed" >&2
        exit 1
    fi
}

# Test adding a property to a class
test_class_add_property() {
    reset
    echo "Testing class add property..." >&2
    local class_name="TestClass"
    local class_id
    local property="TestProperty"
    class_id=$(class_init "$class_name")
    class_add_property "$class_id" "$property"
    local properties_list_id
    properties_list_id=$(map_get "$class_id" "properties_list_id")
    if list_get "$properties_list_id" 1 | grep -q "$property"; then
        echo "Class add property works correctly" >&2
    else
        echo "Class add property failed" >&2
        exit 1
    fi
}

# Test creating an instance of a class
test_class_create_instance() {
    reset
    echo "Testing class create instance..." >&2
    local class_name="TestClass"
    local class_id
    local instance_name="TestInstance"
    class_id=$(class_init "$class_name")
    instance_id=$(class_create_instance "$class_id" "$instance_name")
    local instances_map_id
    instances_map_id=$(map_get "$class_id" "instances_map_id")
    if map_contains_key "$instances_map_id" "$instance_name"; then
        echo "Class create instance works correctly" >&2
    else
        echo "Class create instance failed" >&2
        exit 1
    fi
}

# Test setting and getting a property for a specific instance
test_instance_set_get_property() {
    reset
    echo "Testing instance set/get property..." >&2
    local class_name="TestClass"
    local class_id
    local instance_name="TestInstance"
    local property="TestProperty"
    local value="TestValue"
    class_id=$(class_init "$class_name")
    instance_id=$(class_create_instance "$class_id" "$instance_name")
    instance_set_property "$class_id" "$instance_name" "$property" "$value"
    local fetched_value
    fetched_value=$(instance_get_property "$class_id" "$instance_name" "$property")
    if [ "$fetched_value" == "$value" ]; then
        echo "Instance set/get property works correctly" >&2
    else
        echo "Instance set/get property failed" >&2
        exit 1
    fi
}

# Test listing all instances of a class
test_class_list_instances() {
    reset
    echo "Testing class list instances..." >&2
    local class_name="TestClass"
    local class_id
    local instance_name1="TestInstance1"
    local instance_name2="TestInstance2"
    class_id=$(class_init "$class_name")
    class_create_instance "$class_id" "$instance_name1"
    class_create_instance "$class_id" "$instance_name2"
    local instances
    instances=$(class_list_instances "$class_id")
    if echo "$instances" | grep -q "$instance_name1" && echo "$instances" | grep -q "$instance_name2"; then
        echo "Class list instances works correctly" >&2
    else
        echo "Class list instances failed" >&2
        exit 1
    fi
}

# Test getting instances of a class by property value
test_class_get_by_property() {
    reset
    echo "Testing class get by property..." >&2
    local class_name="TestClass"
    local class_id
    local instance_name="TestInstance"
    local property="TestProperty"
    local value="TestValue"
    class_id=$(class_init "$class_name")
    instance_id=$(class_create_instance "$class_id" "$instance_name")
    instance_set_property "$class_id" "$instance_name" "$property" "$value"
    local fetched_instance
    fetched_instance=$(class_get_by_property "$class_id" "$property" "$value")
    if [ "$fetched_instance" == "$instance_name" ]; then
        echo "Class get by property works correctly" >&2
    else
        echo "Class get by property failed" >&2
        exit 1
    fi
}

# Test sorting instances of a class by a property
test_class_sort_by_property() {
    reset
    echo "Testing class sort by property..." >&2
    local class_name="TestClass"
    local class_id
    local instance_name1="TestInstance1"
    local instance_name2="TestInstance2"
    local property="TestProperty"
    class_id=$(class_init "$class_name")
    instance_id1=$(class_create_instance "$class_id" "$instance_name1")
    instance_id2=$(class_create_instance "$class_id" "$instance_name2")
    instance_set_property "$class_id" "$instance_name1" "$property" 20
    instance_set_property "$class_id" "$instance_name2" "$property" 10
    local sorted_instances
    sorted_instances=$(class_sort_by_property "$class_id" "$property")
    if [[ "${sorted_instances[0]}" == "$instance_name2" ]] && [[ "${sorted_instances[1]}" == "$instance_name1" ]]; then
        echo "Class sort by property works correctly" >&2
    else
        echo "Class sort by property failed" >&2
        exit 1
    fi
}

# Test cascade subtraction on a property of all instances in a class
test_class_cascade_subtract_property() {
    reset
    echo "Testing class cascade subtract property..." >&2
    local class_name="TestClass"
    local class_id
    local instance_name1="TestInstance1"
    local instance_name2="TestInstance2"
    local instance_name3="TestInstance3"
    local property="TestProperty"
    class_id=$(class_init "$class_name")
    instance_id1=$(class_create_instance "$class_id" "$instance_name1")
    instance_id2=$(class_create_instance "$class_id" "$instance_name2")
    instance_id3=$(class_create_instance "$class_id" "$instance_name3")
    instance_set_property "$class_id" "$instance_name1" "$property" 15
    instance_set_property "$class_id" "$instance_name2" "$property" 10
    instance_set_property "$class_id" "$instance_name3" "$property" 25
    class_cascade_subtract_property "$class_id" "$property"
    local value1 value2 value3
    value1=$(instance_get_property "$class_id" "$instance_name1" "$property")
    value2=$(instance_get_property "$class_id" "$instance_name2" "$property")
    value3=$(instance_get_property "$class_id" "$instance_name3" "$property")
    if [ "$value1" -eq 15 ] && [ "$value2" -eq -5 ] && [ "$value3" -eq 15 ]; then
        echo "Class cascade subtract property works correctly" >&2
    else
        echo "Class cascade subtract property failed" >&2
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
run_test test_class_init
run_test test_class_add_property
run_test test_class_create_instance
run_test test_instance_set_get_property
run_test test_class_list_instances
run_test test_class_get_by_property
run_test test_class_sort_by_property
run_test test_class_cascade_subtract_property
