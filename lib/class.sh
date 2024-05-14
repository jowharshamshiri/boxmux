#!/usr/bin/env bash

source "/Users/bahram/ws/prj/machinegenesis/crossbash/map.sh"

class_init() {
    local class_name="$1"
    map_init "${class_name}_properties"
    map_init "${class_name}_instances"
}

class_add_property() {
    local class_name="$1"
    local property="$2"
    map_add_or_set "${class_name}_properties" "$property" "$property"
}

random_string() {
    echo "Generating random string" >&2
    echo "i$(cat /dev/urandom | tr -dc 'a-zA-Z0-9' | fold -w 15 | head -n 1)"
}

class_create_instance() {
    local class_name="$1"
    local instance_name="${2:-$(random_string)}"
    map_add_or_set "${class_name}_instances" "$instance_name" "$instance_name"
    map_init "${instance_name}_properties"
}

instance_set_property() {
    local instance_name="$1"
    local property="$2"
    local value="$3"
    map_add_or_set "${instance_name}_properties" "$property" "$value"
}

instance_get_property() {
    local instance_name="$1"
    local property="$2"
    map_get "${instance_name}_properties" "$property"
}

class_list_instances() {
    local class_name="$1"
    map_print "${class_name}_instances"
}

class_get_by_property() {
    local class_name="$1"
    local property="$2"
    local value="$3"
    local instances
    instances=$(map_print "${class_name}_instances" | cut -d':' -f1)
    for instance in $instances; do
        local prop_value
        prop_value=$(instance_get_property "$instance" "$property")
        if [ "$prop_value" == "$value" ]; then
            echo "$instance"
        fi
    done
}

class_sort_by_property() {
    local class_name="$1"
    local property="$2"
    local instances
    instances=$(map_print "${class_name}_instances" | cut -d':' -f1)
    local temp_map="sorted_${class_name}_${property}"
    map_init "$temp_map"
    for instance in $instances; do
        local prop_value
        prop_value=$(instance_get_property "$instance" "$property")
        map_add_or_set "$temp_map" "$instance" "$prop_value"
    done
    map_sort_by_value "$temp_map"
}

class_cascade_subtract_property() {
    local class_name="$1"
    local property="$2"
    map_exists "${class_name}_instances" || return 1

    local instances
    instances=$(map_print "${class_name}_instances" | cut -d':' -f1)
    local temp_map="cascade_${class_name}_${property}"
    map_init "$temp_map"

    for instance in $instances; do
        local prop_value
        prop_value=$(instance_get_property "$instance" "$property")
        map_add_or_set "$temp_map" "$instance" "$prop_value"
    done

    echo "Before cascade subtraction:" >&2
    map_print "$temp_map" >&2

    map_cascade_subtract "$temp_map"

    echo "After cascade subtraction:" >&2
    map_print "$temp_map" >&2

    local updated_instances
    updated_instances=$(map_print "$temp_map")

    while IFS= read -r line; do
        local instance="${line%%:*}"
        local new_value="${line##*: }"
        echo "Setting property for $instance to $new_value" >&2
        instance_set_property "$instance" "$property" "$new_value"
    done <<<"$updated_instances"
}
