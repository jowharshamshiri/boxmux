#!/usr/bin/env bash

class_new() {
    local class_name="$1"

    if [ -z "$class_name" ]; then
        log_fatal "Usage: class_new <class_name>"
        return 1
    fi

    if [ "$CLASS_CHECKS" == "true" ] && [[ $(validate_text "$class_name") -ne 0 ]]; then
        log_fatal "class_new: Invalid class name"
        return 1
    fi

    class_id=$("""$DUCKDB_EXECUTABLE""" "$DUCKDB_FILE_NAME" -csv "INSERT INTO classes (class_name) VALUES ('$class_name') RETURNING id;" | tail -n +2)

    if [ -z "$class_id" ]; then
        log_fatal "class_new: Failed to create class"
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

    if [ "$CLASS_CHECKS" == "true" ] && [[ $(validate_integer "$class_id") -ne 0 ]]; then
        echo "class_exists: Invalid class ID" >&2
        return 1
    fi

    local result=$("""$DUCKDB_EXECUTABLE""" "$DUCKDB_FILE_NAME" -csv "SELECT COUNT(*) AS count FROM classes WHERE id = $class_id;")
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
        echo "class_add_property: Invalid class ID, property, or data type ID. class_id: $class_id, property: $property, data_type_id: $data_type_id" >&2
        return 1
    fi

    [ "$CLASS_CHECKS" == "true" ] && { class_exists "$class_id" || {
        echo "class_add_property: Class does not exist: $class_id" >&2
        return 1
    }; }

    property_id=$("""$DUCKDB_EXECUTABLE""" "$DUCKDB_FILE_NAME" -csv "INSERT INTO classes_properties (class_id, property, data_type_id) VALUES ($class_id, '$property', $data_type_id) RETURNING id;" | tail -n +2)

    if [ -z "$property_id" ]; then
        echo "class_add_property: Failed to add property: $property" >&2
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

    if [ "$CLASS_CHECKS" == "true" ] && [[ $(validate_integer "$class_id") -ne 0 ]]; then
        echo "class_create_instance: Invalid class ID: $class_id" >&2
        return 1
    fi

    [ "$CLASS_CHECKS" == "true" ] && { class_exists "$class_id" || {
        echo "class_create_instance: Class does not exist: $class_id" >&2
        return 1
    }; }

    local max_idx=$("""$DUCKDB_EXECUTABLE""" "$DUCKDB_FILE_NAME" -csv "SELECT COALESCE(MAX(idx), -1) + 1 FROM classes_instances WHERE class_id = $class_id;" | tail -n 1)
    instance_id=$("""$DUCKDB_EXECUTABLE""" "$DUCKDB_FILE_NAME" -csv "INSERT INTO classes_instances (class_id, idx) VALUES ($class_id, $max_idx) RETURNING instance_id;" | tail -n +2)

    if [ -z "$instance_id" ]; then
        echo "class_create_instance: Failed to create instance: $class_id" >&2
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
        echo "Usage: instance_set_property <class_id> <instance_id> <property_id> <value>: $class_id, $instance_id, $property_id, $value" >&2
        return 1
    fi

    if [ "$CLASS_CHECKS" == "true" ] && { [[ $(validate_integer "$class_id") -ne 0 || $(validate_integer "$instance_id") -ne 0 || $(validate_integer "$property_id") -ne 0 || $(validate_text "$value") -ne 0 ]]; }; then
        echo "instance_set_property: Invalid class ID, instance ID, property ID, or value: class_id: $class_id, instance_id: $instance_id, property_id: $property_id, value: $value" >&2
        return 1
    fi

    [ "$CLASS_CHECKS" == "true" ] && { class_exists "$class_id" || {
        echo "instance_set_property: Class does not exist: $class_id" >&2
        return 1
    }; }

    """$DUCKDB_EXECUTABLE""" "$DUCKDB_FILE_NAME" -csv "INSERT INTO classes_instances_data (instance_id, property_id, value) VALUES ($instance_id, $property_id, '$value') ON CONFLICT (instance_id, property_id) DO UPDATE SET value = EXCLUDED.value;" || {
        echo "instance_set_property: Failed to set property: $property_id" >&2
        return 1
    }
}

instance_get_property() {
    local class_id="$1"
    local instance_id="$2"
    local property_id="$3"

    if [ -z "$class_id" ] || [ -z "$instance_id" ] || [ -z "$property_id" ]; then
        echo "Usage: instance_get_property <class_id> <instance_id> <property_id>: $class_id, $instance_id, $property_id" >&2
        return 1
    fi

    if [ "$CLASS_CHECKS" == "true" ] && { [[ $(validate_integer "$class_id") -ne 0 || $(validate_integer "$instance_id") -ne 0 || $(validate_integer "$property_id") -ne 0 ]]; }; then
        echo "instance_get_property: Invalid class ID, instance ID, or property ID: class_id: $class_id, instance_id: $instance_id, property_id: $property_id" >&2
        return 1
    fi

    [ "$CLASS_CHECKS" == "true" ] && { class_exists "$class_id" || return 1; }

    local value=$("""$DUCKDB_EXECUTABLE""" "$DUCKDB_FILE_NAME" -csv "SELECT value FROM classes_instances_data WHERE instance_id = $instance_id AND property_id = $property_id;" | tail -n 1)

    # if [[ -z "$value" ]]; then
    #     echo "instance_get_property: Property not found: $property_id for instance: $instance_id in class: $class_id" >&2
    #     return 1
    # fi

    echo "$value"
}

cache_get_instance_old() {
    local class_id="$1"
    local instance_id="$2"

    if [ -z "$class_id" ] || [ -z "$instance_id" ]; then
        echo "Usage: cache_get_instance <class_id> <instance_id>" >&2
        return 1
    fi

    if [ "$CLASS_CHECKS" == "true" ]; then
        if [[ $(validate_integer "$class_id") -ne 0 || $(validate_integer "$instance_id") -ne 0 ]]; then
            echo "cache_get_instance: Invalid class ID or instance ID: class_id: $class_id, instance_id: $instance_id" >&2
            return 1
        fi
        class_exists "$class_id" || return 1
    fi

    local query="SELECT p.property, d.value
                 FROM classes_properties p
                 JOIN classes_instances_data d ON p.id = d.property_id
                 WHERE d.instance_id = $instance_id;"

    local result=$("""$DUCKDB_EXECUTABLE""" "$DUCKDB_FILE_NAME" -csv "$query" | tail -n +2)

    if [ -z "$result" ]; then
        echo "cache_get_instance: No properties found for instance: $instance_id in class: $class_id" >&2
        return 1
    fi

    echo "$result"
}

class_list_instances() {
    local class_id="$1"

    if [ -z "$class_id" ]; then
        echo "Usage: class_list_instances <class_id>" >&2
        return 1
    fi

    if [[ $(validate_integer "$class_id") -ne 0 ]]; then
        echo "class_list_instances: Invalid class ID: $class_id" >&2
        return 1
    fi

    [ "$CLASS_CHECKS" == "true" ] && { class_exists "$class_id" || return 1; }

    local instances=$("""$DUCKDB_EXECUTABLE""" "$DUCKDB_FILE_NAME" -csv "SELECT instance_id FROM classes_instances WHERE class_id = $class_id ORDER BY idx;")
    echo "$instances" | tail -n +2
}

class_get_list_by_property() {
    local class_id="$1"
    local property_id="$2"
    local value="$3"

    if [ -z "$class_id" ] || [ -z "$property_id" ] || [ -z "$value" ]; then
        echo "Usage: class_get_by_property <class_id> <property_id> <value>" >&2
        return 1
    fi

    if [[ $(validate_integer "$class_id") -ne 0 || $(validate_integer "$property_id") -ne 0 || $(validate_text "$value") -ne 0 ]]; then
        echo "class_get_by_property: Invalid class ID, property ID, or value: class_id: $class_id, property_id: $property_id, value: $value" >&2
        return 1
    fi

    [ "$CLASS_CHECKS" == "true" ] && { class_exists "$class_id" || return 1; }

    local instances=$("""$DUCKDB_EXECUTABLE""" "$DUCKDB_FILE_NAME" -csv "SELECT instance_id FROM classes_instances_data WHERE property_id = $property_id AND value = '$value';")
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

    results=$(class_get_list_by_property "$class_id" "$property_id" "$value")

    if [ -z "$results" ]; then
        echo "class_get_by_property: No instances found for property: $property_id, value: $value" >&2
        return 1
    fi

    echo "$results" | head -n 1
}

class_sort_by_property() {
    local class_id="$1"
    local property_id="$2"

    if [ -z "$class_id" ] || [ -z "$property_id" ]; then
        echo "Usage: class_sort_by_property <class_id> <property_id>" >&2
        return 1
    fi

    if [ "$CLASS_CHECKS" == "true" ] && { [[ $(validate_integer "$class_id") -ne 0 || $(validate_integer "$property_id") -ne 0 ]]; }; then
        echo "class_sort_by_property: Invalid class ID or property ID: class_id: $class_id, property_id: $property_id" >&2
        return 1
    fi

    [ "$CLASS_CHECKS" == "true" ] && { class_exists "$class_id" || return 1; }

    """$DUCKDB_EXECUTABLE""" "$DUCKDB_FILE_NAME" -csv "CREATE TABLE class_temp_sorted AS SELECT instance_id, value, ROW_NUMBER() OVER (ORDER BY value) - 1 AS new_idx FROM classes_instances_data WHERE property_id = $property_id;"
    """$DUCKDB_EXECUTABLE""" "$DUCKDB_FILE_NAME" -csv "UPDATE classes_instances SET idx = (SELECT new_idx FROM class_temp_sorted WHERE classes_instances.instance_id = class_temp_sorted.instance_id) WHERE class_id = $class_id;"
    """$DUCKDB_EXECUTABLE""" "$DUCKDB_FILE_NAME" -csv "DROP TABLE class_temp_sorted;" || {
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

    if [ "$CLASS_CHECKS" == "true" ] && { [[ $(validate_integer "$class_id") -ne 0 || $(validate_integer "$property_id") -ne 0 ]]; }; then
        echo "class_cascade_subtract_property: Invalid class ID or property ID: class_id: $class_id, property_id: $property_id" >&2
        return 1
    fi

    [ "$CLASS_CHECKS" == "true" ] && { class_exists "$class_id" || return 1; }

    class_sort_by_property "$class_id" "$property_id"

    local instances_and_values=$("""$DUCKDB_EXECUTABLE""" "$DUCKDB_FILE_NAME" -csv "
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

    log_trace "Initial instances: ${instances[*]}"
    log_trace "Initial values: ${values[*]}"

    local num_values=${#values[@]}
    for ((i = num_values - 1; i > 0; i--)); do
        values[$i]=$((values[i] - values[i - 1]))
    done

    log_trace "Updated values after subtraction: ${values[*]}"

    for ((i = 0; i < num_values; i++)); do
        """$DUCKDB_EXECUTABLE""" "$DUCKDB_FILE_NAME" -csv "UPDATE classes_instances_data SET value = '${values[$i]}' WHERE property_id = $property_id AND instance_id = '${instances[$i]}';" || {
            log_trace "Failed to update value for instance ${instances[$i]}"
            return 1
        }
    done
}
# Arrays to emulate associative arrays for caching
CACHE_KEYS=()
CACHE_VALUES=()
CACHE_TIMESTAMPS=()

# Cache timeout in seconds (e.g., 5 minutes)
CACHE_TIMEOUT=300

# Function to find cache index
find_cache_index() {
    local key="$1"
    local i
    for ((i = 0; i < ${#CACHE_KEYS[@]}; i++)); do
        if [[ "${CACHE_KEYS[$i]}" == "$key" ]]; then
            echo "$i"
            return
        fi
    done
    echo "-1"
}

# Function to read and cache all properties of a given instance
cache_get_instance() {
    local class_id="$1"
    local instance_id="$2"
    local force=${3:-false}
    local current_time
    current_time=$(date +%s)

    local cache_key="${class_id}_${instance_id}"
    local cache_value
    local cache_timestamp

    # Find cache index for the given key
    local cache_index
    cache_index=$(find_cache_index "$cache_key")

    # Check if we should use the cached data
    if [[ "$force" != "true" && "$cache_index" -ge 0 ]]; then
        cache_value="${CACHE_VALUES[$cache_index]}"
        cache_timestamp="${CACHE_TIMESTAMPS[$cache_index]}"
        local cache_age=$((current_time - cache_timestamp))

        if [[ $cache_age -le $CACHE_TIMEOUT ]]; then
            echo "$cache_value"
            return 0
        fi
    fi

    if [ -z "$class_id" ] || [ -z "$instance_id" ]; then
        echo "Usage: cache_get_instance <class_id> <instance_id> [force]" >&2
        return 1
    fi

    if [ "$CLASS_CHECKS" == "true" ]; then
        if [[ $(validate_integer "$class_id") -ne 0 || $(validate_integer "$instance_id") -ne 0 ]]; then
            echo "cache_get_instance: Invalid class ID or instance ID: class_id: $class_id, instance_id: $instance_id" >&2
            return 1
        fi
        class_exists "$class_id" || return 1
    fi

    local query="SELECT d.property_id, d.value
                 FROM classes_properties p
                 JOIN classes_instances_data d ON p.id = d.property_id
                 WHERE d.instance_id = $instance_id;"

    local result
    result=$("""$DUCKDB_EXECUTABLE""" "$DUCKDB_FILE_NAME" -csv "$query" | tail -n +2)

    if [ -z "$result" ]; then
        echo "cache_get_instance: No properties found for instance: $instance_id in class: $class_id" >&2
        return 1
    fi

    # Update the cache
    if [[ "$cache_index" -ge 0 ]]; then
        CACHE_VALUES[$cache_index]="$result"
        CACHE_TIMESTAMPS[$cache_index]=$current_time
    else
        CACHE_KEYS+=("$cache_key")
        CACHE_VALUES+=("$result")
        CACHE_TIMESTAMPS+=("$current_time")
    fi

    echo "$result"
}

cache_get_property() {
    local class_id="$1"
    local instance_id="$2"
    local property_id="$3"
    local force=${4:-false}

    if [ -z "$class_id" ] || [ -z "$instance_id" ] || [ -z "$property_id" ]; then
        echo "Usage: cache_get_property <class_id> <instance_id> <property_id>" >&2
        return 1
    fi

    local properties
    properties=$(cache_get_instance "$class_id" "$instance_id" "$force")

    if [ $? -ne 0 ]; then
        echo "Failed to get properties for class_id: $class_id, instance_id: $instance_id" >&2
        return 1
    fi

    local result
    result=$(echo "$properties" | grep "^$property_id," | cut -d ',' -f 2)

    if [ -z "$result" ]; then
        echo "Property '$property_id' not found for instance_id: $instance_id in class_id: $class_id" >&2
        return 1
    fi

    echo "$result"
}

source ~/.xbashrc

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    if [ -z "$1" ]; then
        # No function name supplied, do nothing
        exit 0
    fi

    func_name="$1" # Store the first argument (function name)
    shift          # Remove the first argument, now $@ contains only the arguments for the function

    # Check if the function exists
    if declare -f "$func_name" >/dev/null; then
        "$func_name" "$@" # Call the function with the remaining arguments
    else
        log_fatal "'$func_name' is not a valid function name."
        exit 1
    fi
fi
