#!/bin/bash

update() {
    current_dir=$(pwd)
    abrcity_version=$(ac_version)
    machinegenesis_version=$(xb_version)

    cd "$ABRCITY_HOME" || exit
    abrcity_output=$(git pull)
    cd "$XB_HOME" || exit
    mg_output=$(git pull)

    new_abrcity_version=$(ac_version)
    new_machinegenesis_version=$(xb_version)

    pulled=false

    if [[ "$abrcity_version" != "$new_abrcity_version" ]]; then
        log_state "Abrcity updated from $abrcity_version to $new_abrcity_version"
        write_to_run_state "$RUN_STATE_KEY_UPDATE_OLD_AC_VERSION" "$abrcity_version"
        write_to_run_state "$RUN_STATE_KEY_UPDATE_NEW_AC_VERSION" "$new_abrcity_version"
        pulled=true
    else
        log_debug "Abrcity is already up to date: $abrcity_version"
    fi

    if [[ "$machinegenesis_version" != "$new_machinegenesis_version" ]]; then
        log_state "MachineGenesis updated from $machinegenesis_version to $new_machinegenesis_version"
        write_to_run_state "$RUN_STATE_KEY_UPDATE_OLD_MG_VERSION" "$machinegenesis_version"
        write_to_run_state "$RUN_STATE_KEY_UPDATE_NEW_MG_VERSION" "$new_machinegenesis_version"
        pulled=true
    else
        log_debug "MachineGenesis is already up to date: $machinegenesis_version"
    fi

    save_update_attempt_result "true" "$pulled"

    update_if_needed

    cd "$current_dir" || exit
}

save_function_call() {
    func_name="$1"
    func_args="$2"

    echo "2 Function name: $func_name, Function arguments: $func_args" >&2

    if [ -z "$func_name" ]; then
        log_fatal "Usage: save_function_call <function_name> <function_arguments>"
        return 1
    fi

    write_to_run_state "$RUN_STATE_KEY_FUNCTION_NAME" "$func_name"
    write_to_run_state "$RUN_STATE_KEY_FUNCTION_ARGS" "$func_args"
}

load_function_call() {
    if [ ! -f "$RUN_STATE_FILE" ]; then
        log_fatal "Run state file not found."
        return 1
    fi

    if ! is_set_in_run_state "$RUN_STATE_KEY_FUNCTION_NAME"; then
        log_fatal "Function name not found in run state."
        return 1
    fi

    function_name=$(read_from_run_state "$RUN_STATE_KEY_FUNCTION_NAME")
    function_args=$(read_from_run_state "$RUN_STATE_KEY_FUNCTION_ARGS")

    echo "$function_name $function_args"
}

recall_function_call() {
    command=$(load_function_call)
    return_code=$?
    if [ $return_code -ne 0 ]; then
        log_fatal "Failed to recall function call."
        return 1
    fi

    log_debug "Recalling command: $command"
    eval "$command"
}

save_update_attempt_result() {
    success="$1"
    pulled="$2"

    if [ -z "$success" ]; then
        log_fatal "Usage: save_update_attempt_result <success>"
        return 1
    fi

    if [ "$success" != "true" ] && [ "$success" != "false" ]; then
        log_fatal "Invalid value for success: $success: must be 'true' or 'false'"
        return 1
    fi

    if [ -z "$pulled" ]; then
        log_fatal "Usage: save_update_attempt_result <success> <pulled>"
        return 1
    fi

    if [ "$pulled" != "true" ] && [ "$pulled" != "false" ]; then
        log_fatal "Invalid value for pulled: $pulled: must be 'true' or 'false'"
        return 1
    fi

    write_to_run_state "$RUN_STATE_KEY_UPDATE_ATTEMPTED" "true"
    write_to_run_state "$RUN_STATE_KEY_UPDATE_SUCCESSFUL" "$success"
    write_to_run_state "$RUN_STATE_KEY_UPDATE_PULLED" "$pulled"
}

updated_was_attempted() {
    if ! is_set_in_run_state "$RUN_STATE_KEY_UPDATE_ATTEMPTED"; then
        log_trace "Update was not attempted."
        return 1
    fi

    result=$(read_from_run_state "$RUN_STATE_KEY_UPDATE_ATTEMPTED")

    if [ "$result" == "true" ]; then
        log_trace "Update was attempted."
        return 0
    fi

    if [ "$result" == "false" ]; then
        log_trace "Update was not attempted."
        return 1
    fi

    log_fatal "Invalid value for '$RUN_STATE_KEY_UPDATE_ATTEMPTED': $result"
}

update_succeeded() {
    if ! is_set_in_run_state "$RUN_STATE_KEY_UPDATE_SUCCESSFUL"; then
        log_fatal "Update result not found in run state."
        return 1
    fi

    result=$(read_from_run_state "$RUN_STATE_KEY_UPDATE_SUCCESSFUL")

    if [ $? -ne 0 ]; then
        log_fatal "Failed to read update result from run state."
        return 1
    fi

    if [ "$result" == "true" ]; then
        log_trace "Update succeeded."
        return 0
    fi

    if [ "$result" == "false" ]; then
        log_trace "Update failed."
        return 1
    fi

    log_fatal "Invalid value for '$RUN_STATE_KEY_UPDATE_SUCCESSFUL': $result"
}

update_pulled() {
    if ! is_set_in_run_state "$RUN_STATE_KEY_UPDATE_PULLED"; then
        log_fatal "Update pulled not found in run state."
        return 1
    fi

    result=$(read_from_run_state "$RUN_STATE_KEY_UPDATE_PULLED")

    if [ $? -ne 0 ]; then
        log_fatal "Failed to read update pulled from run state."
        return 1
    fi

    if [ "$result" == "true" ]; then
        log_trace "Update pulled."
        return 0
    fi

    if [ "$result" == "false" ]; then
        log_trace "Update did not pull."
        return 1
    fi

    log_fatal "Invalid value for '$RUN_STATE_KEY_UPDATE_PULLED': $result"
}

update_if_needed() {
    if updated_was_attempted; then
        log_trace "Update was attempted. Checking if it was successful..."
        if update_succeeded; then
            log_trace "Update was successful. Checking if it pulled..."
            if update_pulled; then
                log_debug "Update pulled. Now running $(load_function_call) again..."
                recall_function_call
            else
                log_debug "Update did not pull. Now running $(load_function_call)..."
                recall_function_call
            fi
        else
            log_debug "Update failed. Trying to run $(load_function_call)..."
            recall_function_call
        fi
    else
        log_debug "Checking for updates..."
        update
    fi
}

ac_version() {
    if [ ! -f "$AC_VERSION_FILE" ]; then
        log_fatal "Version file $AC_VERSION_FILE does not exist."
        exit 1
    fi

    ac_version=$(cat "$AC_VERSION_FILE")

    if [ -z "$ac_version" ]; then
        log_fatal "Version file $AC_VERSION_FILE is empty."
        exit 1
    fi

    echo "$ac_version"
}

new() {
    project_path=$1
    project_name=$2

    if [ -z "$project_path" ]; then
        echo
        return
    fi
}

# print_ac_header() {
#     # ANSI color codes for styling
#     NC='\033[0m' # No Color
#     GREEN='\033[0;32m'
#     BLUE='\033[0;34m'
#     YELLOW='\033[1;33m'
#     CYAN='\033[0;36m'
#     MAGENTA='\033[0;35m'

#     echo -e "${YELLOW}┌─┐┌┐ ┬─┐┌─┐┬┌┬┐┬ ┬"
#     echo -e "├─┤├┴┐├┬┘│  │ │ └┬┘"
#     echo -e "┴ ┴└─┘┴└─└─┘┴ ┴  ┴ ${MAGENTA}v$(ac_version)${NC}"
# }

print_help() {
    log_fatal "Usage: abr [command] [options]"
    echo "Commands:"
    echo "  up [--force-recreate] [--force-rebuild] Start all services"
    echo "  down                                    Stop all services"
    echo "  reset                                   Restart all services"
    echo "  status                                  Display the status of all services"
    echo "  help                                    Display this help message"
    echo "  service_status <service_name>           Display the status of a specific service"
    echo "  get_tls_certs                           Get TLS certificates from Let's Encrypt"
    echo "  setup_network                           Setup the Docker network"
    echo "  setup_dns                               Setup the DNS server"
    echo "  setup_gw                                Setup the Abrcity Gateway server"
    echo "  setup_registry                          Setup the Docker registry"
    echo "  setup_tftp                              Setup the TFTP server"
    echo "  setup_sa                                Setup the Static Assets server"
    echo "  setup_ntp                               Setup the NTP server"
    echo "  setup_squid                             Setup the Squid proxy server"
    echo "  setup_nfs                               Setup the NFS server"
    echo "  setup_pypi                              Setup the PyPI server"
    echo "  setup_jenkins                           Setup the Jenkins server"
    echo "  setup_dhcp                              Setup the DHCP server"
    echo "  setup_openvpn                           Setup the OpenVPN server"

    echo "Options:"
    echo "  --force-recreate                        Force recreate all containers"
    echo "  --force-rebuild                          Force rebuild all containers"
}

source ~/.xbashrc

# Prevent the file from being sourced multiple times
if [ -z "$AC_ROOT_SOURCED" ]; then
    AC_ROOT_SOURCED=1
    AC_LIB_DIR="$ABRCITY_HOME/lib"
    # add_wait_marquee

    # declare -a scripts_to_source
    # list_directory_contents "$AC_LIB_DIR" scripts_to_source

    # source_scripts "${scripts_to_source[@]}"

    # print_ac_header
    # remove_wait_marquee
    # load_layout_yaml "$XB_HOME/assets/layout.yaml"
    start_layout "dashboard"
fi

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    if [ -z "$1" ]; then
        # status
        exit 0
    fi

    if [ "$1" == "--help" ] || [ "$1" == "help" ]; then
        print_help
        exit 1
    fi

    func_name="$1" # The first argument is the function name
    echo "Function name: $func_name" >&2
    shift # Remove the first argument, now $@ contains only the arguments for the function
    echo "Function arguments: $@" >&2
    # Check if the function exists
    if declare -f "$func_name" >/dev/null; then
        save_function_call "$func_name" "$@"
        update_if_needed
        # "$func_name" "$@"
    else
        log_fatal "'$func_name' is not a valid command."
        print_help
        exit 1
    fi
fi
