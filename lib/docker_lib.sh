check_and_push_image() {
    local image_url=$1
    local directory=$2
    local force_rebuild=${3:-false}

    if [ -z "$image_url" ] || [ -z "$directory" ]; then
        log_fatal "Usage: check_and_push_image <image_url> <directory> [force_rebuild]"
        return 1
    fi

    # Check if image exists
    if [ $force_rebuild = "false" ] && check_image_exists "$image_url"; then
        log_debug "Image already exists: $image_url"
    else
        if [ $force_rebuild = "true" ]; then
            log_debug "Force rebuild is enabled. Removing the existing image..."
            docker rmi "$image_url" > /dev/null 2>&1
        else
            log_debug "Image does not exist, attempting to build and push..."
        fi

        # save the current directory
        local current_dir=$(pwd)

        # Change to the directory with the Docker Compose file
        cd "$directory" || return 1

        # Build and push the image using Docker Compose
        docker-compose build && docker-compose push

        # Return to the original directory
        cd $current_dir

        # Check again if the image exists
        if check_image_exists "$image_url"; then
            log_debug "Image has been successfully pushed: $image_url"
        else
            log_error "Failed to create or push the image."
            return 1
        fi
    fi
}

check_and_build_image() {
    local image_url=$1
    local directory=$2
    local force_rebuild=${3:-false}

    if [ -z "$image_url" ] || [ -z "$directory" ]; then
        log_fatal "Usage: check_and_build_image <image_url> <directory> [force_rebuild]"
        return 1
    fi

    # Check if image exists
    if [ $force_rebuild = "false" ] && check_image_exists "$image_url"; then
        log_debug "Image already exists: $image_url"
    else
        if [ $force_rebuild = "true" ]; then
            log_debug "Force rebuild is enabled. Removing the existing image..."
            docker rmi "$image_url" > /dev/null 2>&1
        else
            log_debug "Image does not exist, attempting to build..."
        fi

        local current_dir=$(pwd)

        # Change to the directory with the Docker Compose file
        cd "$directory" || return 1

        # Build the image using Docker Compose
        docker-compose build

        # Return to the original directory
        cd $current_dir

        # Check again if the image exists
        if check_image_exists "$image_url"; then
            log_debug "Image has been successfully built: $image_url"
        else
            log_error "Failed to build the image."
            return 1
        fi
    fi
}

check_image_exists() {
    local image_url="$1"

    if [ -z "$image_url" ]; then
        log_fatal "Usage: check_image_exists <image_url>"
        return 1
    fi

    # docker pull "$image_url" > /dev/null 2>&1
    #     return $?

    if docker image inspect "$image_url" > /dev/null 2>&1; then
        return 0  # Image exists
    else
        return 1  # Image does not exist
    fi
}

get_container_ip() {
    local container_name="$1"

    if [ -z "$container_name" ]; then
        log_fatal "Usage: get_container_ip <container_name>"
        return 1
    fi

    if ! container_exists "$container_name"; then
        log_error "Container $container_name does not exist."
        return 1
    fi

    local container_ip=$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' "$container_name")

    if [ -z "$container_ip" ]; then
        log_error "Failed to get IP address for container $container_name."
        return 1
    fi

    echo "$container_ip"
}

sync_docker_image() {
    local remote_full_image_url="$1"
    local local_full_image_url="$2"

    # Check if the image exists in the local registry
    if ! docker manifest inspect "${local_full_image_url}" > /dev/null 2>&1; then
        log_debug "Image not found in local registry. Syncing now..."

        # Pull the image from the remote location
        docker pull "${remote_full_image_url}"

        # Tag the image for the local registry
        docker tag "${remote_full_image_url}" "${local_full_image_url}"

        # Push the image to the local registry
        docker push "${local_full_image_url}"
    else
        log_debug "Image already exists in the local registry."
    fi
}

kill_and_remove_container_if_exists() {
    local container_name="$1"

    if [ -z "$container_name" ]; then
        log_fatal "Usage: kill_and_remove_container_if_exists <container_name>"
        return 1
    fi

    valid_container_names=("${CONTAINER_LIST[@]}")
    if ! contains "$container_name" "${valid_container_names[@]}"; then
        log_error "Container $container_name is not in the list of valid containers."
        return 1
    fi

    #check if container exists
    if container_exists "$container_name"; then
        # Check if the container is running and kill it
        if [ "$(docker ps -q -f name=^/${container_name}$)" ]; then
            log_state "Container $container_name is running, killing it..."
            docker kill "$container_name" > /dev/null 2>&1
        else
            log_debug "Container $container_name is not running."
        fi

        # Check if the container exists (running or stopped) and remove it
        if [ "$(docker ps -a -q -f name=^/${container_name}$)" ]; then
            log_debug "Removing container $container_name..."
            docker rm -v "$container_name" > /dev/null 2>&1
        else
            log_debug "Container $container_name does not exist, no action taken."
        fi
    else
        log_debug "Container $container_name does not exist."
    fi
}

login_to_docker_registry_if_needed() {
    # Check if already logged in
    if docker info | grep -q "Username: $REGISTRY_USERNAME"; then
        log_debug "Already logged in to the Docker registry."
        return 0
    fi

    # Prompt for password or read from a secured environment variable
    if [ -z "$REGISTRY_PASSWORD" ]; then
        log_fatal "Please export REGISTRY_PASSWORD with your Docker registry password."
        return 1
    fi

    docker_config_file="$MG_USER_HOME/.docker/config.json"
    touch "$docker_config_file"

    # update_json_file "$docker_config_file" "auths.$REGISTRY_HOSTNAME" "$(echo -n "$REGISTRY_USERNAME:$REGISTRY_PASSWORD" | base64)"
    update_json_file "$docker_config_file" "credsStore" "pass"

    # https://github.com/docker/docker-credential-helpers/releases/download/v0.8.1/docker-credential-pass-v0.8.1.linux-amd64
    # gpg --full-generate-key
    # pass init jowharshamshiri@gmail.com

    # Login without passing password on the command line
    echo "$REGISTRY_PASSWORD" | docker login -u "$REGISTRY_USERNAME" --password-stdin "$REGISTRY_HOSTNAME"
    if [ $? -ne 0 ]; then
        log_fatal "Failed to login to the Docker registry. Please check the credentials and try again."
        return 1
    fi
}

pull_docker_image_if_not_exists() {
    local IMAGE_NAME="$1"

    if [ -z "$IMAGE_NAME" ]; then
        log_fatal "Usage: pull_docker_image_if_not_exists <image_name>"
        return 1
    fi

    if ! docker image inspect "$IMAGE_NAME" > /dev/null 2>&1; then
        log_debug "Image $IMAGE_NAME not found locally. Pulling..."
        docker pull "$IMAGE_NAME"
    else
        log_debug "Image $IMAGE_NAME already exists locally."
    fi
}


function remove_all_docker_containers() {
    local docker_containers=$(docker ps -aq)

    if [ -n "$docker_containers" ]; then
        log_state "Stopping $(echo $docker_containers | wc -w) running Docker containers..."
        docker stop $docker_containers
    else
        log_debug "No running Docker containers found to stop."
    fi

    docker_containers=$(docker ps -aq)

    if [ -n "$docker_containers" ]; then
        log_debug "Removing $(echo $docker_containers | wc -w) Docker containers..."
        docker rm -f $docker_containers
    else
        log_debug "No Docker containers found to remove."
    fi
}


function container_exists() {
    local container_name="$1"

    if [ -z "$container_name" ]; then
        log_fatal "Usage: container_exists <container_name>"
        return 1
    fi

    if docker ps -a --format '{{.Names}}' | grep -q "^${container_name}$"; then
        return 0  # Container exists
    else
        return 1  # Container does not exist
    fi
}

function is_container_running() {
    local container_name="$1"

    if [ -z "$container_name" ]; then
        log_fatal "Usage: is_container_running <container_name>"
        return 1
    fi

    if docker ps --format '{{.Names}}' | grep -q "^${container_name}$"; then
        return 0  # Container is running
    else
        return 1  # Container is not running
    fi
}

function check_all_containers_exist() {
    local container_names=("$@")

    for container_name in "${container_names[@]}"; do
        if ! container_exists "$container_name"; then
            return 1  # At least one container does not exist
        fi
    done

    return 0  # All containers exist
}

function check_all_containers_running() {
    local container_names=("$@")

    for container_name in "${container_names[@]}"; do
        if container_exists "$container_name"; then
            if ! is_container_running "$container_name"; then
                return 1  # At least one container is not running
            fi
        fi
    done

    return 0  # All containers are running
}

function start_containers_if_not_running() {
    local container_names=("$@")

    for container_name in "${container_names[@]}"; do
        if ! is_container_running "$container_name"; then
            log_debug "Starting container $container_name..."
            docker start "$container_name"
        else
            log_debug "Container $container_name is already running."
        fi
    done
}

function get_docker_network_interface() {
    local network_name="$1"
    # Inspect the Docker network to get details in JSON format
    local network_info=$(docker network inspect "$network_name" 2>/dev/null)

    if [ -z "$network_info" ]; then
        log_error "Network not found or access denied"
        return 1
    fi

    local network_info_json=$(docker network inspect "$network_name" --format '{{json .}}' 2>/dev/null)

    local driver=$(echo "$network_info_json" | jq -r '.Driver')

    if [[ "$driver" != "bridge" ]]; then
        log_error "Docker network $network_name is not a bridge network."
        return 1
    fi

    # Parse the interface name from the network information
    # The bridge name for Docker networks is usually stored under the "Id" field,
    # and Docker often uses a truncated form of this ID for the bridge interface name.
    local network_id=$(echo "$network_info" | jq -r '.[0].Id')
    local interface_name="br-${network_id:0:12}"

    if [ -z "$network_id" ]; then
        log_error "Network ID not found, cannot determine interface name."
        return 1
    fi

    echo "$interface_name"
}

get_docker_network_subnet() {
    local network_name="$1"
    # Inspect the Docker network to get details in JSON format
    local network_info=$(docker network inspect "$network_name" 2>/dev/null)

    if [ -z "$network_info" ]; then
        log_error "Network not found or access denied"
        return 1
    fi

    local network_info_json=$(docker network inspect "$network_name" --format '{{json .}}' 2>/dev/null)

    local driver=$(echo "$network_info_json" | jq -r '.Driver')

    if [[ "$driver" != "bridge" ]]; then
        log_error "Docker network $network_name is not a bridge network."
        return 1
    fi

    # Parse the subnet from the network information
    local subnet=$(echo "$network_info_json" | jq -r '.IPAM.Config[0].Subnet')

    if [ -z "$subnet" ]; then
        log_error "Subnet not found, cannot determine network subnet."
        return 1
    fi

    echo "$subnet"
}

get_docker_network_gateway() {
    local network_name="$1"
    # Inspect the Docker network to get details in JSON format
    local network_info=$(docker network inspect "$network_name" 2>/dev/null)

    if [ -z "$network_info" ]; then
        log_error "Network not found or access denied"
        return 1
    fi

    local network_info_json=$(docker network inspect "$network_name" --format '{{json .}}' 2>/dev/null)

    local driver=$(echo "$network_info_json" | jq -r '.Driver')

    if [[ "$driver" != "bridge" ]]; then
        log_error "Docker network $network_name is not a bridge network."
        return 1
    fi

    # Parse the gateway from the network information
    local gateway=$(echo "$network_info_json" | jq -r '.IPAM.Config[0].Gateway')

    if [ -z "$gateway" ]; then
        log_error "Gateway not found, cannot determine network gateway."
        return 1
    fi

    echo "$gateway"
}

setup_docker_network() {
    local network_name="$1"
    local subnet="$2"
    local gateway="$3"
    local force_reset="${4:-false}"  # Default to false if not provided

    if [ -z "$network_name" ] || [ -z "$subnet" ] || [ -z "$gateway" ]; then
        log_fatal "Usage: setup_docker_network <network_name> <subnet> <gateway> [force_reset]"
        return 1
    fi

    # Check if the network exists
    local existing_network=$(docker network ls --filter name=^${network_name}$ --format "{{.Name}}")

    if [[ "$existing_network" == "$network_name" ]]; then
        log_debug "Network $network_name already exists."

        if [[ "$force_reset" == "true" ]]; then
            log_state "Force reset is enabled. Removing and recreating the network..."
        else
            # Fetch the existing network's subnet and gateway
            local existing_subnet=$(docker network inspect $network_name --format '{{range .IPAM.Config}}{{.Subnet}}{{end}}')
            local existing_gateway=$(docker network inspect $network_name --format '{{range .IPAM.Config}}{{.Gateway}}{{end}}')

            # Compare the existing subnet and gateway with the desired configuration
            if [[ "$existing_subnet" == "$subnet" && "$existing_gateway" == "$gateway" ]]; then
                log_debug "Network $network_name is correctly configured."
                return 0  # Exit if no updates are needed
            else
                log_state "Network configuration differs. Updating network..."
            fi
        fi

        # Remove the existing network
        docker network rm $network_name
        # There may be a delay needed here if the network deletion takes time to propagate
        sleep 2
    fi

    # Network does not exist or was removed, so create it
    log_debug "Creating network $network_name with subnet $subnet and gateway $gateway..."
    docker network create --subnet $subnet --gateway $gateway $network_name
    log_debug "Network $network_name has been created or updated with new settings."
}

source "/etc/machinegenesis/mg_env"
source "$INFRA_DIR/root.sh"

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

