#!/usr/bin/env bash

# Initializes a new stack with the given name
stack_new() {
    local stack_name="$1"
    local data_type_id="$2"

    if [ -z "$stack_name" ] || [ -z "$data_type_id" ]; then
        echo "Usage: stack_new <stack_name> <data_type_id>" >&2
        return 1
    fi

    if [[ $(validate_text "$stack_name") -ne 0 || $(validate_integer "$data_type_id") -ne 0 ]]; then
        echo "Invalid stack name or data type ID" >&2
        return 1
    fi

    stack_id=$(""$DUCKDB_EXECUTABLE"" "$DUCKDB_FILE_NAME" -csv "INSERT INTO stacks (stack_name, data_type_id) VALUES ('$stack_name', $data_type_id) RETURNING id;" | tail -n +2)

    if [ -z "$stack_id" ]; then
        echo "Failed to create stack" >&2
        return 1
    fi

    echo "$stack_id"
}

# Pushes an item onto the specified stack
stack_push() {
    local stack_id="$1"
    local value="$2"

    if [ -z "$stack_id" ] || [ -z "$value" ]; then
        echo "Usage: stack_push <stack_id> <value>" >&2
        return 1
    fi

    if [[ $(validate_integer "$stack_id") -ne 0 || $(validate_text "$value") -ne 0 ]]; then
        echo "Invalid stack ID or value" >&2
        return 1
    fi

    ""$DUCKDB_EXECUTABLE"" "$DUCKDB_FILE_NAME" -csv "INSERT INTO stacks_data (stack_id, value, idx) VALUES ($stack_id, '$value', nextval('seq_a'));" || {
        echo "Failed to push value onto stack" >&2
        return 1
    }
}

# Pops an item off the specified stack and returns it
stack_pop() {
    local stack_id="$1"

    if [ -z "$stack_id" ]; then
        echo "Usage: stack_pop <stack_id>" >&2
        return 1
    fi

    if [[ $(validate_integer "$stack_id") -ne 0 ]]; then
        echo "Invalid stack ID" >&2
        return 1
    fi

    local value=$(""$DUCKDB_EXECUTABLE"" "$DUCKDB_FILE_NAME" -csv "SELECT value FROM stacks_data WHERE stack_id = $stack_id ORDER BY idx DESC LIMIT 1;" | tail -n 1)

    if [[ -z "$value" ]]; then
        echo "Stack is empty" >&2
        return 1
    fi

    ""$DUCKDB_EXECUTABLE"" "$DUCKDB_FILE_NAME" -csv "DELETE FROM stacks_data WHERE stack_id = $stack_id AND value = '$value' AND idx = (SELECT idx FROM stacks_data WHERE stack_id = $stack_id ORDER BY idx DESC LIMIT 1);" || {
        echo "Failed to pop value from stack" >&2
        return 1
    }

    echo "$value"
}

# Stack Top: Peek the last element without removing it
stack_top() {
    local stack_id="$1"

    if [ -z "$stack_id" ]; then
        echo "Usage: stack_top <stack_id>" >&2
        return 1
    fi

    if [[ $(validate_integer "$stack_id") -ne 0 ]]; then
        echo "Invalid stack ID" >&2
        return 1
    fi

    local value=$(""$DUCKDB_EXECUTABLE"" "$DUCKDB_FILE_NAME" -csv "SELECT value FROM stacks_data WHERE stack_id = $stack_id ORDER BY idx DESC LIMIT 1;" | tail -n 1)

    if [[ -z "$value" ]]; then
        echo "Stack is empty" >&2
        return 1
    fi

    echo "$value"
}

# Clears all elements from the specified stack
stack_clear() {
    local stack_id="$1"

    if [ -z "$stack_id" ]; then
        echo "Usage: stack_clear <stack_id>" >&2
        return 1
    fi

    if [[ $(validate_integer "$stack_id") -ne 0 ]]; then
        echo "Invalid stack ID" >&2
        return 1
    fi

    ""$DUCKDB_EXECUTABLE"" "$DUCKDB_FILE_NAME" -csv "DELETE FROM stacks_data WHERE stack_id = $stack_id;" || {
        echo "Failed to clear stack" >&2
        return 1
    }
}

# Prints all elements in the specified stack
stack_print() {
    local stack_id="$1"

    if [ -z "$stack_id" ]; then
        echo "Usage: stack_print <stack_id>" >&2
        return 1
    fi

    if [[ $(validate_integer "$stack_id") -ne 0 ]]; then
        echo "Invalid stack ID" >&2
        return 1
    fi

    ""$DUCKDB_EXECUTABLE"" "$DUCKDB_FILE_NAME" -csv "SELECT value FROM stacks_data WHERE stack_id = $stack_id ORDER BY idx;" || {
        echo "Failed to print stack" >&2
        return 1
    }
}

# Returns the size of the specified stack
stack_size() {
    local stack_id="$1"

    if [ -z "$stack_id" ]; then
        echo "Usage: stack_size <stack_id>" >&2
        return 1
    fi

    if [[ $(validate_integer "$stack_id") -ne 0 ]]; then
        echo "Invalid stack ID" >&2
        return 1
    fi

    local size=$(""$DUCKDB_EXECUTABLE"" "$DUCKDB_FILE_NAME" -csv "SELECT COUNT(*) AS count FROM stacks_data WHERE stack_id = $stack_id;" | tail -n 1)

    echo "$size"
}

# Checks if the specified stack is empty
stack_is_empty() {
    local stack_id="$1"

    if [ -z "$stack_id" ]; then
        echo "Usage: stack_is_empty <stack_id>" >&2
        return 1
    fi

    if [[ $(validate_integer "$stack_id") -ne 0 ]]; then
        echo "Invalid stack ID" >&2
        return 1
    fi

    local size=$(stack_size "$stack_id")

    if [[ "$size" -eq 0 ]]; then
        return 0
    else
        return 1
    fi
}

# Duplicates the specified stack
stack_duplicate() {
    local stack_id="$1"

    if [ -z "$stack_id" ]; then
        echo "Usage: stack_duplicate <stack_id>" >&2
        return 1
    fi

    if [[ $(validate_integer "$stack_id") -ne 0 ]]; then
        echo "Invalid stack ID" >&2
        return 1
    fi

    local stack_name=$(""$DUCKDB_EXECUTABLE"" "$DUCKDB_FILE_NAME" -csv "SELECT stack_name FROM stacks WHERE id = $stack_id;" | tail -n 1)
    local data_type_id=$(""$DUCKDB_EXECUTABLE"" "$DUCKDB_FILE_NAME" -csv "SELECT data_type_id FROM stacks WHERE id = $stack_id;" | tail -n 1)

    if [[ -z "$stack_name" || -z "$data_type_id" ]]; then
        echo "Stack not found" >&2
        return 1
    fi

    local new_stack_id=$(stack_new "${stack_name}_copy" "$data_type_id")

    ""$DUCKDB_EXECUTABLE"" "$DUCKDB_FILE_NAME" -csv "INSERT INTO stacks_data (stack_id, value, idx) SELECT $new_stack_id, value, idx FROM stacks_data WHERE stack_id = $stack_id;" || {
        echo "Failed to duplicate stack" >&2
        return 1
    }

    echo "$new_stack_id"
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
