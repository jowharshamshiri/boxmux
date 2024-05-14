#!/bin/bash
#DO NOT USE ECHO, INSTEAD USE PRINTF
#MUST STAY COMPATIBLE WITH BASH 3

source "./parse_yaml.sh"
source "./map.sh"
source "./stack.sh"
source "./class.sh"

HOR_LINE_CHAR="━"
VER_LINE_CHAR="┃"
TOP_LEFT_CHAR="┏"
TOP_RIGHT_CHAR="┓"
BOTTOM_LEFT_CHAR="┗"
BOTTOM_RIGHT_CHAR="┛"

RED="\033[0;31m"
GREEN="\033[0;32m"
YELLOW="\033[0;33m"
BLUE="\033[0;34m"
MAGENTA="\033[0;35m"
CYAN="\033[0;36m"
WHITE="\033[0;37m"
BLACK="\033[0;30m"
RESET="\033[0m"

random_prefix() {
    echo "$(date +%s | sha256sum | base64 | head -c 6)"
}

separator="___"
prefix="$(random_prefix)"$separator""

root_element_name="layout"

class_init "box"
class_add_property "box" "id"
class_add_property "box" "path"
class_add_property "box" "parent_id"
class_add_property "box" "parent_path"
class_add_property "box" "title"
class_add_property "box" "interval"
class_add_property "box" "output"
class_add_property "box" "x1"
class_add_property "box" "y1"
class_add_property "box" "x2"
class_add_property "box" "y2"
class_add_property "box" "abs_x1"
class_add_property "box" "abs_y1"
class_add_property "box" "abs_x2"
class_add_property "box" "abs_y2"
class_add_property "box" "fill"
class_add_property "box" "fill_color"
class_add_property "box" "fill_char"
class_add_property "box" "border_color"
class_add_property "box" "title_color"
class_add_property "box" "text_color"
class_add_property "box" "output"

# id_path_map_name=""${prefix}"id_path_map"
# map_init """"$id_path_map_name""""
# id_interval_map_name=""${prefix}"id_interval_map"
# map_init """""""$id_interval_map_name"""""""
# id_output_map_name=""${prefix}"id_output_map"
# map_init """"$id_output_map_name""""
# id_absolute_position_map_name=""${prefix}"id_absolute_position_map"
# map_init """"$id_absolute_position_map_name""""

menu_stack_name=""${prefix}"menu_stack"
stack_init """""""""$menu_stack_name"""""""""

change_color() {
    local color=$1
    printf """""""$color"""""""
}

setup_terminal() {
    # Setup the terminal for the TUI.
    # '\e[?1049h': Use alternative screen buffer.
    # '\e[?7l':    Disable line wrapping.
    # '\e[?25l':   Hide the cursor.
    # '\e[2J':     Clear the screen.
    # '\e[1;Nr':   Limit scrolling to scrolling area.
    #              Also sets cursor to (0,0).
    printf '\e[?1049h\e[?7l\e[?25l\e[2J\e[1;%sr' ""$max_items""

    # Hide echoing of user input
    stty -echo
}

get_term_size() {
    # Get terminal size ('stty' is POSIX and always available).
    # This can't be done reliably across all bash versions in pure bash.
    read -r LINES COLUMNS < <(stty size)

    # Max list items that fit in the scroll area.
    ((max_items = LINES - 3))
}

get_ls_colors() {
    # Parse the LS_COLORS variable and declare each file type
    # as a separate variable.
    # Format: ':.ext=0;0:*.jpg=0;0;0:*png=0;0;0;0:'
    [[ -z $LS_COLORS ]] && {
        FFF_LS_COLORS=0
        return
    }

    # Turn $LS_COLORS into an array.
    IFS=: read -ra ls_cols <<<"""""""$LS_COLORS"""""""

    for ((i = 0; i < ${#ls_cols[@]}; i++)); do
        # Separate patterns from file types.
        [[ ${ls_cols[i]} =~ ^\*[^\.] ]] &&
            ls_patterns+=""${ls_cols[i]/=*/}"|"

        # Prepend 'ls_' to all LS_COLORS items
        # if they aren't types of files (symbolic links, block files etc.)
        [[ ${ls_cols[i]} =~ ^(\*|\.) ]] && {
            ls_cols[i]=${ls_cols[i]#\*}
            ls_cols[i]=ls_${ls_cols[i]#.}
        }
    done

    # Strip non-ascii characters from the string as they're
    # used as a key to color the dir items and variable
    # names in bash must be '[a-zA-z0-9_]'.
    ls_cols=("${ls_cols[@]//[^a-zA-Z0-9=\\;]/_}")

    # Store the patterns in a '|' separated string
    # for use in a REGEX match later.
    ls_patterns=${ls_patterns//\*/}
    ls_patterns=${ls_patterns%?}

    # Define the ls_ variables.
    # 'declare' can't be used here as variables are scoped
    # locally. 'declare -g' is not available in 'bash 3'.
    # 'export' is a viable alternative.
    export "${ls_cols[@]}" &>/dev/null
}

setup_options() {
    # Some options require some setup.
    # This function is called once on open to parse
    # select options so the operation isn't repeated
    # multiple times in the code.

    # Format for normal files.
    [[ $FFF_FILE_FORMAT == *%f* ]] && {
        file_pre=${FFF_FILE_FORMAT/'%f'*/}
        file_post=${FFF_FILE_FORMAT/*'%f'/}
    }

    # Format for marked files.
    # Use affixes provided by the user or use defaults, if necessary.
    if [[ $FFF_MARK_FORMAT == *%f* ]]; then
        mark_pre=${FFF_MARK_FORMAT/'%f'*/}
        mark_post=${FFF_MARK_FORMAT/*'%f'/}
    else
        mark_pre=" "
        mark_post="*"
    fi

    # Find supported 'file' arguments.
    file -I &>/dev/null || : ""${file_flags:=biL}""
}

get_os() {
    # Figure out the current operating system to set some specific variables.
    # '$OSTYPE' typically stores the name of the OS kernel.
    case $OSTYPE in
    # Mac OS X / macOS.
    darwin*)
        opener=open
        file_flags=bIL
        ;;

    haiku)
        opener=open

        [[ -z $FFF_TRASH_CMD ]] &&
            FFF_TRASH_CMD=trash

        [[ $FFF_TRASH_CMD == trash ]] && {
            FFF_TRASH=$(finddir -v "$PWD" B_TRASH_DIRECTORY)
            mkdir -p """""""""$FFF_TRASH"""""""""
        }
        ;;
    esac
}

reset_terminal() {
    # Reset the terminal to a useable state (undo all changes).
    # '\e[?7h':   Re-enable line wrapping.
    # '\e[?25h':  Unhide the cursor.
    # '\e[2J':    Clear the terminal.
    # '\e[;r':    Set the scroll region to its default value.
    #             Also sets cursor to (0,0).
    # '\e[?1049l: Restore main screen buffer.
    printf '\e[?7h\e[?25h\e[2J\e[;r\e[?1049l'

    # Show user input.
    stty echo
}

# clear_screen() {
#     # Only clear the scrolling window (dir item list).
#     # '\e[%sH':    Move cursor to bottom of scroll area.
#     # '\e[9999C':  Move cursor to right edge of the terminal.
#     # '\e[1J':     Clear screen to top left corner (from cursor up).
#     # '\e[2J':     Clear screen fully (if using tmux) (fixes clear issues).
#     # '\e[1;%sr':  Clearing the screen resets the scroll region(?). Re-set it.
#     #              Also sets cursor to (0,0).
#     printf '\e[%sH\e[9999C\e[1J%b\e[1;%sr' \
#         "$((LINES - 2))" "${TMUX:+\e[2J}" ""$max_items""
# }

# Function to clear the screen
clear_screen() {
    printf "\033[2J"
}

screen_height() {
    tput lines
}

screen_width() {
    tput cols
}

# Function to move the cursor to a specific location (row, col)
move_cursor() {
    printf "\033[%d;%dH" """""""$1""""""" """""""$2"""""""
}

print_at() {
    local y=$1
    local x=$2
    local text=$3

    move_cursor """""""$y""""""" """""""$x"""""""

    printf "%s" """""$text"""""
}

print_with_color_at() {
    local y=$1
    local x=$2
    local text=$3
    local color=${4:-"black"}

    ansi_color=$(get_color "$color")

    move_cursor """""$y""""" """""$x"""""
    change_color """""$ansi_color"""""

    printf "%s" """""$text"""""
}

vertical_line() {
    local x=$1
    local y1=$2
    local y2=$3
    local color=$4
    local character=${5:-"$VER_LINE_CHAR"}

    [ -n """""$color""""" ] && change_color """""$color"""""

    for ((i = y1; i <= y2; i++)); do
        move_cursor ""$i"" """""$x"""""
        printf "%s" """""$character"""""
    done
}

horizontal_line() {
    local y=$1
    local x1=$2
    local x2=$3
    local color=$4
    local character=${5:-"$HOR_LINE_CHAR"}

    [ -n """""$color""""" ] && change_color """""$color"""""
    for ((i = x1; i <= x2; i++)); do
        move_cursor """""$y""""" ""$i""
        printf "%s" """""$character"""""
    done
}

box() {
    local x1=$1
    local y1=$2
    local x2=$3
    local y2=$4
    local color=${5:-"white"}

    # echo "Drawing box at ($x1, $y1) to ($x2, $y2) with color $color"

    ansi_color=$(get_color "$color")

    change_color "$ansi_color"

    # Draw vertical lines
    vertical_line "$x1" "$((y1 + 1))" "$((y2 - 1))"
    vertical_line "$x2" "$((y1 + 1))" "$((y2 - 1))"

    # Draw horizontal lines
    horizontal_line "$y1" "$((x1 + 1))" "$((x2 - 1))"
    horizontal_line "$y2" "$((x1 + 1))" "$((x2 - 1))"

    # Draw corners
    print_at "$y1" "$x1" "$TOP_LEFT_CHAR"
    print_at "$y1" "$x2" "$TOP_RIGHT_CHAR"
    print_at "$y2" "$x1" "$BOTTOM_LEFT_CHAR"
    print_at "$y2" "$x2" "$BOTTOM_RIGHT_CHAR"

    change_color "$RESET"
}

get_parent_box_path() {
    local path=$1

    # count the number of times the separator appears in the path
    local count=$(grep -o "$separator" <<<"$path" | wc -l)
    if [ "$count" -eq 0 ]; then
        echo "Path '$path' is not a valid box path" >&2
        return 1
    fi

    # If the path has only one separator, then it is the root element
    if [ "$count" -eq 1 ]; then
        echo "$path"
        return 0
    fi

    if [ "$count" -eq 2 ]; then
        get_root_element_name
        return 0
    else
        echo """${path%___children___*}"""
        return 0
    fi
}

get_color() {
    local color=$1

    case $color in
    red)
        echo "$RED"
        ;;
    green)
        echo "$GREEN"
        ;;
    yellow)
        echo "$YELLOW"
        ;;
    blue)
        echo "$BLUE"
        ;;
    magenta)
        echo "$MAGENTA"
        ;;
    cyan)
        echo "$CYAN"
        ;;
    white)
        echo "$WHITE"
        ;;
    black)
        echo "$BLACK"
        ;;
    *)
        echo "$color"
        ;;
    esac

}

fill_box() {
    local x1=$1
    local y1=$2
    local x2=$3
    local y2=$4
    local fill_color=${5:-"red"}
    local fill_char=${6:-"█"}

    local i j

    ansi_fill_color=$(get_color "$fill_color")
    change_color "$ansi_fill_color"

    for ((i = y1; i <= y2; i++)); do
        for ((j = x1; j <= x2; j++)); do
            move_cursor "$i" "$j"
            printf "%s" "$fill_char"
        done
    done

    change_color "$RESET"
}

redraw() {
    local box_id="$1"

    if [ -z "$box_id" ]; then
        clear_screen
        draw_boxes "$(get_root_element_name)" 0 0 "$(screen_width)" "$(screen_height)"
    else
        local box
        box=$(class_get_by_property "box" "id" "$box_id")

        if [ -z "$box" ]; then
            echo "Error: Box with ID '$box_id' not found" >&2
            return 1
        fi

        local box_path
        box_path=$(instance_get_property "$box" "path")

        local parent_box_id
        parent_box_id=$(instance_get_property "$box" "parent_id")

        local parent_box
        parent_box=$(class_get_by_property "box" "id" "$parent_box_id")

        local parent_x1
        local parent_y1
        local parent_x2
        local parent_y2
        parent_x1=$(instance_get_property "$parent_box" "abs_x1")
        parent_y1=$(instance_get_property "$parent_box" "abs_y1")
        parent_x2=$(instance_get_property "$parent_box" "abs_x2")
        parent_y2=$(instance_get_property "$parent_box" "abs_y2")

        local box_absolute_x1
        local box_absolute_y1
        local box_absolute_x2
        local box_absolute_y2
        IFS=' ' read -r box_absolute_x1 box_absolute_y1 box_absolute_x2 box_absolute_y2 <<<"$(calculate_absolute_position "$box_path" "$parent_x1" "$parent_y1" "$parent_x2" "$parent_y2")"

        draw_box "$box_id" "$box_path" "$box_absolute_x1" "$box_absolute_y1" "$box_absolute_x2" "$box_absolute_y2"
    fi
}

draw_box() {
    local box_id="$1"
    local box_path="$2"
    local absolute_x1="$3"
    local absolute_y1="$4"
    local absolute_x2="$5"
    local absolute_y2="$6"

    local box
    box=$(class_get_by_property "box" "id" "$box_id")

    local fill
    fill=$(instance_get_property "$box" "fill")

    if [ -n "$fill" ]; then
        local fill_color
        local fill_char
        fill_color=$(instance_get_property "$box" "fill_color")
        fill_char=$(instance_get_property "$box" "fill_char")
        fill_box "$absolute_x1" "$absolute_y1" "$absolute_x2" "$absolute_y2" "$fill_color" "$fill_char"
    fi

    local border_color
    border_color=$(instance_get_property "$box" "border_color")
    box "$absolute_x1" "$absolute_y1" "$absolute_x2" "$absolute_y2" "$border_color"

    local title
    title=$(instance_get_property "$box" "title")

    if [ -n "$title" ]; then
        local title_color
        title_color=$(instance_get_property "$box" "title_color")
        print_with_color_at "$((absolute_y1 + 1))" "$((absolute_x1 + 1))" "$title" "$title_color"
    fi

    local output
    output=$(instance_get_property "$box" "output")

    if [ -n "$output" ]; then
        local text_color
        text_color=$(instance_get_property "$box" "text_color")
        print_with_color_at "$((absolute_y1 + 3))" "$((absolute_x1 + 3))" "$output" "$text_color"
    fi
}

calculate_absolute_position() {
    local box_path="$1"
    local parent_x1="$2"
    local parent_y1="$3"
    local parent_x2="$4"
    local parent_y2="$5"

    if [ "$box_path" == "$(get_root_element_name)" ]; then
        echo "0 0 $(screen_width) $(screen_height)"
    else
        local box_id
        box_id=$(eval "echo \${${box_path}___id}")
        local box_x1
        local box_y1
        local box_x2
        local box_y2
        box_x1=$(instance_get_property "$box_id" "x1")
        box_y1=$(instance_get_property "$box_id" "y1")
        box_x2=$(instance_get_property "$box_id" "x2")
        box_y2=$(instance_get_property "$box_id" "y2")

        local abs_x1=$((parent_x1 + box_x1))
        local abs_y1=$((parent_y1 + box_y1))
        local abs_x2=$((parent_x2 + box_x2))
        local abs_y2=$((parent_y2 + box_y2))

        echo "$abs_x1 $abs_y1 $abs_x2 $abs_y2"
    fi
}

draw_boxes() {
    local box_path=$1

    local parent_x1=$2
    local parent_y1=$3
    local parent_x2=$4
    local parent_y2=$5

    local box_id=$(eval "echo \${${box_path}___id}")

    local box_absolute_x1 box_absolute_y1 box_absolute_x2 box_absolute_y2
    IFS=' ' read -r box_absolute_x1 box_absolute_y1 box_absolute_x2 box_absolute_y2 <<<"$(calculate_absolute_position "$box_path" "$parent_x1" "$parent_y1" "$parent_x2" "$parent_y2")"

    draw_box "$box_id" "$box_path" "$box_absolute_x1" "$box_absolute_y1" "$box_absolute_x2" "$box_absolute_y2"

    local j=1
    local current_id_path="${box_path}___children___${j}___id"

    # Ensure that the loop checks correctly if the current_id_path variable exists
    while eval "[[ -n \${${current_id_path}} ]]"; do
        local child_box_id="${!current_id_path}"
        local child_box_path="${box_path}___children___${j}"

        # local position_x1=$(eval "echo \${${child_box_path}___position___x1} | tr -d '%'")
        # local position_y1=$(eval "echo \${${child_box_path}___position___y1} | tr -d '%'")
        # local position_x2=$(eval "echo \${${child_box_path}___position___x2} | tr -d '%'")
        # local position_y2=$(eval "echo \${${child_box_path}___position___y2} | tr -d '%'")

        # Calculate absolute position using bash arithmetic and ensure values are integers
        # local absolute_x1=$((parent_x1 + (parent_x2 - parent_x1) * position_x1 / 100))
        # local absolute_y1=$((parent_y1 + (parent_y2 - parent_y1) * position_y1 / 100))
        # local absolute_x2=$((parent_x1 + (parent_x2 - parent_x1) * position_x2 / 100))
        # local absolute_y2=$((parent_y1 + (parent_y2 - parent_y1) * position_y2 / 100))

        # local box_absolute_x1=$(eval "echo \${${box_path}___position___abs_x1}")
        # local box_absolute_y1=$(eval "echo \${${box_path}___position___abs_y1}")
        # local box_absolute_x2=$(eval "echo \${${box_path}___position___abs_x2}")
        # local box_absolute_y2=$(eval "echo \${${box_path}___position___abs_y2}")

        draw_boxes "$child_box_path" "$box_absolute_x1" "$box_absolute_y1" "$box_absolute_x2" "$box_absolute_y2"
        # Increment to the next child
        ((j++))
        current_id_path="${box_path}___children___${j}___id"
    done
}

get_root_element_name() {
    echo "$prefix$root_element_name"
}

default_refresh_interval=1

setup_box_instances() {
    local parent_path=$1

    local root_path=$(get_root_element_name)

    local root_refresh_interval=$(eval "echo \${${root_path}___refresh_interval}")

    if [ -z "$root_refresh_interval" ]; then
        root_refresh_interval=$default_refresh_interval
    fi

    local j=1
    local current_id_path="${parent_path}___children___${j}___id"
    local parent_id=$(eval "echo \${${parent_path}___id}")

    # Ensure that the loop checks correctly if the current_id_path variable exists
    while eval "[[ -n \${${current_id_path}} ]]"; do
        local child_id="${!current_id_path}"

        local child_path="${parent_path}___children___${j}"
        local child_refresh_interval=$(eval "echo \${${child_path}___refresh_interval}")

        box_instance=$(class_create_instance "box")
        instance_set_property "$box_instance" "id" "$child_id"
        instance_set_property "$box_instance" "path" "$child_path"
        instance_set_property "$box_instance" "parent_id" "$parent_id"
        instance_set_property "$box_instance" "parent_path" "$parent_path"
        instance_set_property "$box_instance" "interval" "${child_refresh_interval:-$root_refresh_interval}"
        instance_set_property "$box_instance" "x1" "$(eval "echo \${${child_path}___position___x1}")"
        instance_set_property "$box_instance" "y1" "$(eval "echo \${${child_path}___position___y1}")"
        instance_set_property "$box_instance" "x2" "$(eval "echo \${${child_path}___position___x2}")"
        instance_set_property "$box_instance" "y2" "$(eval "echo \${${child_path}___position___y2}")"

        if [ -z "$child_id" ]; then
            echo "Error: ID not found for path '$child_path'"
            return 1
        fi

        # existing_path=$(map_get "$id_path_map_name" "${child_id}")
        # if [ -n "$existing_path" ]; then
        #     echo "Error: Duplicate ID detected: '${child_id}' already mapped to '$existing_path'"
        #     return 1
        # fi

        # Recursively draw its children
        setup_box_instances "$child_path"

        # Increment to the next child
        ((j++))
        current_id_path="${parent_path}___children___${j}___id"
    done
}

trigger_event() {
    local base_path=$1
    local event=${2:-enter}
    local i=1
    local event_path="${base_path}___on_${event}"

    if eval "[[ -n \${${event_path}} ]]"; then
        return 0
    fi

    local commands=()
    local j=1
    local current_command="${event_path}___${j}"
    while eval "[[ -n \${${current_command}} ]]"; do
        commands+=("$(eval "echo \${${current_command}}")")
        ((j++))
        current_command="${event_path}___${j}"
    done

    output=""
    for command in "${commands[@]}"; do
        # echo "Executing: $command" >>./output.txt
        output+=$(eval "$command")
    done

    echo "$output"
}

refresh_cycle() {
    instances=$(class_list_instances "box")
    local total_keys=${#instances[@]}

    local cycle_start_time=$(date +%s)
    local current_time=0
    local next_event_time=0
    local next_event_index=0

    redraw

    local instance_ids=()
    local next_event_times=()

    while true; do
        read -t 1 -n 1 key && handle_key "$key"
        local current_time=$(date +%s)
        local elapsed_time=$((current_time - cycle_start_time))

        # Refresh instance list if needed
        instances=$(class_list_instances "box") # Refresh list to capture any runtime changes
        total_keys=${#instances[@]}

        # Schedule next events for all instances
        for instance in $instances; do
            echo "configuring instance: $instance" >>./output.txt
            local box_id=$(instance_get_property "$instance" "id")
            local interval=$(instance_get_property "$instance" "interval")
            echo "Box ID: $box_id, Interval: $interval"
            next_event_time=$((current_time + interval))
            instance_ids+=("$box_id")
            next_event_times+=("$next_event_time")
        done
        echo "current_time: $current_time" >>./output.txt
        echo "next_event_times:" >>./output.txt
        echo "${next_event_times[@]}" >>./output.txt
        echo "instance_ids:" >>./output.txt
        echo "${instance_ids[@]}" >>./output.txt
        # Process events that are due
        for ((i = 0; i < total_keys; i++)); do
            if [ "${next_event_times[i]}" -le "$current_time" ]; then
                local box_id="${instance_ids[i]}"
                local instance=$(instance_get_by_property "box" "id" "$box_id")
                local box_path=$(instance_get_property "$instance" "path")
                local output=$(trigger_event "$box_path" "refresh")

                if [ -n "$output" ]; then
                    instance_set_property "$instance" "output" "$output"
                    redraw "$box_id"
                fi

                # Reschedule the next event for this instance
                next_event_times[i]=$((current_time + interval))
            fi
        done

        # Exit if no terminal is attached
        [[ -t 1 ]] || exit 1
    done
}

# Handler for key input
handle_key() {
    local key=$1
    case "$key" in
    q)
        echo "Exiting..."
        exit 0
        ;;
    *)
        echo "Pressed: $key"
        ;;
    esac
}

main() {
    local yaml_file=$1
    local separator="___"

    # Parse the YAML file into shell variables
    local parsed
    parsed="$(parse_yaml "$yaml_file" "$prefix" "$separator")"
    # echo "$parsed" >>./parsed.txt
    if [ -z "$parsed" ]; then
        echo "Error: Unable to parse YAML file. Check file path and structure."
        exit 1
    fi
    eval "$parsed"

    setup_box_instances "$(get_root_element_name)"

    ((BASH_VERSINFO[0] > 3)) &&
        read_flags=(-t 0.05)

    ((${FFF_LS_COLORS:=1} == 1)) &&
        get_ls_colors

    ((${FFF_HIDDEN:=0} == 1)) &&
        shopt -s dotglob

    # Create the trash and cache directory if they don't exist.
    mkdir -p "${XDG_CACHE_HOME:=${HOME}/.cache}/fff" \
        "${FFF_TRASH:=${XDG_DATA_HOME:=${HOME}/.local/share}/fff/trash}"

    # 'nocaseglob': Glob case insensitively (Used for case insensitive search).
    # 'nullglob':   Don't expand non-matching globs to themselves.
    shopt -s nocaseglob nullglob

    # Trap the exit signal (we need to reset the terminal to a useable state.)
    trap 'reset_terminal' EXIT

    # Trap the window resize signal (handle window resize events).
    trap 'get_term_size; redraw' WINCH

    get_os
    get_term_size
    setup_options
    setup_terminal
    redraw

    # map_print "$id_path_map_name"

    # Calculate sleep intervals for each ID
    # local sleep_intervals=()
    # sleep_intervals=($(calculate_sleep_intervals "$id_interval_map_name"))

    # map_sort_by_value "$id_interval_map_name"
    # map_cascade_subtract "$id_interval_map_name"
    class_sort_by_property "box" "interval"
    class_cascade_subtract_property "box" "interval"

    # map_print "$id_path_map_name" >>./intervals.txt
    # map_print "$id_interval_map_name" >>./intervals.txt
    # echo "${sleep_intervals[@]}"
    # echo "${sleep_intervals[@]}" >>./intervals.txt
    # echo id_interval_map
    # map_print "$id_interval_map_name"

    # sleep 30
    # refresh_cycle "${sleep_intervals[@]}"
    refresh_cycle
    # draw_boxes 0 0 $(($(tput cols))) $(($(tput lines))) "root:0:0:100:100"

    # echo "ID to Path Mapping:"

    # # Example of getting a path using an ID
    # local example_id="opt2"
    # if map_contains_key "$id_path_map_name" "$example_id"; then
    #   echo "Path for '$example_id': $(map_get "$id_path_map_name" "$example_id")"
    # else
    #   echo "ID '$example_id' not found"
    # fi

    # Trigger initial menu events and options
    # trigger_event "${prefix}menu" "enter"
    # enter_path "${prefix}menu"

    # bash 5 and some versions of bash 4 don't allow SIGWINCH to interrupt
    # a 'read' command and instead wait for it to complete. In this case it
    # causes the window to not redraw on resize until the user has pressed
    # a key (causing the read to finish). This sets a read timeout on the
    # affected versions of bash.
    # NOTE: This shouldn't affect idle performance as the loop doesn't do
    # anything until a key is pressed.
    # SEE: https://github.com/dylanaraps/fff/issues/48

    # Vintage infinite loop.

}

main "layout.yaml"
