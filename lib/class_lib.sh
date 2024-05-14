#!/usr/bin/env bash

source "/Users/bahram/ws/prj/machinegenesis/crossbash/lib/duckdb_lib.sh"

class_new() {
    local class_name="$1"

    if [ -z "$class_name" ]; then
        echo "Usage: class_new <class_name>" >&2
        return 1
    fi

    if [[ $(validate_text "$class_name") -ne 0 ]]; then
        echo "Invalid class name" >&2
        return 1
    fi

    class_id=$(./duckdb "$DUCKDB_FILE_NAME" -csv "INSERT INTO classes (class_name) VALUES ('$class_name') RETURNING id;" | tail -n +2)

    if [ -z "$class_id" ]; then
        echo "Failed to create class" >&2
        return 1
    fi

    echo "$class_id"
}

class_exists() {
    local class_id="$1"

    if [ -z "$class_id" ]; then
        echo "Usage: class_exists <class_id>" >&2
        return 1
    fi

    if [[ $(validate_integer "$class_id") -ne 0 ]]; then
        echo "Invalid class ID" >&2
        return 1
    fi

    local result=$(./duckdb "$DUCKDB_FILE_NAME" -csv "SELECT COUNT(*) AS count FROM classes WHERE id = $class_id;")
    result=$(echo "$result" | tail -n 1) # Get the actual count result
    if [[ "$result" -eq 1 ]]; then
        return 0
    else
        return 1
    fi
}

class_add_property() {
    local class_id="$1"
    local property="$2"
    local data_type_id="$3"

    if [ -z "$class_id" ] || [ -z "$property" ] || [ -z "$data_type_id" ]; then
        echo "Usage: class_add_property <class_id> <property> <data_type_id>" >&2
        return 1
    fi

    if [[ $(validate_integer "$class_id") -ne 0 || $(validate_text "$property") -ne 0 || $(validate_integer "$data_type_id") -ne 0 ]]; then
        echo "Invalid class ID, property, or data type ID" >&2
        return 1
    fi

    class_exists "$class_id" || {
        echo "Class does not exist" >&2
        return 1
    }

    property_id=$(./duckdb "$DUCKDB_FILE_NAME" -csv "INSERT INTO classes_properties (class_id, property, data_type_id) VALUES ($class_id, '$property', $data_type_id) RETURNING id;" | tail -n +2)

    if [ -z "$property_id" ]; then
        echo "Failed to add property" >&2
        return 1
    fi

    echo "$property_id"
}

class_create_instance() {
    local class_id="$1"

    if [ -z "$class_id" ]; then
        echo "Usage: class_create_instance <class_id>" >&2
        return 1
    fi

    if [[ $(validate_integer "$class_id") -ne 0 ]]; then
        echo "Invalid class ID" >&2
        return 1
    fi

    class_exists "$class_id" || {
        echo "Class does not exist" >&2
        return 1
    }

    local max_idx=$(./duckdb "$DUCKDB_FILE_NAME" -csv "SELECT COALESCE(MAX(idx), -1) + 1 FROM classes_instances WHERE class_id = $class_id;" | tail -n 1)
    instance_id=$(./duckdb "$DUCKDB_FILE_NAME" -csv "INSERT INTO classes_instances (class_id, idx) VALUES ($class_id, $max_idx) RETURNING instance_id;" | tail -n +2)

    if [ -z "$instance_id" ]; then
        echo "Failed to create instance" >&2
        return 1
    fi

    echo "$instance_id"
}

instance_set_property() {
    local class_id="$1"
    local instance_id="$2"
    local property_id="$3"
    local value="$4"

    if [ -z "$class_id" ] || [ -z "$instance_id" ] || [ -z "$property_id" ] || [ -z "$value" ]; then
        echo "Usage: instance_set_property <class_id> <instance_id> <property_id> <value>" >&2
        return 1
    fi

    if [[ $(validate_integer "$class_id") -ne 0 || $(validate_integer "$instance_id") -ne 0 || $(validate_integer "$property_id") -ne 0 || $(validate_text "$value") -ne 0 ]]; then
        echo "Invalid class ID, instance ID, property ID, or value" >&2
        return 1
    fi

    class_exists "$class_id" || {
        echo "Class does not exist" >&2
        return 1
    }

    ./duckdb "$DUCKDB_FILE_NAME" -csv "INSERT INTO classes_instances_data (instance_id, property_id, value) VALUES ($instance_id, $property_id, '$value') ON CONFLICT (instance_id, property_id) DO UPDATE SET value = EXCLUDED.value;" || {
        echo "Failed to set property" >&2
        return 1
    }
}

instance_get_property() {
    local class_id="$1"
    local instance_id="$2"
    local property_id="$3"

    if [ -z "$class_id" ] || [ -z "$instance_id" ] || [ -z "$property_id" ]; then
        echo "Usage: instance_get_property <class_id> <instance_id> <property_id>" >&2
        return 1
    fi

    if [[ $(validate_integer "$class_id") -ne 0 || $(validate_integer "$instance_id") -ne 0 || $(validate_integer "$property_id") -ne 0 ]]; then
        echo "Invalid class ID, instance ID, or property ID" >&2
        return 1
    fi

    class_exists "$class_id" || return 1

    local value=$(./duckdb "$DUCKDB_FILE_NAME" -csv "SELECT value FROM classes_instances_data WHERE instance_id = $instance_id AND property_id = $property_id;" | tail -n 1)

    if [[ -z "$value" ]]; then
        echo "Property not found" >&2
        return 1
    fi

    echo "$value"
}

class_list_instances() {
    local class_id="$1"

    if [ -z "$class_id" ]; then
        echo "Usage: class_list_instances <class_id>" >&2
        return 1
    fi

    if [[ $(validate_integer "$class_id") -ne 0 ]]; then
        echo "Invalid class ID" >&2
        return 1
    fi

    class_exists "$class_id" || return 1

    local instances=$(./duckdb "$DUCKDB_FILE_NAME" -csv "SELECT instance_id FROM classes_instances WHERE class_id = $class_id ORDER BY idx;")
    echo "$instances" | tail -n +2
}

class_get_by_property() {
    local class_id="$1"
    local property_id="$2"
    local value="$3"

    if [ -z "$class_id" ] || [ -z "$property_id" ] || [ -z "$value" ]; then
        echo "Usage: class_get_by_property <class_id> <property_id> <value>" >&2
        return 1
    fi

    if [[ $(validate_integer "$class_id") -ne 0 || $(validate_integer "$property_id") -ne 0 || $(validate_text "$value") -ne 0 ]]; then
        echo "Invalid class ID, property ID, or value" >&2
        return 1
    fi

    class_exists "$class_id" || return 1

    local instances=$(./duckdb "$DUCKDB_FILE_NAME" -csv "SELECT instance_id FROM classes_instances_data WHERE property_id = $property_id AND value = '$value' ORDER BY instance_id;")
    echo "$instances" | tail -n +2
}

class_sort_by_property() {
    local class_id="$1"
    local property_id="$2"

    if [ -z "$class_id" ] || [ -z "$property_id" ]; then
        echo "Usage: class_sort_by_property <class_id> <property_id>" >&2
        return 1
    fi

    if [[ $(validate_integer "$class_id") -ne 0 || $(validate_integer "$property_id") -ne 0 ]]; then
        echo "Invalid class ID or property ID" >&2
        return 1
    fi

    class_exists "$class_id" || return 1

    ./duckdb "$DUCKDB_FILE_NAME" -csv "CREATE TABLE class_temp_sorted AS SELECT instance_id, value, ROW_NUMBER() OVER (ORDER BY value) - 1 AS new_idx FROM classes_instances_data WHERE property_id = $property_id;"
    ./duckdb "$DUCKDB_FILE_NAME" -csv "UPDATE classes_instances SET idx = (SELECT new_idx FROM class_temp_sorted WHERE classes_instances.instance_id = class_temp_sorted.instance_id) WHERE class_id = $class_id;"
    ./duckdb "$DUCKDB_FILE_NAME" -csv "DROP TABLE class_temp_sorted;" || {
        echo "Failed to sort instances" >&2
        return 1
    }
}

class_cascade_subtract_property() {
    local class_id="$1"
    local property_id="$2"

    if [ -z "$class_id" ] || [ -z "$property_id" ]; then
        echo "Usage: class_cascade_subtract_property <class_id> <property_id>" >&2
        return 1
    fi

    if [[ $(validate_integer "$class_id") -ne 0 || $(validate_integer "$property_id") -ne 0 ]]; then
        echo "Invalid class ID or property ID" >&2
        return 1
    fi

    class_exists "$class_id" || return 1

    class_sort_by_property "$class_id" "$property_id"

    local instances_and_values=$(./duckdb "$DUCKDB_FILE_NAME" -csv "
        SELECT i.instance_id, d.value
        FROM classes_instances i
        JOIN classes_instances_data d ON i.instance_id = d.instance_id
        WHERE i.class_id = $class_id AND d.property_id = $property_id
        ORDER BY i.idx;
    ")

    local instances=()
    local values=()
    local header_skipped=false
    while IFS=, read -r instance value; do
        if ! $header_skipped; then
            header_skipped=true
            continue
        fi
        instances+=("$instance")
        values+=("$value")
    done <<<"$instances_and_values"

    echo "Initial instances: ${instances[*]}" >&2
    echo "Initial values: ${values[*]}" >&2

    local num_values=${#values[@]}
    for ((i = num_values - 1; i > 0; i--)); do
        values[$i]=$((values[i] - values[i - 1]))
    done

    echo "Updated values after subtraction: ${values[*]}" >&2

    for ((i = 0; i < num_values; i++)); do
        ./duckdb "$DUCKDB_FILE_NAME" -csv "UPDATE classes_instances_data SET value = '${values[$i]}' WHERE property_id = $property_id AND instance_id = '${instances[$i]}';" || {
            echo "Failed to update value for instance ${instances[$i]}" >&2
            return 1
        }
    done
}
