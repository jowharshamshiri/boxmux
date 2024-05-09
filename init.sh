#!/bin/bash

valid_bash_var_name() {
    local input_string="$1"

    # Replace invalid characters with underscores
    # and ensure the string starts with a letter or underscore.
    # sed 's/[^a-zA-Z0-9_]/_/g' <<< "$var_name"
    local sanitized_string=$(echo "$input_string" | sed 's/[^a-zA-Z0-9_]/_/g')

    echo "$sanitized_string"
}

ensure_env_file_exists() {
    if [ -z "$ENV_FILE" ]; then
        log_error "Environment file path is not set."
        return 1
    fi

    sudo touch "$ENV_FILE"
    sudo chown "$MG_USER:$MG_USERGROUP" "$ENV_FILE"
    # Ensure the .env file is only writable by the owner but readable by everyone
    sudo chmod 640 "$ENV_FILE"

    if [ -f "$ENV_FILE" ]; then
        if [ ! -r "$ENV_FILE" ]; then
            log_error "Environment file is not readable."
            return 1
        fi
        if [ ! -w "$ENV_FILE" ]; then
            log_error "Environment file is not writable."
            return 1
        fi
        if [ ! -s "$ENV_FILE" ]; then
            log_state "Environment file is empty. Writing initial values."
            set_env_var "MG_HOME" "$MG_HOME"
            set_env_var "INFRA_DIR" "$INFRA_DIR"
            set_env_var "INFRA_DIR" "$INFRA_DIR"
            set_env_var "ABRCITY_HOME" "$ABRCITY_HOME"
            set_env_var "MACHINEFABRIC_HOME" "$MACHINEFABRIC_HOME"
            set_env_var "MG_WORKDIR" "$MG_WORKDIR"
            set_env_var "MG_USER" "$MG_USER"
            set_env_var "MG_USERGROUP" "$MG_USERGROUP"
            set_env_var "MG_USER_HOME" "$MG_USER_HOME"
            set_env_var "LOG_LEVEL" "$LOG_LEVEL"
            set_env_var "HOST_LAN_ADAPTER" "$HOST_LAN_ADAPTER"
            set_env_var "HOST_WLAN_ADAPTER" "$HOST_WLAN_ADAPTER"
        fi
    fi
}

escape_value() {
    local value="$1"
    echo "$value" | sed -e "s/'/'\\\\''/g; 1s/^/'/; \$s/\$/'/"
}

update_env_file() {
    local var_name="$1"
    local value="$2"
    local valid_var_name=$(valid_bash_var_name "$var_name")
    local escaped_value=$(escape_value "$value")

    if [ -z "$ENV_FILE" ]; then
        log_error "Environment file path is not set."
        return 1
    fi

    if [ ! -f "$ENV_FILE" ]; then
        log_error "Environment file does not exist."
        return 1
    elif [ ! -w "$ENV_FILE" ]; then
        log_error "Environment file is not writable."
        return 1
    fi

    # Check if the variable already exists and replace or append accordingly
    if grep -q "^export $valid_var_name=" "$ENV_FILE"; then
        # Variable exists, replace its value
        sed -i '' "s|^export $valid_var_name=.*|export $valid_var_name=$escaped_value|" "$ENV_FILE"
    else
        # Variable does not exist, append to file
        echo "export $valid_var_name=$escaped_value" >> "$ENV_FILE"
    fi
}

load_env_file_if_not_set() {
    if [ -z "$ENV_FILE" ]; then
        log_fatal "Environment file path is not set."
        return 1
    fi

    if [ -f "$ENV_FILE" ]; then
        if [ ! -r "$ENV_FILE" ]; then
            log_fatal "Environment file is not readable."
            return 1
        fi
    else
        log_fatal "Environment file does not exist."
        return 1
    fi

    # Read each line from the env file
    while IFS='=' read -r key value; do
        # Remove 'export ' from the start of the key if present
        key=$(echo "$key" | sed 's/^export //')

        # Trim leading and trailing whitespace from key
        key=$(echo "$key" | xargs)

        # Check if variable is unset in the current environment
        if [ -z "${!key}" ]; then
            # Evaluate the line directly to export it
            eval "$key=$value"
            log_debug "Set $key to $value"
        else
            log_debug "$key is already set to ${!key}"
        fi
    done < "$ENV_FILE"
}

set_env_var() {
    local var_name="$1"
    local value="$2"

    # Check if variable name and value were provided
    if [[ -z "$var_name" || -z "$value" ]]; then
        log_fatal "Usage: set_env_var <variable_name> <value>"
        return 1
    fi

    # Check if the value contains newlines
    if [[ "$value" == *$'\n'* ]]; then
        # Value contains newlines; wrap it in quotes
        value="\"$value\""
    elif [[ "$value" == *\ * ]]; then
        # Value contains spaces; wrap it in quotes
        value="\"$value\""
    fi

    # Export the environment variable
    export "$var_name=$value"
    log_debug "Set $var_name=$value"
    update_env_file "$var_name" "$value"
}

load_env_file() {
    if [ -f "$ENV_FILE" ]; then
        if [ ! -r "$ENV_FILE" ]; then
            log_fatal "Environment file is not readable."
            return 1
        fi
    else
        log_fatal "Environment file does not exist."
        return 1
    fi

    source "$ENV_FILE"
}

check_all_vars_set_in_env_file() {
    local var_names=("$@")

    if [ -z "$var_names" ]; then
        log_fatal "Usage: check_all_vars_set_in_env_file <variable_name1> <variable_name2> ..."
        return 1
    fi

    for var_name in "${var_names[@]}"; do
        local valid_var_name=$(valid_bash_var_name "$var_name")

        if ! grep -q "^export $valid_var_name=" "$ENV_FILE"; then
            log_error "Variable $var_name is not declared in the .env file."
            return 1
        fi

        if ! grep -q "^export $valid_var_name=.*" "$ENV_FILE"; then
            log_error "Variable $var_name does not have a value in the .env file."
            return 1
        fi

        if [ -z "$(grep "^export $valid_var_name=.*" "$ENV_FILE" | cut -d'=' -f2-)" ]; then
            log_error "Variable $var_name has empty value in the .env file."
            return 1
        fi
    done
}

check_all_ip_vars_set_in_env_file_and_reachable() {
    local var_names=("$@")

    if [ -z "$var_names" ]; then
        log_fatal "Usage: check_all_ip_vars_set_in_env_file_and_reachable <variable_name1> [<variable_name2> ...]"
        return 1
    fi

    for var_name in "${var_names[@]}"; do
        local valid_var_name=$(valid_bash_var_name "$var_name")

        if ! grep -q "^export $valid_var_name=" "$ENV_FILE"; then
            log_error "Variable $var_name is not declared in the .env file."
            return 1
        fi

        if ! grep -q "^export $valid_var_name=.*" "$ENV_FILE"; then
            log_error "Variable $var_name does not have a value in the .env file."
            return 1
        fi

        local ip_address=$(grep "^export $valid_var_name=.*" "$ENV_FILE" | cut -d'=' -f2-)
        if [ -z "$ip_address" ]; then
            log_error "Variable $var_name has empty value in the .env file."
            return 1
        fi

        if ! ping -c 1 -W 1 "$ip_address" > /dev/null 2>&1; then
            log_error "IP address $ip_address for $var_name is not reachable."
            return 1
        fi
    done
}

check_var_set_in_env_file() {
    local var_name="$1"

    if [ -z "$var_name" ]; then
        log_fatal "Usage: check_var_set_in_env_file <variable_name>"
        return 1
    fi

    local valid_var_name=$(valid_bash_var_name "$var_name")

    if ! grep -q "^export $valid_var_name=" "$ENV_FILE"; then
        log_error "Variable $var_name is not declared in the .env file."
        return 1
    fi

    if ! grep -q "^export $valid_var_name=.*" "$ENV_FILE"; then
        log_error "Variable $var_name does not have a value in the .env file."
        return 1
    fi

    if [ -z "$(grep "^export $valid_var_name=.*" "$ENV_FILE" | cut -d'=' -f2-)" ]; then
        log_error "Variable $var_name has empty value in the .env file."
        return 1
    fi
}

update_bashrc(){
    local instantiated_template="$RUN_TEMP_DIR/mf-bashrc-section.sh"
    cp "$INFRA_DIR/bashrc-section-template.sh" "$instantiated_template"
    sed -i '' "s|ENV_FILE|$ENV_FILE|g" "$instantiated_template"
    sed -i '' "s|MG_HOME|$MG_HOME|g" "$instantiated_template"
    sed -i '' "s|MG_WORKDIR|$MG_WORKDIR|g" "$instantiated_template"
    sed -i '' "s|MG_USER|$MG_USER|g" "$instantiated_template"
    sed -i '' "s|MG_USER_HOME|$MG_USER_HOME|g" "$instantiated_template"
    #the order of following line 3 lines is important
    sed -i '' "s|MACHINEFABRIC_HOME|$MACHINEFABRIC_HOME|g" "$instantiated_template"
    sed -i '' "s|ABRCITY_HOME|$ABRCITY_HOME|g" "$instantiated_template"
    sed -i '' "s|INFRA_DIR|$INFRA_DIR|g" "$instantiated_template"

    update_file_section "$instantiated_template" "$MG_USER_HOME/.bashrc" "$MG_BASHRC_SECTION_START" "$MG_BASHRC_SECTION_END" "$MG_BASHRC_SECTION_COMMENT"
}

mg_version() {
    local mg_version_file="$MG_HOME/version.txt"

    if [ ! -f "$mg_version_file" ]; then
        log_fatal "Version file not found."
    fi

    local result=$(cat "$mg_version_file")

    if [ -z "$result" ]; then
        log_fatal "Version file is empty."
        return 1
    fi

    echo "$result"
}

source_scripts() {
    scripts_to_source=("$@")
    for script_path in "${scripts_to_source[@]}"; do
        source_script "$script_path"
        # prefix_source "$MG_HOME" "$script_path"
    done
}

list_directory_contents() {
    local dir_path="$1"
    local arr_name=$2

    # Check if the directory exists
    if [[ ! -d "$dir_path" ]]; then
        log_fatal "Directory does not exist."
        return 1
    fi

    # Clear the array using indirect expansion
    eval "$arr_name=()"

    # Populate the array with absolute paths
    local item
    for item in "$dir_path"/*; do
        if [[ -e "$item" ]]; then # Check if the item exists
            # macOS compatible realpath alternative
            local abs_path="$(cd "$(dirname "$item")"; pwd)/$(basename "$item")"
            # Append to array indirectly
            eval "$arr_name+=(\"\$abs_path\")"
        fi
    done
}


print_mg_header() {
    # ANSI color codes for styling
    NC='\033[0m'  # No Color
    GREEN='\033[0;32m'
    BLUE='\033[0;34m'
    YELLOW='\033[1;33m'
    CYAN='\033[0;36m'
    MAGENTA='\033[0;35m'

    echo -e "${YELLOW}┌┬┐┌─┐┌─┐┬ ┬┬┌┐┌┌─┐┌─┐┌─┐┌┐┌┌─┐┌─┐┬┌─┐"
    echo -e "│││├─┤│  ├─┤││││├┤ │ ┬├┤ │││├┤ └─┐│└─┐"
    echo -e "┴ ┴┴ ┴└─┘┴ ┴┴┘└┘└─┘└─┘└─┘┘└┘└─┘└─┘┴└─┘ ${MAGENTA}v$(mg_version)${NC}"

    echo -e "${YELLOW}State file: ${MAGENTA}$RUN_STATE_FILE${NC}"
    echo -e "${YELLOW}Log file: ${MAGENTA}$RUN_LOG_FILE${NC}"
}

    # prefix_source() {
    #     local base_dir="$1"
    #     local file_path="$2"
    #     local prefix="${file_path#$base_dir/}"  # Remove base directory from file path
    #     prefix="${prefix%.*}"                 # Remove file extension
    #     prefix="${prefix//\//__}"             # Replace path separators with double underscores
    #     prefix="${prefix//[^a-zA-Z0-9_]/_}"   # Replace non-alphanumeric characters with underscores

    #     # Read the script into an array, line by line
    #     local script_lines=()
    #     local declared_funcs=()
    #     local declared_vars=()
    #     while IFS= read -r line; do
    #         script_lines+=("$line")
    #         if [[ "$line" =~ ^function[[:space:]]+([a-zA-Z_][a-zA-Z0-9_]*) ]] || [[ "$line" =~ ^([a-zA-Z_][a-zA-Z0-9_]*)[[:space:]]*\(\) ]]; then
    #             declared_funcs+=("${BASH_REMATCH[1]}")
    #         elif [[ "$line" =~ ^([a-zA-Z_][a-zA-Z0-9_]*)([[:space:]]*=[[:space:]]*) ]]; then
    #             declared_vars+=("${BASH_REMATCH[1]}")
    #         fi
    #     done < "$file_path"

    #     # Create a temporary file for the modified script
    #     local temp_script="/tmp/${prefix}_$(basename "$file_path")"
    #     > "$temp_script"

    #     # Modify the script lines by replacing local references with prefixed ones where applicable
    #     for line in "${script_lines[@]}"; do
    #         for func in "${declared_funcs[@]}"; do
    #             # Replace function calls with prefixed version only if they are not defined as a function or variable locally
    #             if ! [[ " ${declared_funcs[*]} " =~ " $func " || " ${declared_vars[*]} " =~ " $func " ]]; then
    #                 line="${line//\b$func\b/${prefix}__$func}"
    #             fi
    #         done
    #         for var in "${declared_vars[@]}"; do
    #             # Replace variable references with prefixed version only if they are not defined as a function or variable locally
    #             if ! [[ " ${declared_vars[*]} " =~ " $var " || " ${declared_funcs[*]} " =~ " $var " ]]; then
    #                 line="${line//\b$var\b/${prefix}__$var}"
    #             fi
    #         done
    #         echo "$line" >> "$temp_script"
    #     done

    #     # Source the modified script
    #     . "$temp_script"
    #     rm "$temp_script"  # Clean up the temp file
    # }



# Prevent the file from being sourced multiple times
if [ -z "$ROOT_BASH_SOURCED" ]; then
    ROOT_BASH_SOURCED=1

    source_script() {
        local script_path="$1"
        if [ ! -f "$script_path" ]; then
            echo -e "$(date +'%Y-%m-%d %H:%M:%S') [ERROR] Script not found: $script_path"
            return 1
        fi
        source "$script_path"
    }

    LOG_LEVEL=5
    ENV_FILE="/etc/machinegenesis/mg_env"
    sudo mkdir -p "$(dirname "$ENV_FILE")"
    sudo touch "$ENV_FILE"
    sudo chmod +x "$ENV_FILE"

    INFRA_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    MG_HOME="$(dirname "$INFRA_DIR")"

    echo "Please enter the home directory for MachineGenesis:"
    read -p "[$MG_HOME]: " input
    MG_HOME=${input:-$MG_HOME}

    source_script "$INFRA_DIR/lib/log_lib.sh"

    ABRCITY_HOME="$MG_HOME/abrcity"
    MACHINEFABRIC_HOME="$MG_HOME/machinefabric"
    INFRA_DIR="$MG_HOME/infra"
    LIB_DIR="$INFRA_DIR/lib"

    MG_USER=$(whoami)
    echo "Please enter the user for MachineGenesis"
    read -p "[$MG_USER]: " input
    MG_USER=${input:-$MG_USER}

    MG_USERGROUP=$(id -gn "$MG_USER")
    echo "Please enter the user group for MachineGenesis:"
    read -p "[$MG_USERGROUP]: " input
    MG_USERGROUP=${input:-$MG_USERGROUP}

    MG_USER_HOME=$(eval echo ~$MG_USER)

    MG_WORKDIR="$MG_USER_HOME/mg"

    echo "Please enter the work directory for MachineGenesis:"
    read -p "[$MG_WORKDIR]: " input
    MG_WORKDIR=${input:-$MG_WORKDIR}

    HOST_LAN_ADAPTER="eno2"

    echo "Please enter the host LAN adapter:"
    read -p "[$HOST_LAN_ADAPTER]: " input
    HOST_LAN_ADAPTER=${input:-$HOST_LAN_ADAPTER}

    HOST_WLAN_ADAPTER="wlo1"

    echo "Please enter the host WLAN adapter:"
    read -p "[$HOST_WLAN_ADAPTER]: " input
    HOST_WLAN_ADAPTER=${input:-$HOST_WLAN_ADAPTER}


    declare -a scripts_to_source
    list_directory_contents "$LIB_DIR" scripts_to_source

    source_scripts "${scripts_to_source[@]}"

    print_mg_header
    ensure_env_file_exists
    update_bashrc
    install_dependency_repos
    install_dependencies
    manual_setup
    write_to_run_state "timestamp" "$(date +'%Y-%m-%d %H:%M:%S')"
    write_to_run_state "mg_version" "$(mg_version)"
    write_to_run_state "running_user" "$(whoami)"
fi

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    if [ -z "$1" ]; then
        # No function name supplied, do nothing
        exit 0
    fi

    func_name="$1"  # Store the first argument (function name)
    shift           # Remove the first argument, now $@ contains only the arguments for the function

    # Check if the function exists
    if declare -f "$func_name" > /dev/null; then
        "$func_name" "$@"  # Call the function with the remaining arguments
    else
        log_fatal "'$func_name' is not a valid function name."
        exit 1
    fi
fi
