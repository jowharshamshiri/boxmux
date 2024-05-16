#!/usr/bin/env bash

source "/Users/bahram/ws/prj/machinegenesis/crossbash/map.sh"
source "./Users/bahram/ws/prj/machinegenesis/crossbash/class.sh"

test_class_functions() {
    echo "Running tests..."

    local class_name="TestClass"
    local instance_name="TestInstance"
    local property_name="TestProperty"
    local property_value="TestValue"

    # Test class_init
    class_init "$class_name"
    if ! map_contains_key "${class_name}_properties" "$property_name"; then
        echo "class_init passed"
    else
        echo "class_init failed"
    fi

    # Test class_add_property
    class_add_property "$class_name" "$property_name"
    if map_contains_key "${class_name}_properties" "$property_name"; then
        echo "class_add_property passed"
    else
        echo "class_add_property failed"
    fi

    # Test instance_new
    instance_new "$class_name" "$instance_name"
    if map_contains_key "${class_name}_instances" "$instance_name"; then
        echo "instance_new passed"
    else
        echo "instance_new failed"
    fi

    # Test instance_set_property
    instance_set_property "$instance_name" "$property_name" "$property_value"
    if [[ "$(instance_get_property "$instance_name" "$property_name")" == "$property_value" ]]; then
        echo "instance_set_property passed"
    else
        echo "instance_set_property failed"
    fi

    # Test instance_get_property
    if [[ "$(instance_get_property "$instance_name" "$property_name")" == "$property_value" ]]; then
        echo "instance_get_property passed"
    else
        echo "instance_get_property failed"
    fi

    # Test instance_get_by_property
    if [[ "$(instance_get_by_property "$class_name" "$property_name" "$property_value")" == "$instance_name" ]]; then
        echo "instance_get_by_property passed"
    else
        echo "instance_get_by_property failed"
    fi

    # Test instance_list
    instance_new "$class_name" "Instance1"
    instance_new "$class_name" "Instance2"
    if [[ "$(instance_list "$class_name")" == *"$instance_name Instance1 Instance2"* ]]; then
        echo "instance_list passed"
    else
        echo "instance_list failed"
    fi

    # Test instance_get_by_property
    if [[ "$(instance_get_by_property "$class_name" "$property_name" "$property_value")" == "$instance_name" ]]; then
        echo "instance_get_by_property passed"
    else
        echo "instance_get_by_property failed"
    fi

    # Test class_sort_by_property
    instance_set_property "Instance1" "$property_name" 2
    instance_set_property "Instance2" "$property_name" 1
    echo "Expected order: Instance2, Instance1, $instance_name"
    class_sort_by_property "$class_name" "$property_name"

    # Test class_cascade_subtract_property
    instance_set_property "$instance_name" "$property_name" 3
    instance_set_property "Instance1" "$property_name" 7
    instance_set_property "Instance2" "$property_name" 5
    echo "Expected values after cascading subtract: Instance1=4, Instance2=2, $instance_name=3"
    class_cascade_subtract_property "$class_name" "$property_name"

    echo "Tests completed."
}

# Run the tests
test_class_functions
