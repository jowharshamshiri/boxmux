#!/usr/bin/env bash

# Function to detect OS and architecture, and download the appropriate DuckDB binary
download_duckdb() {
    if [ -z "$DUCKDB_EXECUTABLE" ] || [ ! -f "$DUCKDB_EXECUTABLE" ]; then
        local url=""
        if [[ "$OSTYPE" == "darwin"* ]]; then
            url="https://github.com/duckdb/duckdb/releases/download/v0.10.2/duckdb_cli-osx-universal.zip"
        elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
            local arch
            arch=$(uname -m)
            if [[ "$arch" == "x86_64" ]]; then
                url="https://github.com/duckdb/duckdb/releases/download/v0.10.2/duckdb_cli-linux-amd64.zip"
            elif [[ "$arch" == "aarch64" ]]; then
                url="https://github.com/duckdb/duckdb/releases/download/v0.10.2/duckdb_cli-linux-aarch64.zip"
            else
                log_fatal "Unsupported architecture: $arch"
                return 1
            fi
        else
            log_fatal "Unsupported OS: $OSTYPE"
            return 1
        fi

        log_state "Downloading DuckDB binary from $url..."
        add_download_and_download "$url"

        download_path=$(get_download_path "$url")

        if was_downloaded "$url"; then
            log_debug "DuckDB binary downloaded successfully"

            unzip "$download_path" -d "$RUN_WORKSPACE/duckdb_bin"

            # Find the executable within the extracted files
            local extracted_duckdb
            extracted_duckdb=$(find "$RUN_WORKSPACE/duckdb_bin" -type f -name 'duckdb*' | head -n 1)

            if [[ -z "$extracted_duckdb" ]]; then
                log_error "Failed to find DuckDB executable in the extracted files. Deleting the extracted files..."
                rm -rf "$RUN_WORKSPACE/duckdb_bin"
                return 1
            fi

            mv "$extracted_duckdb" "$RUN_WORKSPACE/duckdb"
            DUCKDB_EXECUTABLE="$RUN_WORKSPACE/duckdb"
            chmod +x "$DUCKDB_EXECUTABLE"
            set_env_var "DUCKDB_EXECUTABLE" "$DUCKDB_EXECUTABLE"
            rm -rf "$RUN_WORKSPACE/duckdb.zip" "$RUN_WORKSPACE/duckdb_bin"
            log_debug "DuckDB binary saved to $DUCKDB_EXECUTABLE"
        else
            log_error "Failed to download DuckDB binary"
            return 1
        fi
    else
        log_debug "DuckDB binary already exists at $DUCKDB_EXECUTABLE"
    fi
}

validate_data_type() {
    local data_type_id=$1
    local value=$2

    if [[ $(validate_integer "$data_type_id") -ne 0 ]]; then
        log_error "Invalid data type ID"
        return 1
    fi

    local data_type
    data_type=$(run_duckdb_csv_query "SELECT type FROM data_types WHERE id=$data_type_id;" | tail -n +2)

    if [ -z "$data_type" ]; then
        log_error "Data type with ID $data_type_id does not exist"
        return 1
    fi

    case "$data_type" in
    INTEGER)
        if [[ $(validate_integer "$value") -ne 0 ]]; then
            log_error "Invalid INTEGER value"
            return 1
        fi
        ;;
    TEXT)
        if [[ $(validate_text "$value") -ne 0 ]]; then
            log_error "Invalid TEXT value"
            return 1
        fi
        ;;
    REAL)
        if [[ $(validate_real "$value") -ne 0 ]]; then
            log_error "Invalid REAL value"
            return 1
        fi
        ;;
    BOOLEAN)
        if [[ $(validate_boolean "$value") -ne 0 ]]; then
            log_error "Invalid BOOLEAN value"
            return 1
        fi
        ;;
    DATE)
        if [[ $(validate_date "$value") -ne 0 ]]; then
            log_error "Invalid DATE value"
            return 1
        fi
        ;;
    TIME)
        if [[ $(validate_time "$value") -ne 0 ]]; then
            log_error "Invalid TIME value"
            return 1
        fi
        ;;
    TIMESTAMP)
        if [[ $(validate_timestamp "$value") -ne 0 ]]; then
            log_error "Invalid TIMESTAMP value"
            return 1
        fi
        ;;
    BLOB)
        if [[ $(validate_blob "$value") -ne 0 ]]; then
            log_error "Invalid BLOB value"
            return 1
        fi
        ;;
    *)
        log_error "Unsupported data type: $data_type"
        return 1
        ;;
    esac
}

validate_integer() {
    local value=$1
    if [[ "$value" =~ ^-?[0-9]+$ ]]; then
        return 0
    else
        return 1
    fi
}

validate_text() {
    local value=$1
    if [[ -n "$value" && ! "$value" =~ ^[[:space:]]*$ ]]; then
        return 0
    else
        return 1
    fi
}

validate_real() {
    local value=$1
    if [[ "$value" =~ ^-?[0-9]+(\.[0-9]+)?$ ]]; then
        return 0
    else
        return 1
    fi
}

validate_boolean() {
    local value=$1
    if [[ "$value" == "TRUE" || "$value" == "FALSE" || "$value" == "true" || "$value" == "false" ]]; then
        return 0
    else
        return 1
    fi
}

validate_date() {
    local value=$1
    if [[ "$value" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ && "$value" > "0000-00-00" ]]; then
        return 0
    else
        return 1
    fi
}

validate_time() {
    local value=$1
    if [[ "$value" =~ ^[0-9]{2}:[0-9]{2}:[0-9]{2}$ && "$value" < "24:00:00" ]]; then
        return 0
    else
        return 1
    fi
}

validate_timestamp() {
    local value=$1
    if [[ "$value" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}[[:space:]][0-9]{2}:[0-9]{2}:[0-9]{2}$ ]]; then
        if [[ "$value" > "0000-00-00 00:00:00" ]]; then
            return 0
            return
        fi
    fi
    return 1
}

validate_blob() {
    local value=$1
    if [[ -n "$value" ]]; then
        return 0
    else
        return 1
    fi
}

check_duckdb_state() {
    if [ -z "$DUCKDB_EXECUTABLE" ] || [ ! -f "$DUCKDB_EXECUTABLE" ]; then
        log_fatal "DUCKDB_EXECUTABLE is not set or the file does not exist."
        return 1
    fi
    if ! [ -x "$DUCKDB_EXECUTABLE" ]; then
        log_fatal "$DUCKDB_EXECUTABLE is not executable(Permission denied)."
        return 1
    fi
    if [ -z "$DUCKDB_FILE_NAME" ] || [ ! -f "$DUCKDB_FILE_NAME" ]; then
        log_fatal "DuckDB database not initialized or the file does not exist."
        return 1
    fi
    if ! [ ! -r "$DUCKDB_FILE_NAME" ]; then
        log_fatal "DuckDB database is not readable."
        return 1
    fi
    if [ ! -w "$DUCKDB_FILE_NAME" ]; then
        log_fatal "DuckDB database is writable. Please make it read-only."
        return 1
    fi
}

run_duckdb_csv_query() {
    local query="$1"

    if [ -z "$query" ]; then
        log_fatal "Usage: run_duckdb_query <query>"
        return 1
    fi

    # check_duckdb_state
    # output=$("$DUCKDB_EXECUTABLE" "$DUCKDB_FILE_NAME" -c "$query" -csv ".exit" 2>&1)

    "$DUCKDB_EXECUTABLE" "$DUCKDB_FILE_NAME" -csv "$query"
}

setup_data_types() {
    get_or_create_data_type_id() {
        local type=$1
        local result=$(run_duckdb_csv_query "SELECT id FROM data_types WHERE type='$type';")
        if [[ -z $result ]]; then
            result=$(run_duckdb_csv_query "INSERT INTO data_types(type) VALUES ('$type') RETURNING id;" | tail -n +2)
        fi
        echo "$result"
    }

    if [ -z "$DATATYPE_ID_INTEGER" ]; then
        DATATYPE_ID_INTEGER=$(get_or_create_data_type_id 'INTEGER')
        set_env_var "DATATYPE_ID_INTEGER" "$DATATYPE_ID_INTEGER"
    fi

    if [ -z "$DATATYPE_ID_TEXT" ]; then
        DATATYPE_ID_TEXT=$(get_or_create_data_type_id 'TEXT')
        set_env_var "DATATYPE_ID_TEXT" "$DATATYPE_ID_TEXT"
    fi

    if [ -z "$DATATYPE_ID_REAL" ]; then
        DATATYPE_ID_REAL=$(get_or_create_data_type_id 'REAL')
        set_env_var "DATATYPE_ID_REAL" "$DATATYPE_ID_REAL"
    fi

    if [ -z "$DATATYPE_ID_BOOLEAN" ]; then
        DATATYPE_ID_BOOLEAN=$(get_or_create_data_type_id 'BOOLEAN')
        set_env_var "DATATYPE_ID_BOOLEAN" "$DATATYPE_ID_BOOLEAN"
    fi

    if [ -z "$DATATYPE_ID_DATE" ]; then
        DATATYPE_ID_DATE=$(get_or_create_data_type_id 'DATE')
        set_env_var "DATATYPE_ID_DATE" "$DATATYPE_ID_DATE"
    fi

    if [ -z "$DATATYPE_ID_TIME" ]; then
        DATATYPE_ID_TIME=$(get_or_create_data_type_id 'TIME')
        set_env_var "DATATYPE_ID_TIME" "$DATATYPE_ID_TIME"
    fi

    if [ -z "$DATATYPE_ID_TIMESTAMP" ]; then
        DATATYPE_ID_TIMESTAMP=$(get_or_create_data_type_id 'TIMESTAMP')
        set_env_var "DATATYPE_ID_TIMESTAMP" "$DATATYPE_ID_TIMESTAMP"
    fi

    if [ -z "$DATATYPE_ID_BLOB" ]; then
        DATATYPE_ID_BLOB=$(get_or_create_data_type_id 'BLOB')
        set_env_var "DATATYPE_ID_BLOB" "$DATATYPE_ID_BLOB"
    fi

}

run_duck_db_csv_script() {
    local file_path="$1"

    # -- Check if the file exists
    if [[ ! -f "$file_path" ]]; then
        log_fatal "File not found: $file_path"
        return 1
    fi

    # -- Read the file line by line
    while IFS= read -r line; do
        # -- Trim leading and trailing whitespace
        line=$(echo "$line" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')

        # -- Ignore lines that begin with -- or whitespace followed by --
        [[ "$line" =~ ^--.* ]] && continue
        [[ "$line" =~ ^[[:space:]]*--.* ]] && continue

        # -- Ignore everything after the first --
        line=$(echo "$line" | sed 's/--.*//')

        # -- Trim the line again to remove any trailing whitespace after removing comments
        line=$(echo "$line" | sed 's/[[:space:]]*$//')

        # -- Print the processed line if it is not empty
        [[ -n "$line" ]] && { run_duckdb_csv_query "$line" && log_trace "DUCKDB: $line"; }
    done <"$file_path"
}

# Function to initialize the DuckDB database
initialize_db() {
    if [ "$DUCKDB_FKS" = "false" ]; then
        run_duck_db_csv_script "$XB_HOME/assets/db_no_fks.sql"
    else
        run_duck_db_csv_script "$XB_HOME/assets/db_with_fks.sql"
    fi
    setup_data_types
}

setting() {
    local key=$1
    local value=$2
    local data_type_id=${3:-$DATATYPE_ID_TEXT}

    if [ -z "$key" ]; then
        log_fatal "Usage: set_settings <key> [<value>]"
        return 1
    fi

    if [ -z "$value" ]; then
        result=$(run_duckdb_csv_query "SELECT value FROM settings WHERE key='$key';")
        log_trace "Setting $key is $result"
        echo "$result"
    else
        if validate_data_type "$data_type_id" "$value"; then
            log_debug "Setting '$key' being set to $value"
            run_duckdb_csv_query "INSERT INTO settings (key, data_type_id, value) VALUES ('$key', $data_type_id, '$value') ON CONFLICT (key) DO UPDATE SET value=excluded.value;"
        else
            log_error "Cannot set setting '$key'. Invalid value for data type '$data_type_id': $value"
            return 1
        fi
    fi
}

clear_settings() {
    run_duckdb_csv_query "DELETE FROM settings;"
}

clear_setting() {
    local key=$1

    if [ -z "$key" ]; then
        log_fatal "Usage: clear_setting <key>"
        return 1
    fi

    run_duckdb_csv_query "DELETE FROM settings WHERE key='$key';"
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
