#!/usr/bin/env bash

map_new() {
    local map_name="$1"
    local data_type_id="$2"

    if [ -z "$map_name" ] || [ -z "$data_type_id" ]; then
        echo "Usage: map_new <map_name> <data_type_id>" >&2
        return 1
    fi

    if [[ $(validate_text "$map_name") -ne 0 || $(validate_integer "$data_type_id") -ne 0 ]]; then
        echo "Invalid map name or data type ID" >&2
        return 1
    fi

    map_id=$(""$DUCKDB_EXECUTABLE"" "$DUCKDB_FILE_NAME" -csv "INSERT INTO maps (map_name, data_type_id) VALUES ('$map_name', $data_type_id) RETURNING id;" | tail -n +2)

    if [ -z "$map_id" ]; then
        echo "Failed to create map" >&2
        return 1
    fi

    echo "$map_id"
}

map_exists() {
    local map_id="$1"

    if [ -z "$map_id" ]; then
        echo "Usage: map_exists <map_id>" >&2
        return 1
    fi

    if [[ $(validate_integer "$map_id") -ne 0 ]]; then
        echo "Invalid map ID" >&2
        return 1
    fi

    local result=$(""$DUCKDB_EXECUTABLE"" "$DUCKDB_FILE_NAME" -csv "SELECT COUNT(*) AS count FROM maps WHERE id = $map_id;")
    result=$(echo "$result" | tail -n 1) # Get the actual count result
    if [[ "$result" -eq 1 ]]; then
        return 0
    else
        return 1
    fi
}

map_clear() {
    local map_id="$1"

    if [ -z "$map_id" ]; then
        echo "Usage: map_clear <map_id>" >&2
        return 1
    fi

    if [[ $(validate_integer "$map_id") -ne 0 ]]; then
        echo "Invalid map ID" >&2
        return 1
    fi

    map_exists "$map_id" || {
        echo "Map does not exist" >&2
        return 1
    }

    ""$DUCKDB_EXECUTABLE"" "$DUCKDB_FILE_NAME" -csv "DELETE FROM maps_data WHERE map_id = $map_id;" || {
        echo "Failed to clear map" >&2
        return 1
    }
}

map_add_or_set() {
    local map_id="$1"
    local key="$2"
    local value="$3"

    if [ -z "$map_id" ] || [ -z "$key" ] || [ -z "$value" ]; then
        echo "Usage: map_add_or_set <map_id> <key> <value>" >&2
        return 1
    fi

    if [[ $(validate_integer "$map_id") -ne 0 || $(validate_text "$key") -ne 0 || $(validate_text "$value") -ne 0 ]]; then
        echo "Invalid map ID, key, or value" >&2
        return 1
    fi

    map_exists "$map_id" || {
        echo "Map does not exist" >&2
        return 1
    }

    ""$DUCKDB_EXECUTABLE"" "$DUCKDB_FILE_NAME" -csv "INSERT INTO maps_data (map_id, key, value, idx) VALUES ($map_id, '$key', '$value', nextval('seq_a')) ON CONFLICT (map_id, key) DO UPDATE SET value = EXCLUDED.value;" || {
        echo "Failed to add or set key-value pair" >&2
        return 1
    }
}

map_get() {
    local map_id="$1"
    local key="$2"

    if [ -z "$map_id" ] || [ -z "$key" ]; then
        echo "Usage: map_get <map_id> <key>" >&2
        return 1
    fi

    if [[ $(validate_integer "$map_id") -ne 0 || $(validate_text "$key") -ne 0 ]]; then
        echo "Invalid map ID or key" >&2
        return 1
    fi

    map_exists "$map_id" || return 1

    local value=$(""$DUCKDB_EXECUTABLE"" "$DUCKDB_FILE_NAME" -csv "SELECT value FROM maps_data WHERE map_id = $map_id AND key = '$key';" | tail -n 1)

    echo "value is $value" >&2

    if [[ -z "$value" ]]; then
        echo "Key not found" >&2
        return 1
    fi

    echo "$value"
}

map_contains_key() {
    local map_id="$1"
    local key="$2"

    if [ -z "$map_id" ] || [ -z "$key" ]; then
        echo "Usage: map_contains_key <map_id> <key>" >&2
        return 1
    fi

    if [[ $(validate_integer "$map_id") -ne 0 || $(validate_text "$key") -ne 0 ]]; then
        echo "Invalid map ID or key" >&2
        return 1
    fi

    map_exists "$map_id" || return 1

    local result=$(""$DUCKDB_EXECUTABLE"" "$DUCKDB_FILE_NAME" -csv "SELECT COUNT(*) AS count FROM maps_data WHERE map_id = $map_id AND key = '$key';")
    result=$(echo "$result" | tail -n 1) # Get the actual count result

    if [[ "$result" -eq 1 ]]; then
        echo 0
    else
        echo 1
    fi
}

map_remove() {
    local map_id="$1"
    local key="$2"

    if [ -z "$map_id" ] || [ -z "$key" ]; then
        echo "Usage: map_remove <map_id> <key>" >&2
        return 1
    fi

    if [[ $(validate_integer "$map_id") -ne 0 || $(validate_text "$key") -ne 0 ]]; then
        echo "Invalid map ID or key" >&2
        return 1
    fi

    map_exists "$map_id" || {
        echo "Map does not exist" >&2
        return 1
    }

    ""$DUCKDB_EXECUTABLE"" "$DUCKDB_FILE_NAME" -csv "DELETE FROM maps_data WHERE map_id = $map_id AND key = '$key';" || {
        echo "Failed to remove key-value pair" >&2
        return 1
    }
}

map_print() {
    local map_id="$1"

    if [ -z "$map_id" ]; then
        echo "Usage: map_print <map_id>" >&2
        return 1
    fi

    if [[ $(validate_integer "$map_id") -ne 0 ]]; then
        echo "Invalid map ID" >&2
        return 1
    fi

    map_exists "$map_id" || {
        echo "Map does not exist" >&2
        return 1
    }

    ""$DUCKDB_EXECUTABLE"" "$DUCKDB_FILE_NAME" -csv "SELECT key, value FROM maps_data WHERE map_id = $map_id ORDER BY idx;" || {
        echo "Failed to print map" >&2
        return 1
    }
}

map_sort_by_value() {
    local map_id="$1"

    if [ -z "$map_id" ]; then
        echo "Usage: map_sort_by_value <map_id>" >&2
        return 1
    fi

    if [[ $(validate_integer "$map_id") -ne 0 ]]; then
        echo "Invalid map ID" >&2
        return 1
    fi

    map_exists "$map_id" || {
        echo "Map does not exist" >&2
        return 1
    }

    ""$DUCKDB_EXECUTABLE"" "$DUCKDB_FILE_NAME" -csv "CREATE TABLE map_temp_sorted AS SELECT key, value, ROW_NUMBER() OVER (ORDER BY value) - 1 AS new_idx FROM maps_data WHERE map_id = $map_id;"
    ""$DUCKDB_EXECUTABLE"" "$DUCKDB_FILE_NAME" -csv "UPDATE maps_data SET idx = (SELECT new_idx FROM map_temp_sorted WHERE maps_data.map_id = $map_id AND maps_data.key = map_temp_sorted.key) WHERE map_id = $map_id;"
    ""$DUCKDB_EXECUTABLE"" "$DUCKDB_FILE_NAME" -csv "DROP TABLE map_temp_sorted;" || {
        echo "Failed to sort map" >&2
        return 1
    }
}

map_keys() {
    local map_id="$1"

    if [ -z "$map_id" ]; then
        echo "Usage: map_keys <map_id>" >&2
        return 1
    fi

    if [[ $(validate_integer "$map_id") -ne 0 ]]; then
        echo "Invalid map ID" >&2
        return 1
    fi

    map_exists "$map_id" || {
        echo "Map does not exist" >&2
        return 1
    }

    local keys=$(""$DUCKDB_EXECUTABLE"" "$DUCKDB_FILE_NAME" -csv "SELECT key FROM maps_data WHERE map_id = $map_id ORDER BY idx;")
    echo "$keys" | tail -n +2 | paste -sd "," -
}

map_cascade_subtract() {
    local map_id="$1"

    if [ -z "$map_id" ]; then
        echo "Usage: map_cascade_subtract <map_id>" >&2
        return 1
    fi

    if [[ $(validate_integer "$map_id") -ne 0 ]]; then
        echo "Invalid map ID" >&2
        return 1
    fi

    map_exists "$map_id" || {
        echo "Map does not exist" >&2
        return 1
    }

    map_sort_by_value "$map_id"

    local keys=($(""$DUCKDB_EXECUTABLE"" "$DUCKDB_FILE_NAME" -csv "SELECT key FROM maps_data WHERE map_id = $map_id ORDER BY idx;"))
    local values=($(""$DUCKDB_EXECUTABLE"" "$DUCKDB_FILE_NAME" -csv "SELECT value FROM maps_data WHERE map_id = $map_id ORDER BY idx;"))

    echo "Initial keys: ${keys[*]}" >&2
    echo "Initial values: ${values[*]}" >&2

    local num_values=${#values[@]}
    for ((i = num_values - 1; i > 0; i--)); do
        values[$i]=$((values[i] - values[i - 1]))
    done

    echo "Updated values: ${values[*]}" >&2

    for ((i = 0; i < num_values; i++)); do
        ""$DUCKDB_EXECUTABLE"" "$DUCKDB_FILE_NAME" -csv "UPDATE maps_data SET value = '${values[$i]}' WHERE map_id = $map_id AND key = '${keys[$i]}';" || {
            echo "Failed to update value for key ${keys[$i]}" >&2
            return 1
        }
    done
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
