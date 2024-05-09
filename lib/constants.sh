source "/etc/machinegenesis/mg_env"
source "$INFRA_DIR/root.sh"

REFERENCE_DOMAIN="google.com"
REFERENCE_DNS_IP="8.8.8.8"

# defined in root.sh:
# ABRCITY_HOME="$MG_HOME/abrcity"
# MACHINEFABRIC_HOME="$MG_HOME/machinefabric"
# LIB_DIR="$INFRA_DIR/lib"

PACKAGES_DIR="$INFRA_DIR/packages"
PACKAGE_REPOS_DIR="$INFRA_DIR/package_repos"

DUMMY_DOMAIN="acdummy.com"
MINIKUBE_HOSTNAME="$MACHINEGENESIS_DOMAIN"

get_ip_address() {
    local host_lan_adapter=$1
    local os_type=$(uname -s)
    local host_lan_ip=""

    case "$os_type" in
        Linux)
            # Using ip command available in modern Linux distributions
            host_lan_ip=$(ip -4 addr show $host_lan_adapter | grep -oP '(?<=inet\s)\d+(\.\d+){3}')
            ;;
        Darwin)
            # Using ifconfig command, typical in macOS
            host_lan_ip=$(ifconfig $host_lan_adapter | awk '/inet / {print $2}')
            ;;
        *)
            echo "Unsupported OS."
            return 1
            ;;
    esac

    echo $host_lan_ip
}

get_subnet() {
    local host_lan_adapter=$1
    local os_type=$(uname -s)
    local host_lan_subnet=""

    if [[ "$os_type" == "Darwin" ]]; then
        # Get the IP address and hex netmask
        local ip_address=$(ifconfig $host_lan_adapter | awk '/inet / {print $2}')
        local hex_netmask=$(ifconfig $host_lan_adapter | awk '/netmask / {print $4}')

        # Convert hex netmask to binary
        local binary_netmask=$(printf '%032b' "0x${hex_netmask}")

        # Calculate CIDR by counting the number of 1s in the binary netmask
        local cidr=$(echo "${binary_netmask}" | grep -o "1" | wc -l)

        host_lan_subnet="${ip_address}/${cidr}"
    elif [[ "$os_type" == "Linux" ]]; then
        host_lan_subnet=$(ip -4 addr show $host_lan_adapter | grep -oP '(?<=inet\s)\d+(\.\d+){3}/\d+')
    else
        echo "Unsupported OS."
        return 1
    fi

    echo $host_lan_subnet
}

HOST_LAN_IP=$(get_ip_address $HOST_LAN_ADAPTER)
HOST_LAN_SUBNET=$(get_subnet $HOST_LAN_ADAPTER)
# log_debug "HOST_LAN_IP: $HOST_LAN_IP"
# log_debug "HOST_LAN_SUBNET: $HOST_LAN_SUBNET"

HOST_WLAN_IP=$(get_ip_address $HOST_WLAN_ADAPTER)
HOST_WLAN_SUBNET=$(get_subnet $HOST_WLAN_ADAPTER)
# log_debug "HOST_WLAN_IP: $HOST_WLAN_IP"
# log_debug "HOST_WLAN_SUBNET: $HOST_WLAN_SUBNET"

ASUSPC_HOSTNAME="asus-pc.$DUMMY_DOMAIN"
ASUSPC_IP=$HOST_LAN_IP

MG_BASHRC_SECTION_START="# --------------- MACHINEGENESIS SECTION START ---------------"
MG_BASHRC_SECTION_END="# --------------- MACHINEGENESIS SECTION END ---------------"
MG_BASHRC_SECTION_COMMENT="# !! contents within this block are managed by machinegenesis scripts !!"

DEFAULT_TIMEOUT=30
DEFAULT_SLEEP_INTERVAL=5
LOCALHOST_IP="127.0.0.1"

LOG_LEVEL_TRACE=5
LOG_LEVEL_DEBUG=4
LOG_LEVEL_STATE=3
LOG_LEVEL_ALERT=2
LOG_LEVEL_ERROR=1
LOG_LEVEL_FATAL=0

RUN_UUID=$(uuidgen)
RUN_TEMP_DIR="/tmp/mg-$RUN_UUID"
mkdir -p "$RUN_TEMP_DIR"
RUN_STATE_FILE="$RUN_TEMP_DIR/state.json"
RUN_LOG_FILE="$RUN_TEMP_DIR/log.txt"
touch "$RUN_LOG_FILE"

