#!/usr/bin/env bash

# Load the functions
source ../class.sh # Adjust the path to your class.sh

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

# Test class initialization
test_class_init() {
    local class_name="Person"
    class_init "$class_name"
    map_exists "${class_name}_properties" && map_exists "${class_name}_instances"
}

# Test adding a property to a class
test_class_add_property() {
    local class_name="Person"
    class_init "$class_name"
    class_add_property "$class_name" "age"
    class_add_property "$class_name" "height"
    map_contains_key "${class_name}_properties" "age" && map_contains_key "${class_name}_properties" "height"
}

# Test creating an instance
test_class_create_instance() {
    local class_name="Person"
    class_init "$class_name"
    class_create_instance "$class_name" "John"
    class_create_instance "$class_name" "Jane"
    map_contains_key "${class_name}_instances" "John" && map_contains_key "${class_name}_instances" "Jane"
}

# Test setting and getting an instance property
test_instance_set_and_get_property() {
    local class_name="Person"
    class_init "$class_name"
    class_create_instance "$class_name" "John"
    instance_set_property "John" "age" "30"
    instance_set_property "John" "height" "180"
    local age
    age=$(instance_get_property "John" "age")
    local height
    height=$(instance_get_property "John" "height")
    [[ "$age" == "30" && "$height" == "180" ]]
}

# Test listing instances
test_class_list_instances() {
    local class_name="Person"
    class_init "$class_name"
    class_create_instance "$class_name" "John"
    class_create_instance "$class_name" "Jane"
    local output
    output=$(class_list_instances "$class_name")
    [[ "$output" == *"John: John"* && "$output" == *"Jane: Jane"* ]]
}

# Test finding instances by property
test_class_get_by_property() {
    local class_name="Person"
    class_init "$class_name"
    class_create_instance "$class_name" "John"
    instance_set_property "John" "age" "30"
    class_create_instance "$class_name" "Jane"
    instance_set_property "Jane" "age" "25"
    local output
    output=$(class_get_by_property "$class_name" "age" "30")
    [[ "$output" == "John" ]]
}

# Test sorting instances by property
test_class_sort_by_property() {
    local class_name="Person"
    class_init "$class_name"
    class_create_instance "$class_name" "John"
    instance_set_property "John" "age" "30"
    class_create_instance "$class_name" "Jane"
    instance_set_property "Jane" "age" "25"
    local output
    output=$(class_sort_by_property "$class_name" "age")
    [[ "$output" == *"Jane: 25"* && "$output" == *"John: 30"* ]]
}

# Test cascade subtract property
test_class_cascade_subtract_property() {
    local class_name="Person"
    class_init "$class_name"
    class_create_instance "$class_name" "John"
    instance_set_property "John" "height" "20"
    class_create_instance "$class_name" "Jane"
    instance_set_property "Jane" "height" "30"
    class_create_instance "$class_name" "Joe"
    instance_set_property "Joe" "height" "60"

    class_sort_by_property "$class_name" "height"

    echo "Before cascade subtract:" >&2
    class_list_instances "$class_name" >&2

    class_cascade_subtract_property "$class_name" "height"

    echo "After cascade subtract:" >&2
    class_list_instances "$class_name" >&2

    local john_height
    john_height=$(instance_get_property "John" "height")
    local jane_height
    jane_height=$(instance_get_property "Jane" "height")
    local joe_height
    joe_height=$(instance_get_property "Joe" "height")
    echo "John's height: $john_height, Jane's height: $jane_height, Joe's height: $joe_height" >&2

    [[ "$john_height" == "20" && "$jane_height" == "10" && "$joe_height" == "30" ]]
}

# Run all tests
run_test "test_class_init" "$(
    test_class_init
    echo $?
)"
run_test "test_class_add_property" "$(
    test_class_add_property
    echo $?
)"
run_test "test_class_create_instance" "$(
    test_class_create_instance
    echo $?
)"
run_test "test_instance_set_and_get_property" "$(
    test_instance_set_and_get_property
    echo $?
)"
run_test "test_class_list_instances" "$(
    test_class_list_instances
    echo $?
)"
run_test "test_class_get_by_property" "$(
    test_class_get_by_property
    echo $?
)"
# run_test "test_class_sort_by_property" "$(
#     test_class_sort_by_property
#     echo $?
# )"
# run_test "test_class_cascade_subtract_property" "$(
#     test_class_cascade_subtract_property
#     echo $?
# )"
