DUCKDB_FILE_NAME="crossbash.duckdb"

# Function to detect OS and architecture, and download the appropriate DuckDB binary
download_duckdb() {
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
            echo "Unsupported architecture: $arch"
            return 1
        fi
    else
        echo "Unsupported OS: $OSTYPE"
        return 1
    fi

    echo "Downloading DuckDB binary from $url..."
    curl -L "$url" -o duckdb.zip
    unzip duckdb.zip -d duckdb_bin
    mv duckdb_bin/duckdb* duckdb
    chmod +x duckdb
    rm -rf duckdb.zip duckdb_bin
    echo "DuckDB binary downloaded and ready to use."
}

DATATYPE_ID_INTEGER=1
DATATYPE_ID_TEXT=2
DATATYPE_ID_REAL=3
DATATYPE_ID_BOOLEAN=4
DATATYPE_ID_DATE=5
DATATYPE_ID_TIME=6
DATATYPE_ID_TIMESTAMP=7
DATATYPE_ID_BLOB=8

validate_integer() {
    local value=$1
    if [[ "$value" =~ ^-?[0-9]+$ ]]; then
        echo 0
    else
        echo 1
    fi
}

validate_text() {
    local value=$1
    if [[ -n "$value" && ! "$value" =~ ^[[:space:]]*$ ]]; then
        echo 0
    else
        echo 1
    fi
}

validate_real() {
    local value=$1
    if [[ "$value" =~ ^-?[0-9]+(\.[0-9]+)?$ ]]; then
        echo 0
    else
        echo 1
    fi
}

validate_boolean() {
    local value=$1
    if [[ "$value" == "TRUE" || "$value" == "FALSE" || "$value" == "true" || "$value" == "false" ]]; then
        echo 0
    else
        echo 1
    fi
}

validate_date() {
    local value=$1
    if [[ "$value" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ && "$value" > "0000-00-00" ]]; then
        echo 0
    else
        echo 1
    fi
}

validate_time() {
    local value=$1
    if [[ "$value" =~ ^[0-9]{2}:[0-9]{2}:[0-9]{2}$ && "$value" < "24:00:00" ]]; then
        echo 0
    else
        echo 1
    fi
}

validate_timestamp() {
    local value=$1
    if [[ "$value" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}[[:space:]][0-9]{2}:[0-9]{2}:[0-9]{2}$ ]]; then
        if [[ "$value" > "0000-00-00 00:00:00" ]]; then
            echo 0
            return
        fi
    fi
    echo 1
}

validate_blob() {
    local value=$1
    if [[ -n "$value" ]]; then
        echo 0
    else
        echo 1
    fi
}

# Function to initialize the DuckDB database
initialize_db() {
    ./duckdb $DUCKDB_FILE_NAME -csv "CREATE SEQUENCE IF NOT EXISTS seq_a START 1;" || {
        echo "Failed to create sequence"
        exit 1
    }

    #data type tables
    ./duckdb "$DUCKDB_FILE_NAME" -csv "CREATE TABLE IF NOT EXISTS data_types (id INTEGER PRIMARY KEY DEFAULT nextval('seq_a'), type TEXT);" || {
        echo "Failed to create data_types table"
        exit 1
    }

    DATATYPE_ID_INTEGER=$(./duckdb "$DUCKDB_FILE_NAME" -csv "INSERT INTO data_types(type) VALUES ('INTEGER') RETURNING id;" | tail -n +2)
    DATATYPE_ID_TEXT=$(./duckdb "$DUCKDB_FILE_NAME" -csv "INSERT INTO data_types(type) VALUES ('TEXT') RETURNING id;" | tail -n +2)
    DATATYPE_ID_REAL=$(./duckdb "$DUCKDB_FILE_NAME" -csv "INSERT INTO data_types(type) VALUES ('REAL') RETURNING id; " | tail -n +2)
    DATATYPE_ID_BOOLEAN=$(./duckdb "$DUCKDB_FILE_NAME" -csv "INSERT INTO data_types(type) VALUES ('BOOLEAN') RETURNING id;" | tail -n +2)
    DATATYPE_ID_DATE=$(./duckdb "$DUCKDB_FILE_NAME" -csv "INSERT INTO data_types(type) VALUES ('DATE') RETURNING id;" | tail -n +2)
    DATATYPE_ID_TIME=$(./duckdb "$DUCKDB_FILE_NAME" -csv "INSERT INTO data_types(type) VALUES ('TIME') RETURNING id;" | tail -n +2)
    DATATYPE_ID_TIMESTAMP=$(./duckdb "$DUCKDB_FILE_NAME" -csv "INSERT INTO data_types(type) VALUES ('TIMESTAMP') RETURNING id;" | tail -n +2)
    DATATYPE_ID_BLOB=$(./duckdb "$DUCKDB_FILE_NAME" -csv "INSERT INTO data_types(type) VALUES ('BLOB') RETURNING id;" | tail -n +2)

    #map tables
    ./duckdb "$DUCKDB_FILE_NAME" -csv "CREATE TABLE IF NOT EXISTS maps (id INTEGER PRIMARY KEY DEFAULT nextval('seq_a'), map_name TEXT, data_type_id INTEGER REFERENCES data_types(id));"
    ./duckdb "$DUCKDB_FILE_NAME" -csv "CREATE TABLE IF NOT EXISTS maps_data (map_id INTEGER REFERENCES maps(id), key TEXT, value TEXT, idx INTEGER, PRIMARY KEY (map_id, key));"

    #list tables
    ./duckdb $DUCKDB_FILE_NAME -csv "CREATE TABLE IF NOT EXISTS lists (id INTEGER PRIMARY KEY DEFAULT nextval('seq_a'), list_name TEXT, data_type_id INTEGER REFERENCES data_types(id));"
    ./duckdb $DUCKDB_FILE_NAME -csv "CREATE TABLE IF NOT EXISTS lists_data (list_id INTEGER REFERENCES lists(id), value TEXT, idx INTEGER, PRIMARY KEY (list_id, idx));"

    #class tables
    ./duckdb $DUCKDB_FILE_NAME -csv "CREATE TABLE IF NOT EXISTS classes (id INTEGER PRIMARY KEY DEFAULT nextval('seq_a'), class_name TEXT);"
    ./duckdb $DUCKDB_FILE_NAME -csv "CREATE TABLE IF NOT EXISTS classes_properties (id INTEGER PRIMARY KEY DEFAULT nextval('seq_a'), class_id INTEGER REFERENCES classes(id), property TEXT, data_type_id INTEGER REFERENCES data_types(id));"
    ./duckdb $DUCKDB_FILE_NAME -csv "CREATE TABLE IF NOT EXISTS classes_instances (class_id INTEGER REFERENCES classes(id), instance_id INTEGER UNIQUE DEFAULT nextval('seq_a'), PRIMARY KEY (class_id, instance_id), idx INTEGER);"
    ./duckdb $DUCKDB_FILE_NAME -csv "CREATE TABLE IF NOT EXISTS classes_instances_data (instance_id INTEGER REFERENCES classes_instances(instance_id), property_id INTEGER REFERENCES classes_properties(id), value TEXT, PRIMARY KEY (instance_id, property_id));"

    # Set tables
    ./duckdb "$DUCKDB_FILE_NAME" -csv "CREATE TABLE IF NOT EXISTS sets (id INTEGER PRIMARY KEY DEFAULT nextval('seq_a'), set_name TEXT, data_type_id INTEGER REFERENCES data_types(id));"
    ./duckdb "$DUCKDB_FILE_NAME" -csv "CREATE TABLE IF NOT EXISTS sets_data (set_id INTEGER REFERENCES sets(id), value TEXT, PRIMARY KEY (set_id, value));"

    # Queue tables
    ./duckdb "$DUCKDB_FILE_NAME" -csv "CREATE TABLE IF NOT EXISTS queues (id INTEGER PRIMARY KEY DEFAULT nextval('seq_a'), queue_name TEXT, data_type_id INTEGER REFERENCES data_types(id));"
    ./duckdb "$DUCKDB_FILE_NAME" -csv "CREATE TABLE IF NOT EXISTS queues_data (queue_id INTEGER REFERENCES queues(id), value TEXT, idx INTEGER, PRIMARY KEY (queue_id, idx));"

    # Multimap tables
    ./duckdb "$DUCKDB_FILE_NAME" -csv "CREATE TABLE IF NOT EXISTS multimaps (id INTEGER PRIMARY KEY DEFAULT nextval('seq_a'), multimap_name TEXT, data_type_id INTEGER REFERENCES data_types(id));"
    ./duckdb "$DUCKDB_FILE_NAME" -csv "CREATE TABLE IF NOT EXISTS multimaps_data (multimap_id INTEGER REFERENCES multimaps(id), key TEXT, value TEXT, idx INTEGER, PRIMARY KEY (multimap_id, key, idx));"

    # Stack tables
    ./duckdb "$DUCKDB_FILE_NAME" -csv "CREATE TABLE IF NOT EXISTS stacks (id INTEGER PRIMARY KEY DEFAULT nextval('seq_a'), stack_name TEXT, data_type_id INTEGER REFERENCES data_types(id));"
    ./duckdb "$DUCKDB_FILE_NAME" -csv "CREATE TABLE IF NOT EXISTS stacks_data (stack_id INTEGER REFERENCES stacks(id), value TEXT, idx INTEGER, PRIMARY KEY (stack_id, idx));"
}

# Main script logic
download_duckdb
initialize_db
