#!/usr/bin/env bash

function setup_nat() {
    local network_1_adapter="$1"
    local network_2_adapter="$2"
    local chain_name="$3"

    # Create the custom chain if it does not exist
    if ! sudo iptables -L "$chain_name" &>/dev/null; then
        log_debug "Creating custom chain: $chain_name"
        sudo iptables -N "$chain_name"
    fi

    # Check if the chain is referenced in the FORWARD chain, if not, add it
    if ! sudo iptables -C FORWARD -j "$chain_name" &>/dev/null; then
        log_debug "Adding $chain_name to FORWARD chain"
        sudo iptables -I FORWARD -j "$chain_name"
    fi

    # Define rules
    local rule1="-i $network_1_adapter -o $network_2_adapter -j ACCEPT"
    local rule2="-i $network_2_adapter -o $network_1_adapter -j ACCEPT"

    # Check if rule1 exists in the custom chain, if not, add it
    if ! iptables -C "$chain_name" "$rule1" &>/dev/null; then
        log_debug "Adding rule: $rule1 to $chain_name"
        sudo iptables -A "$chain_name" "$rule1"
    fi

    # Check if rule2 exists in the custom chain, if not, add it
    if ! iptables -C "$chain_name" "$rule2" &>/dev/null; then
        log_debug "Adding rule: $rule2 to $chain_name"
        sudo iptables -A "$chain_name" "$rule2"
    fi
}

ensure_iptables_rule() {
    local table="$1"
    local chain="$2"
    local rule="$3"
    local full_rule="-t $table -A $chain $rule"

    # Check if the rule already exists
    if ! sudo iptables -t "$table" -C "$chain" "$rule" 2>/dev/null; then
        log_debug "Adding rule to $chain: $rule"
        sudo iptables "$full_rule"
    else
        log_debug "Rule already exists in $chain: $rule"
    fi
}

setup_custom_chain() {
    local table="$1"
    local custom_chain="$2"
    local main_chain="$3"

    # Create the custom chain if it does not exist
    if ! sudo iptables -t "$table" -L "$custom_chain" 2>/dev/null; then
        log_debug "Creating custom chain: $custom_chain"
        sudo iptables -t "$table" -N "$custom_chain"
    fi

    # Link the custom chain to the main chain if not already linked
    if ! sudo iptables -t "$table" -C "$main_chain" -j "$custom_chain" 2>/dev/null; then
        log_debug "Linking $custom_chain to $main_chain"
        sudo iptables -t "$table" -A "$main_chain" -j "$custom_chain"
    fi
}

delete_custom_chain() {
    local custom_chain="${1:-CUSTOM_CHAIN}"

    # If a specific custom chain is provided, try to delete it from all tables
    if [[ -n "$custom_chain" && "$custom_chain" != "CUSTOM_CHAIN" ]]; then
        for table in filter nat mangle raw security; do
            sudo iptables -t "$table" -X "$custom_chain" 2>/dev/null || true
        done
    fi
}

save_iptables() {
    local ip4_backup_file="${1:-/etc/iptables/rules.v4}"
    local ip6_backup_file="${2:-/etc/iptables/rules.v6}"

    backup_file "${ip4_backup_file}"
    backup_file "${ip6_backup_file}"

    sudo iptables-save | sudo tee "$ip4_backup_file" >/dev/null
    sudo ip6tables-save | sudo tee "$ip6_backup_file" >/dev/null
}

reset_iptables() {
    local custom_chain="${1:-CUSTOM_CHAIN}"

    # Flush all rules and delete all custom chains in all tables
    for table in filter nat mangle raw security; do
        sudo iptables -t "$table" -F
        sudo iptables -t "$table" -X
    done

    # If a specific custom chain is provided, try to delete it from all tables
    if [[ -n "$custom_chain" && "$custom_chain" != "CUSTOM_CHAIN" ]]; then
        for table in filter nat mangle raw security; do
            sudo iptables -t "$table" -X "$custom_chain" 2>/dev/null || true
        done
    fi

    # Set default policies for the filter table
    sudo iptables -P INPUT ACCEPT
    sudo iptables -P FORWARD ACCEPT
    sudo iptables -P OUTPUT ACCEPT

    if sudo systemctl is-active --quiet docker; then
        sudo systemctl restart docker
    fi
    if sudo systemctl is-active --quiet libvirtd; then
        sudo systemctl restart libvirtd
    fi

    save_iptables
}

is_valid_mac_address() {
    local mac_address="$1"

    if [[ -z "$mac_address" ]]; then
        log_fatal "Usage: is_valid_mac_address <mac_address>"
        return 1
    fi

    if [[ ! "$mac_address" =~ ^([0-9a-fA-F]{2}:){5}[0-9a-fA-F]{2}$ ]]; then
        return 1
    fi
}

create_basic_bridge() {
    local bridge_name="$1"

    if [[ -z "$bridge_name" ]]; then
        log_fatal "Usage: create_basic_bridge <bridge_name>"
        return 1
    fi

    if ip link show "$bridge_name" &>/dev/null; then
        log_debug "Bridge $bridge_name already exists."
        return 0
    fi

    # Create the bridge interface
    sudo ip link add name "$bridge_name" type bridge

    # Bring the bridge interface up
    sudo ip link set "$bridge_name" up

    echo "Bridge $bridge_name created and started."
}

create_static_bridge() {
    local bridge_name="$1"
    local bridge_ip="$2" # Ensure this includes CIDR notation, e.g., 192.168.1.1/24
    # local bridge_subnet="$3"
    # local bridge_gateway="$4"
    local bridge_mac_address="$3"
    # || -z "$bridge_gateway"|| -z "$bridge_subnet"
    if [[ -z "$bridge_name" || -z "$bridge_ip" || -z "$bridge_mac_address" ]]; then
        log_fatal "Usage: create_static_bridge <bridge_name> <bridge_ip> <bridge_subnet> <bridge_gateway> [bridge_mac_address]"
        return 1
    fi

    if bridge_exists "$bridge_name"; then
        log_debug "Bridge $bridge_name already exists."
        return 0
    fi

    # Create the bridge interface
    sudo ip link add name "$bridge_name" type bridge
    sudo ip link set "$bridge_name" up
    sudo ip link set dev "$bridge_name" address "$bridge_mac_address"
    sudo ip addr add "$bridge_ip" dev "$bridge_name"
    # sudo ip route add default via "$bridge_gateway"
    # { [[ -n "$bridge_mac_address" ]] &&
    if [[ $? -eq 0 ]]; then
        echo "Bridge $bridge_name created and started successfully."
    else
        log_fatal "Failed to create and configure bridge $bridge_name."
        return 1
    fi
}

create_dhcp_managed_bridge() {
    local bridge_name="${1:-$DEFAULT_LIBVIRT_BRIDGE_NAME}"
    local bridge_mac_address="${2:-$DEFAULT_LIBVIRT_BRIDGE_MAC_ADDRESS}"

    if [[ -z "$bridge_name" ]]; then
        log_fatal "Usage: create_dhcp_managed_bridge <bridge_name> [bridge_mac_address]"
        return 1
    fi

    if bridge_exists "$bridge_name"; then
        log_debug "Bridge $bridge_name already exists."
        return 0
    fi

    if [[ -n "$bridge_mac_address" ]]; then
        if ! is_valid_mac_address "$bridge_mac_address"; then
            log_fatal "Invalid MAC address: $bridge_mac_address"
            return 1
        fi
    fi

    if ! sudo which dhclient &>/dev/null; then
        log_fatal "dhclient is not installed."
        return 1
    fi

    # log_trace "Calling ip link add name $bridge_name type bridge"

    # Create the bridge interface
    # sudo ip link add name "$bridge_name" type bridge
    # sudo ip link set "$bridge_name" master "$HOST_LAN_ADAPTER"
    # sudo ip link set "$HOST_LAN_ADAPTER" master "$bridge_name"
    # sudo ip link set "$HOST_LAN_ADAPTER" nomaster
    # sudo ip link set dev "$bridge_name" promisc on

    # if ! ip link show "$bridge_name" &>/dev/null; then
    #     log_fatal "Failed to create bridge $bridge_name."
    #     return 1
    # fi

    # Set the MAC address if specified
    # if [[ -n "$bridge_mac_address" ]]; then
    #     log_trace "Setting MAC address $bridge_mac_address for bridge $bridge_name"
    #     sudo ip link set dev "$bridge_name" address "$bridge_mac_address"
    # fi

    # log_trace "Bringing the bridge interface up"
    # Bring the bridge interface up
    # sudo ip link set "$bridge_name" up

    # sleep 3s

    # if ! ip link show "$bridge_name" | grep "UP" &>/dev/null; then
    #     log_fatal "$bridge_name could not be brought up."
    #     return 1
    # fi

    # sudo systemctl restart isc-dhcp-relay

    # log_trace "Starting DHCP client to get IP and gateway via DHCP"
    # Start DHCP client to get IP and gateway via DHCP
    # sudo dhclient -1 -v "$bridge_name"

    # echo "Bridge $bridge_name is up and configured via DHCP."
}

device_broadcast_address() {
    ip addr show dev "$DEV" | grep brd | awk '/inet / {print $4}'
}

subnet_broadcast_address() {
    IP=$1
    NETMASK=$2

    if [[ -z "$IP" || -z "$NETMASK" ]]; then
        log_fatal "Usage: subnet_broadcast_address <ip_address> <netmask>"
        return 1
    fi

    # Convert IP and netmask to binary, then perform bitwise OR to get broadcast
    IFS=. read -r i1 i2 i3 i4 <<<"$IP"
    IFS=. read -r m1 m2 m3 m4 <<<"$NETMASK"

    # Calculate broadcast by ORing the inverted netmask with the IP address
    BCAST=$(printf "%d.%d.%d.%d\n" "$((i1 | ~m1 & 255))" "$((i2 | ~m2 & 255))" "$((i3 | ~m3 & 255))" "$((i4 | ~m4 & 255))")

    echo "$BCAST"
}

cidr_broadcast_address() {
    cidr_address=$1

    if [[ -z "$cidr_address" ]]; then
        log_fatal "Usage: cidr_broadcast_address <cidr_address>"
        return 1
    fi

    # Extract IP address and CIDR prefix from input
    IFS=/ read -r IP CIDR <<<"$cidr_address"

    # Initialize subnet mask
    MASK=""
    for ((i = 1; i <= 32; i++)); do
        if [[ $i -le $CIDR ]]; then
            MASK+="1"
        else
            MASK+="0"
        fi

        # Add dots between octets
        if (($i % 8 == 0)) && [ $i -ne 32 ]; then
            MASK+="."
        fi
    done

    # Convert binary mask to decimal IP form
    IFS=. read -r mb1 mb2 mb3 mb4 <<<"$MASK"
    MASK_IP=$(printf "%d.%d.%d.%d\n" "$((2#$mb1))" "$((2#$mb2))" "$((2#$mb3))" "$((2#$mb4))")

    # Calculate the broadcast address using bitwise operations
    IFS=. read -r i1 i2 i3 i4 <<<"$IP"
    IFS=. read -r m1 m2 m3 m4 <<<"$MASK_IP"

    # Invert mask for broadcast calculation
    m1=$((~m1 & 0xFF))
    m2=$((~m2 & 0xFF))
    m3=$((~m3 & 0xFF))
    m4=$((~m4 & 0xFF))

    BCAST=$(printf "%d.%d.%d.%d\n" "$((i1 | m1))" "$((i2 | m2))" "$((i3 | m3))" "$((i4 | m4))")

    echo "$BCAST"
}

random_mac_address() {
    MAC=$(hexdump -n6 -e '/1 ":%02X"' /dev/urandom | sed 's/^://; s/\(..\)/\10/' | sed 's/^\(..\)/\1/' | awk -F: '{printf "%02X:%s:%s:%s:%s:%s\n", $1 - ($1 % 2), $2, $3, $4, $5, $6}')
    echo "$MAC"
}

delete_bridge() {
    local bridge_name=$1

    if [[ -z "$bridge_name" ]]; then
        log_fatal "Usage: delete_bridge <bridge_name>"
        return 1
    fi

    if ! ip link show "$bridge_name" &>/dev/null; then
        log_debug "Bridge $bridge_name does not exist."
        return 0
    fi

    if ip link show "$bridge_name" | grep -q "state UP"; then
        log_debug "Bringing down bridge $bridge_name..."
        sudo ip link set "$bridge_name" down
    fi

    # Remove the bridge interface
    sudo ip link delete "$bridge_name" type bridge

    echo "Bridge $bridge_name stopped and removed."
}

bridge_exists() {
    local bridge_name="$1"

    if [[ -z "$bridge_name" ]]; then
        log_fatal "Usage: bridge_exists <bridge_name>"
        return 1
    fi

    ip link show "$bridge_name" &>/dev/null
}

is_bridge_up() {
    local bridge_name="$1"

    if [[ -z "$bridge_name" ]]; then
        log_fatal "Usage: is_bridge_up <bridge_name>"
        return 1
    fi

    ip link show "$bridge_name" | grep -q "state UP"
}

get_bridge_ip() {
    local bridge_name="$1"

    if [[ -z "$bridge_name" ]]; then
        log_fatal "Usage: get_bridge_ip <bridge_name>"
        return 1
    fi

    if ! bridge_exists "$bridge_name"; then
        log_fatal "Bridge $bridge_name does not exist."
        return 1
    fi

    if ! is_bridge_up "$bridge_name"; then
        log_fatal "Bridge $bridge_name is not up."
        return 1
    fi

    ip -4 addr show "$bridge_name" | grep -oP '(?<=inet\s)\d+(\.\d+){3}'
}

get_bridge_subnet() {
    local bridge_name="$1"

    if [[ -z "$bridge_name" ]]; then
        log_fatal "Usage: get_bridge_subnet <bridge_name>"
        return 1
    fi

    if ! bridge_exists "$bridge_name"; then
        log_fatal "Bridge $bridge_name does not exist."
        return 1
    fi

    if ! is_bridge_up "$bridge_name"; then
        log_fatal "Bridge $bridge_name is not up."
        return 1
    fi

    ip -4 addr show "$bridge_name" | grep -oP '(?<=inet\s)\d+(\.\d+){3}/\d+'
}

get_bridge_gateway() {
    local bridge_name="$1"

    if [[ -z "$bridge_name" ]]; then
        log_fatal "Usage: get_bridge_gateway <bridge_name>"
        return 1
    fi

    if ! bridge_exists "$bridge_name"; then
        log_fatal "Bridge $bridge_name does not exist."
        return 1
    fi

    if ! is_bridge_up "$bridge_name"; then
        log_fatal "Bridge $bridge_name is not up."
        return 1
    fi

    ip route show dev "$bridge_name" | grep -oP '(?<=via\s)\d+(\.\d+){3}'
}

get_bridge_mac_address() {
    local bridge_name="$1"

    if [[ -z "$bridge_name" ]]; then
        log_fatal "Usage: get_bridge_mac_address <bridge_name>"
        return 1
    fi

    if ! bridge_exists "$bridge_name"; then
        log_fatal "Bridge $bridge_name does not exist."
        return 1
    fi

    ip link show "$bridge_name" | grep -oP '(?<=link/ether\s)[0-9a-f:]+'
}

bring_bridge_up() {
    local bridge_name="$1"

    if [[ -z "$bridge_name" ]]; then
        log_fatal "Usage: bring_bridge_up <bridge_name>"
        return 1
    fi

    if ! bridge_exists "$bridge_name"; then
        log_fatal "Bridge $bridge_name does not exist."
        return 1
    fi

    if is_bridge_up "$bridge_name"; then
        log_debug "Bridge $bridge_name is already up."
        return 0
    fi

    sudo ip link set "$bridge_name" up

    if [[ $? -eq 0 ]]; then
        echo "Bridge $bridge_name brought up successfully."
    else
        log_fatal "Failed to bring up bridge $bridge_name."
        return 1
    fi
}

bring_bridge_down() {
    local bridge_name="$1"

    if [[ -z "$bridge_name" ]]; then
        log_fatal "Usage: bring_bridge_down <bridge_name>"
        return 1
    fi

    if ! bridge_exists "$bridge_name"; then
        log_fatal "Bridge $bridge_name does not exist."
        return 1
    fi

    if ! is_bridge_up "$bridge_name"; then
        log_debug "Bridge $bridge_name is already down."
        return 0
    fi

    sudo ip link set "$bridge_name" down

    if [[ $? -eq 0 ]]; then
        echo "Bridge $bridge_name brought down successfully."
    else
        log_fatal "Failed to bring down bridge $bridge_name."
        return 1
    fi
}

get_nth_available_ip() {
    local subnet="$1"
    local gateway="$2"
    local nth="$3" # The n-th available IP after the gateway

    if [ -z "$subnet" ] || [ -z "$gateway" ] || [ -z "$nth" ]; then
        log_fatal "Usage: get_nth_available_ip <subnet> <gateway> <nth>"
        return 1
    fi

    # Validate the nth value
    if ! [[ "$nth" =~ ^[0-9]+$ ]] || [ "$nth" -lt 1 ]; then
        log_fatal "Invalid number for the n-th available IP. Must be a positive integer."
        return 1
    fi

    # Extract the base IP and the last octet of the gateway IP
    local base_ip="${gateway%.*}"     # Remove the last octet
    local last_octet="${gateway##*.}" # Extract the last octet

    # Calculate the nth available IP by incrementing the last octet of the gateway by nth (instead of nth - 1)
    local nth_available=$((last_octet + nth))

    # Combine the base IP with the new last octet
    local nth_available_ip="$base_ip.$nth_available"

    echo "$nth_available_ip"
}

# start_bridge_dhcp_server() {
#     local bridge_name="$1"
#     local dhcp_range_start="$2"
#     local dhcp_range_end="$3"
#     local dhcp_subnet="$4"
#     local dhcp_dns="$5"
#     local dhcp_router="$6"
#     local dhcp_lease_time="${7:-12h}"

#     if [[ -z "$bridge_name" || -z "$dhcp_range_start" || -z "$dhcp_range_end" || -z "$dhcp_subnet" || -z "$dhcp_dns" || -z "$dhcp_router" ]]; then
#         log_fatal "Usage: start_bridge_dhcp_server <bridge_name> <dhcp_range_start> <dhcp_range_end> <dhcp_subnet> <dhcp_dns> <dhcp_router> [dhcp_lease_time]"
#         return 1
#     fi

#     # Check if the bridge exists
#     if ! bridge_exists "$bridge_name"; then
#         log_fatal "Bridge $bridge_name does not exist."
#         return 1
#     fi

#     # Check if the bridge is up
#     if ! is_bridge_up "$bridge_name"; then
#         log_fatal "Bridge $bridge_name is not up."
#         return 1
#     fi

#     # Check if the DHCP server is already running
#     if pgrep -f "dnsmasq --interface=$bridge_name" &>/dev/null; then
#         log_debug "DHCP server is already running on bridge $bridge_name."
#         return 0
#     fi

#     # Start the DHCP server
#     sudo dnsmasq --interface="$bridge_name" \
#                  --dhcp-range="$dhcp_range_start,$dhcp_range_end,$dhcp_subnet" \
#                  --dhcp-option=option:dns-server,"$dhcp_dns" \
#                  --dhcp-option=option:router,"$dhcp_router" \
#                  --dhcp-lease-max=253 \
#                  --dhcp-lease-time="$dhcp_lease_time" \
#                  --bind-interfaces \
#                  --quiet-dhcp \
#                  --quiet-dhcp6 \
#                  --quiet-ra \
#                  --no-resolv \
#                  --no-ping \
#                  --no-hosts \
#                  --no-daemon

#     if [[ $? -eq 0 ]]; then
#         echo "DHCP server started on bridge $bridge_name."
#     else
#         log_fatal "Failed to start DHCP server on bridge $bridge_name."
#         return 1
#     fi
# }

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
