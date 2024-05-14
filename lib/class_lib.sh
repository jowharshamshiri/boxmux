#!/usr/bin/env bash

source "/Users/bahram/ws/prj/machinegenesis/crossbash/lib/map_lib.sh"
source "/Users/bahram/ws/prj/machinegenesis/crossbash/lib/list_lib.sh"

CLASSES_LIST_NAME="classes"
CLASSES_LIST_ID=""
CLASSES_MAP_ID=""

class_init() {
    local class_name="$1"
    local class_id=$(./duckdb "$DUCKDB_FILE_NAME" -csv "INSERT INTO classes (id, class_name) VALUES (nextval('seq_class_id'), '$class_name') ON CONFLICT DO NOTHING RETURNING id;" | tail -n +2)
    echo "$class_id"
}

setup_classes() {
    CLASSES_LIST_ID=$(list_init "$CLASSES_LIST_NAME")

}

class_init() {
    local class_name="$1"
    local properties_list_id instances_list_id class_id;

    list_add "$CLASSES_LIST_ID" "$class_name"

    properties_list_id=$(list_init "${class_name}_properties")
    instances_list_id=$(list_init "${class_name}_instances")

    class_id=$(list_init "class_${class_name}")
    map_add_or_set "$class_id" "properties_list_id" "$properties_list_id"
    map_add_or_set "$class_id" "instances_map_id" "$instances_map_id"

    echo "$class_id"
}

class_exists() {
    local class_id="$1"
    map_exists "$class_id"
}

# Add a property to a class.
class_add_property() {
    local class_id="$1"
    local property="$2"
    local properties_list_id

    properties_list_id=$(map_get "$class_id" "properties_list_id")
    list_add "$properties_list_id" "$property"
}


random_string() {
    local length="$1:16"
    local random_string
    random_string=$(cat /dev/urandom | tr -dc 'a-zA-Z0-9' | fold -w "$length" | head -n 1)
    echo "$random_string"
}

# Create an instance of a class.
class_create_instance() {
    local class_id="$1"

    local instance_name=$(random_string)

    local instances_map_id instance_id

    instances_map_id=$(map_get "$class_id" "instances_map_id")
    instance_id=$(map_init "${instance_name}_properties")
    map_add_or_set "$instances_map_id" "$instance_name" "$instance_id"

    echo "$instance_id"
}

# Set a property for a specific instance.
instance_set_property() {
    local class_id="$1"
    local instance_name="$2"
    local property="$3"
    local value="$4"
    local instances_map_id instance_id

    instances_map_id=$(map_get "$class_id" "instances_map_id")
    instance_id=$(map_get "$instances_map_id" "$instance_name")
    map_add_or_set "$instance_id" "$property" "$value"
}

# Get a property value for a specific instance.
instance_get_property() {
    local class_id="$1"
    local instance_name="$2"
    local property="$3"
    local instances_map_id instance_id

    instances_map_id=$(map_get "$class_id" "instances_map_id")
    instance_id=$(map_get "$instances_map_id" "$instance_name")
    map_get "$instance_id" "$property"
}

# List all instances of a class.
class_list_instances() {
    local class_id="$1"
    local instances_map_id

    instances_map_id=$(map_get "$class_id" "instances_map_id")
    map_keys "$instances_map_id"
}

# Get instances of a class by property value.
class_get_by_property() {
    local class_id="$1"
    local property="$2"
    local value="$3"
    local instances_map_id instance_name instance_id instance_value

    instances_map_id=$(map_get "$class_id" "instances_map_id")
    local keys=($(map_keys "$instances_map_id"))
    for instance_name in "${keys[@]}"; do
        instance_id=$(map_get "$instances_map_id" "$instance_name")
        instance_value=$(map_get "$instance_id" "$property")
        if [[ "$instance_value" == "$value" ]]; then
            echo "$instance_name"
        fi
    done
}

# Sort instances of a class by a property.
class_sort_by_property() {
    local class_id="$1"
    local property="$2"
    local instances_map_id instances instance_name instance_id

    class_exists "$class_id" || return 1

    instances_map_id=$(map_get "$class_id" "instances_map_id")
    properties_list_id=$(map_get "$class_id" "properties_list_id")

    # Sort the map by values
    map_sort_by_value "$map_id"

    # Fetch sorted keys and values
    local keys
    local values
    keys=($(map_keys "$map_id"))
    values=($(./duckdb "$DUCKDB_FILE_NAME" -csv "SELECT value FROM maps_data WHERE map_id=$map_id ORDER BY idx;" | tail -n +2))

    echo "Initial keys: ${keys[@]}" >&2
    echo "Initial values: ${values[@]}" >&2

    local num_values=${#values[@]}
    for ((i = 1; i < num_values; i++)); do
        for ((j = i; j < num_values; j++)); do
            values[j]=$((values[j] - values[i - 1]))
        done
    done

    echo "Updated values after subtraction: ${values[@]}" >&2

    for i in "${!keys[@]}"; do
        ./duckdb "$DUCKDB_FILE_NAME" -csv "UPDATE maps_data SET value='${values[i]}' WHERE map_id=$map_id AND key='${keys[i]}';"
    done
}

}

# Perform cascade subtraction on a property of all instances in a class.
class_cascade_subtract_property() {
    local class_id="$1"
    local property="$2"
    local instances_map_id instances instance_name instance_id values
}
