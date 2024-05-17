#!/bin/bash
#DO NOT USE ECHO, INSTEAD USE PRINTF
#MUST STAY COMPATIBLE WITH BASH 3

random_prefix() {
    echo "$(date +%s | sha256sum | base64 | head -c 6)"
}

separator="___"

setup_terminal() {
    ##log_trace "layout_lib.sh: setup_terminal()"
    # Setup the terminal for the TUI.
    # '\e[?1049h': Use alternative screen buffer.
    # '\e[?7l':    Disable line wrapping.
    # '\e[?25l':   Hide the cursor.
    # '\e[2J':     Clear the screen.
    # '\e[1;Nr':   Limit scrolling to scrolling area.
    #              Also sets cursor to (0,0).
    printf '\e[?1049h\e[?7l\e[?25l\e[2J\e[1;%sr' """$max_items"""

    # Hide echoing of user input
    stty -echo
}

get_term_size() {
    ##log_trace "layout_lib.sh: get_term_size()"
    # Get terminal size ('stty' is POSIX and always available).
    # This can't be done reliably across all bash versions in pure bash.
    read -r LINES COLUMNS < <(stty size)

    # Max list items that fit in the scroll area.
    ((max_items = LINES - 3))
}

# get_ls_colors() {
#     ##log_trace "layout_lib.sh: get_ls_colors()"
#     # Parse the LS_COLORS variable and declare each file type
#     # as a separate variable.
#     # Format: ':.ext=0;0:*.jpg=0;0;0:*png=0;0;0;0:'
#     [[ -z $LS_COLORS ]] && {
#         FFF_LS_COLORS=0
#         return
#     }

#     # Turn $LS_COLORS into an array.
#     IFS=: read -ra ls_cols <<<"""""""$LS_COLORS"""""""

#     for ((i = 0; i < ${#ls_cols[@]}; i++)); do
#         # Separate patterns from file types.
#         [[ ${ls_cols[i]} =~ ^\*[^\.] ]] &&
#             ls_patterns+=""${ls_cols[i]/=*/}"|"

#         # Prepend 'ls_' to all LS_COLORS items
#         # if they aren't types of files (symbolic links, block files etc.)
#         [[ ${ls_cols[i]} =~ ^(\*|\.) ]] && {
#             ls_cols[i]=${ls_cols[i]#\*}
#             ls_cols[i]=ls_${ls_cols[i]#.}
#         }
#     done

#     # Strip non-ascii characters from the string as they're
#     # used as a key to color the dir items and variable
#     # names in bash must be '[a-zA-z0-9_]'.
#     ls_cols=("${ls_cols[@]//[^a-zA-Z0-9=\\;]/_}")

#     # Store the patterns in a '|' separated string
#     # for use in a REGEX match later.
#     ls_patterns=${ls_patterns//\*/}
#     ls_patterns=${ls_patterns%?}

#     # Define the ls_ variables.
#     # 'declare' can't be used here as variables are scoped
#     # locally. 'declare -g' is not available in 'bash 3'.
#     # 'export' is a viable alternative.
#     export "${ls_cols[@]}" &>/dev/null
# }

# setup_options() {
#     ##log_trace "layout_lib.sh: setup_options()"
#     # Some options require some setup.
#     # This function is called once on open to parse
#     # select options so the operation isn't repeated
#     # multiple times in the code.

#     # Format for normal files.
#     [[ $FFF_FILE_FORMAT == *%f* ]] && {
#         file_pre=${FFF_FILE_FORMAT/'%f'*/}
#         file_post=${FFF_FILE_FORMAT/*'%f'/}
#     }

#     # Format for marked files.
#     # Use affixes provided by the user or use defaults, if necessary.
#     if [[ $FFF_MARK_FORMAT == *%f* ]]; then
#         mark_pre=${FFF_MARK_FORMAT/'%f'*/}
#         mark_post=${FFF_MARK_FORMAT/*'%f'/}
#     else
#         mark_pre=" "
#         mark_post="*"
#     fi

#     # Find supported 'file' arguments.
#     file -I &>/dev/null || : ""${file_flags:=biL}""
# }

get_os() {
    ##log_trace "layout_lib.sh: get_os()"
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
            mkdir -p """""""""""$FFF_TRASH"""""""""""
        }
        ;;
    esac
}

# Function to clear the screen

get_parent_box_path() {
    ##log_trace "layout_lib.sh: get_parent_box_path(path=$1)"
    local prefix="$1"
    local path=$2

    if [ -z "$prefix" ] || [ -z "$path" ]; then
        log_fatal "Usage: get_parent_box_path <prefix> <path>"
        return 1
    fi

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
        get_root_elem "$prefix"
        return 0
    else
        echo """${path%___children___*}"""
        return 0
    fi
}

redraw_box() {
    local box_instance_id="$1"

    if [ -z "$box_instance_id" ]; then
        log_fatal "Usage: redraw_box <box_instance_id>"
        return 1
    fi

    local is_root=$(instance_get_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_IS_ROOT")

    if [ -n "$is_root" ] && [ "$is_root" == "true" ]; then
        clear_screen
        draw_boxes "$box_instance_id" 0 0 "$(screen_width)" "$(screen_height)"
    else
        local box_path

        local box_parent_id
        box_parent_id=$(get_box_parent_id "$box_instance_id")

        local parent_box_instance_id
        parent_box_instance_id=$(get_box_instance_id "$box_parent_id")

        local parent_abs_x1
        local parent_abs_y1
        local parent_abs_x2
        local parent_abs_y2
        parent_abs_x1=$(get_box_abs_x1 "$parent_box_instance_id")
        parent_abs_y1=$(get_box_abs_y1 "$parent_box_instance_id")
        parent_abs_x2=$(get_box_abs_x2 "$parent_box_instance_id")
        parent_abs_y2=$(get_box_abs_y2 "$parent_box_instance_id")

        local box_absolute_x1
        local box_absolute_y1
        local box_absolute_x2
        local box_absolute_y2
        IFS=' ' read -r box_absolute_x1 box_absolute_y1 box_absolute_x2 box_absolute_y2 <<<"$(calculate_absolute_position "$box_instance_id" "$parent_abs_x1" "$parent_abs_y1" "$parent_abs_x2" "$parent_abs_y2")"

        draw_box "$box_instance_id" "$box_absolute_x1" "$box_absolute_y1" "$box_absolute_x2" "$box_absolute_y2"
    fi
}

draw_box() {
    ##log_trace "layout_lib.sh: draw_box(box_id=$1, box_path=$2, x1=$3, y1=$4, x2=$5, y2=$6)"
    local box_instance_id="$1"
    local absolute_x1="$2"
    local absolute_y1="$3"
    local absolute_x2="$4"
    local absolute_y2="$5"

    if [ -z "$box_instance_id" ] || [ -z "$absolute_x1" ] || [ -z "$absolute_y1" ] || [ -z "$absolute_x2" ] || [ -z "$absolute_y2" ]; then
        log_fatal "Usage: draw_box <box_instance_id> <absolute_x1> <absolute_y1> <absolute_x2> <absolute_y2>"
        return 1
    fi

    local fill
    fill=$(get_box_fill "$box_instance_id")

    if [ -n "$fill" ] && [ "$fill" == "true" ]; then
        local fill_color
        local fill_char
        fill_color=$(get_box_fill_color "$box_instance_id")
        fill_char=$(get_box_fill_char "$box_instance_id")
        fill_box "$absolute_x1" "$absolute_y1" "$absolute_x2" "$absolute_y2" "$fill_color" "$fill_char"
    fi

    local border_color
    border_color=$(get_box_border_color "$box_instance_id")
    box "$absolute_x1" "$absolute_y1" "$absolute_x2" "$absolute_y2" "$border_color"

    local title
    title=$(get_box_title "$box_instance_id")

    if [ -n "$title" ]; then
        local title_color
        title_color=$(get_box_title_color "$box_instance_id")
        print_with_color_at "$((absolute_y1 + 1))" "$((absolute_x1 + 1))" "$title" "$title_color"
    fi

    local output
    output=$(get_box_output "$box_instance_id")

    if [ -n "$output" ]; then
        local text_color
        text_color=$(get_box_text_color "$box_instance_id")
        print_with_color_at "$((absolute_y1 + 3))" "$((absolute_x1 + 3))" "$output" "$text_color"
    fi
}

calculate_absolute_position() {
    ##log_trace "layout_lib.sh: calculate_absolute_position(box_path=$1, parent_x1=$2, parent_y1=$3, parent_x2=$4, parent_y2=$5)"
    local box_instance_id="$1"
    local parent_abs_x1="$2"
    local parent_abs_y1="$3"
    local parent_abs_x2="$4"
    local parent_abs_y2="$5"

    if [ -z "$box_instance_id" ] || [ -z "$parent_abs_x1" ] || [ -z "$parent_abs_y1" ] || [ -z "$parent_abs_x2" ] || [ -z "$parent_abs_y2" ]; then
        log_fatal "Usage: calculate_absolute_position <box_instance_id> <parent_abs_x1> <parent_abs_y1> <parent_abs_x2> <parent_abs_y2>"
        return 1
    fi

    local is_root=$(instance_get_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_IS_ROOT")

    if [ -n "$is_root" ] && [ "$is_root" == "true" ]; then
        echo "0 0 $(screen_width) $(screen_height)"
    else
        local box_x1
        local box_y1
        local box_x2
        local box_y2

        box_x1=$(get_box_x1 "$box_instance_id" | tr -d '%')
        box_y1=$(get_box_y1 "$box_instance_id" | tr -d '%')
        box_x2=$(get_box_x2 "$box_instance_id" | tr -d '%')
        box_y2=$(get_box_y2 "$box_instance_id" | tr -d '%')

        # Calculate absolute position using bash arithmetic and ensure values are integers
        local abs_x1=$((parent_abs_x1 + (parent_abs_x2 - parent_abs_x1) * box_x1 / 100))
        local abs_y1=$((parent_abs_y1 + (parent_abs_y2 - parent_abs_y1) * box_y1 / 100))
        local abs_x2=$((parent_abs_x1 + (parent_abs_x2 - parent_abs_x1) * box_x2 / 100))
        local abs_y2=$((parent_abs_y1 + (parent_abs_y2 - parent_abs_y1) * box_y2 / 100))

        instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_ABS_X1" "$abs_x1"
        instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_ABS_Y1" "$abs_y1"
        instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_ABS_X2" "$abs_x2"
        instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_ABS_Y2" "$abs_y2"

        echo "$abs_x1 $abs_y1 $abs_x2 $abs_y2"
    fi
}

draw_boxes() {
    #log_trace "layout_lib.sh: draw_boxes(box_path=$1, parent_x1=$2, parent_y1=$3, parent_x2=$4, parent_y2=$5)"
    local box_instance_id="$1"
    local parent_abs_x1=$2
    local parent_abs_y1=$3
    local parent_abs_x2=$4
    local parent_abs_y2=$5

    if [ -z "$box_instance_id" ]; then
        log_fatal "Usage: draw_boxes <box_instance_id> [parent_abs_x1] [parent_abs_y1] [parent_abs_x2] [parent_abs_y2]"
        return 1
    fi

    local box_absolute_x1 box_absolute_y1 box_absolute_x2 box_absolute_y2
    IFS=' ' read -r box_absolute_x1 box_absolute_y1 box_absolute_x2 box_absolute_y2 <<<"$(calculate_absolute_position "$box_instance_id" "$parent_abs_x1" "$parent_abs_y1" "$parent_abs_x2" "$parent_abs_y2")"

    draw_box "$box_instance_id" "$box_absolute_x1" "$box_absolute_y1" "$box_absolute_x2" "$box_absolute_y2"
    box_id=$(instance_get_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_ID")

    local childrens_instance_ids=$(instance_list_by_property "$BOX_CLS_ID" "$BOX_PROP_PARENT_ID" "$box_id")
    for child_instance_id in $childrens_instance_ids; do
        draw_boxes "$child_instance_id" "$box_absolute_x1" "$box_absolute_y1" "$box_absolute_x2" "$box_absolute_y2"
    done
}

get_root_elem() {
    local prefix="$1"

    if [ -z "$prefix" ]; then
        log_fatal "Usage: get_root_elem <prefix>"
        return 1
    fi

    echo "$prefix$LAYOUT_ROOT_ELEMENT"
}

trigger_box_event() {
    log_trace "layout_lib.sh: trigger_event(box_instance_id=$1, event=$2)"
    local box_instance_id="$1"
    local event_name=${2:-"enter"}

    if [ -z "$box_instance_id" ]; then
        log_fatal "Usage: trigger_box_event <box_instance_id> <event>"
        return 1
    fi

    event_instance_id=$(instance_get_by_properties "$EVENT_CLS_ID" "$EVENT_PROP_BOX_INSTANCE_ID" "$box_instance_id" "$EVENT_PROP_NAME" "$event_name")

    if [ -z "$event_instance_id" ]; then
        log_debug "Event '$event_name' not found for box instance '$box_instance_id'"
        return 0
    fi

    log_debug "Found event '$event_name' for box instance '$box_instance_id'"

    event_script=$(cache_get_property "$EVENT_CLS_ID" "$event_instance_id" "$EVENT_PROP_SCRIPT")

    log_debug "Executing event script for event '$event_instance_id': '$event_script'"

    output=""

    split_result=$(split_into_array "$event_script" $'____')

    for line in $split_result; do
        log_debug "Executing event script line: $line"
        output+=$(eval "$line")
    done

    echo "$output"
}

event_loop() {
    local layout_instance_id="$1"

    if [ -z "$layout_instance_id" ]; then
        log_fatal "Usage: event_loop <layout_instance_id>"
        return 1
    fi

    root_box_instance_id=$(instance_get_by_properties "$BOX_CLS_ID" "$BOX_PROP_LAYOUT_INSTANCE_ID" "$layout_instance_id" "$BOX_PROP_IS_ROOT" "true")

    if [ -z "$root_box_instance_id" ]; then
        log_fatal "Error: Root box not found for layout '$layout_instance_id'"
        return 1
    fi

    instances=$(instance_list_by_property "$BOX_CLS_ID" "$BOX_PROP_LAYOUT_INSTANCE_ID" "$layout_instance_id")
    IFS=$'\n' read -r -d '' -a instances <<<"$instances"
    local total_keys=${#instances[@]}

    if [ "$total_keys" -eq 0 ]; then
        log_fatal "Error: No boxes found for layout instance '$layout_instance_id'"
        return 1
    fi

    local cycle_start_time=$(date +%s)
    local current_time=0
    local next_event_time=0

    redraw_box "$root_box_instance_id"

    local instance_ids=()
    local next_event_times=()

    log_debug "Starting event loop for layout instance '$layout_instance_id'"

    while true; do
        read -t 1 -n 1 key && handle_key "$key"
        local current_time=$(date +%s)

        # Refresh instance list if needed
        instances=$(instance_list_by_property "$BOX_CLS_ID" "$BOX_PROP_LAYOUT_INSTANCE_ID" "$layout_instance_id")
        IFS=$'\n' read -r -d '' -a instances <<<"$instances"
        log_debug "Found instances: ${instances[*]}"
        total_keys=${#instances[@]}

        # Schedule next events for all instances

        for instance_id in "${instances[@]}"; do
            log_debug "Scheduling next event for instance $instance_id"
            local interval=$(get_box_interval "$instance_id")
            next_event_time=$((current_time + interval))
            instance_ids+=("$instance_id")
            next_event_times+=("$next_event_time")
        done

        # Process events that are due
        for ((i = 0; i < total_keys; i++)); do
            log_debug "Processing event for instance '${instance_ids[i]}'"
            if [ "${next_event_times[i]}" -le "$current_time" ]; then
                local output=$(trigger_box_event "${instance_ids[i]}" "refresh")

                if [ -n "$output" ]; then
                    instance_set_property "$BOX_CLS_ID" "${instance_ids[i]}" "$BOX_PROP_OUTPUT" "$output"
                    redraw_box "${instance_ids[i]}"
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
    ##log_trace "layout_lib.sh: handle_key(key=$1)"
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

load_layout_yaml() {
    prefix="$(random_prefix)$separator"

    local yaml_file="$1"
    local reload=${2:-false}

    if [ -z "$yaml_file" ]; then
        log_fatal "Usage: load_layout_yaml <yaml-file>"
        return 1
    fi

    if [ ! -f "$yaml_file" ]; then
        log_fatal "File not found: $yaml_file"
        return 1
    fi

    local parsed
    parsed="$(parse_yaml "$yaml_file" "$prefix" "$separator")"

    if [ -z "$parsed" ]; then
        log_fatal "Error: Unable to parse YAML file '$yaml_file'. Check file path and structure."
        return 1
    fi
    eval "$parsed"

    root_box_path=$(get_root_elem "$prefix")

    root_box_id=$(eval "echo \${${root_box_path}___id}")

    if [ -z "$root_box_id" ]; then
        log_fatal "Error: Root box id not defined in YAML file '$yaml_file'"
        return 1
    fi

    log_debug "Yaml file '$yaml_file' contains layout with root box id '$root_box_id'"

    layout_instance_id=$(instance_get_by_property "$LAYOUT_CLS_ID" "$LAYOUT_PROP_ID" "$root_box_id")

    if [ -n "$layout_instance_id" ] && [ "$reload" == "false" ]; then
        return 0
    fi

    if [ -n "$layout_instance_id" ]; then
        instance_delete "$LAYOUT_CLS_ID" "$layout_instance_id"
    fi

    layout_instance_id=$(instance_new "$LAYOUT_CLS_ID")

    instance_set_property "$LAYOUT_CLS_ID" "$layout_instance_id" "$LAYOUT_PROP_ID" "$root_box_id"
    instance_set_property "$LAYOUT_CLS_ID" "$layout_instance_id" "$LAYOUT_PROP_FILE_PATH" "$yaml_file"
    instance_set_property "$LAYOUT_CLS_ID" "$layout_instance_id" "$LAYOUT_PROP_PREFIX" "$prefix"

    load_layout "$layout_instance_id" "$prefix" "$root_box_path" "true"

    class_sort_by_property "$BOX_CLS_ID" "$BOX_PROP_INTERVAL"
    class_cascade_subtract_property "$BOX_CLS_ID" "$BOX_PROP_INTERVAL"
}

reload_layout() {
    local layout_instance_id="$1"

    if [ -z "$layout_instance_id" ]; then
        log_fatal "Usage: reload_layout <layout_instance_id>"
        return 1
    fi

    if instance_exists "$LAYOUT_CLS_ID" "$layout_instance_id"; then
        local file_path
        file_path=$(cache_get_property "$LAYOUT_CLS_ID" "$layout_instance_id" "$LAYOUT_PROP_FILE_PATH")

        if [ -z "$file_path" ]; then
            log_fatal "Error: File path not found for layout instance '$layout_instance_id'"
            return 1
        fi

        if [ ! -f "$file_path" ]; then
            log_fatal "Error: YaML file for layout instance '$layout_instance_id' not found at '$file_path'"
            return 1
        fi

        load_layout_yaml "$file_path" "true"
    else
        log_fatal "Error: Layout instance '$layout_instance_id' not found"
        return 1
    fi
}

load_layout() {
    #log_trace "layout_lib.sh: load_layout(box_path=$1)"
    local layout_instance_id="$1"
    local prefix="$2"
    local box_path="$3"
    local is_root=${4:-false}

    if [ -z "$layout_instance_id" ] || [ -z "$prefix" ] || [ -z "$box_path" ]; then
        log_fatal "Usage: load_layout <layout_instance_id> <prefix> <box_path> [is_root]"
        return 1
    fi

    local root_path=$(get_root_elem "$prefix")

    local root_refresh_interval=$(eval "echo \${${root_path}___refresh_interval}")

    if [ -z "$root_refresh_interval" ]; then
        root_refresh_interval=$LAYOUT_DEFAULT_REFRESH_INTERVAL
    fi

    local box_id=$(eval "echo \${${box_path}___id}")

    if [ -z "$box_id" ]; then
        log_fatal "Error: Box id not defined for box path '$box_path'"
        return 1
    fi

    local box_refresh_interval=$(eval "echo \${${box_path}___refresh_interval}")

    instance_delete_by_property "$BOX_CLS_ID" "$BOX_PROP_ID" "$box_id"

    local box_instance_id=$(instance_new "$BOX_CLS_ID")

    instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_ID" "$box_id"
    instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_PATH" "$box_path"
    instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_IS_ROOT" "$is_root"
    instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_LAYOUT_INSTANCE_ID" "$layout_instance_id"
    instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_INTERVAL" "${box_refresh_interval:-$root_refresh_interval}"

    # events

    instance_delete_by_property "$EVENT_CLS_ID" "$EVENT_PROP_BOX_INSTANCE_ID" "$box_instance_id"

    for event_name in $LAYOUT_BOX_EVENTS; do
        local i=1
        local event_path="${box_path}___on_${event_name}"

        if eval "[[ -n \${${event_path}} ]]"; then
            continue
        fi

        local script_lines=()
        local j=1
        local current_line="${event_path}___${j}"
        while eval "[[ -n \${${current_line}} ]]"; do
            script_lines+=("$(eval "echo \${${current_line}}")")
            ((j++))
            current_line="${event_path}___${j}"
        done

        local event_script
        event_script=$(concat_with_separator "____" "${script_lines[@]}")

        if [ -n "$event_script" ]; then
            local event_instance_id=$(instance_new "$EVENT_CLS_ID")

            instance_set_property "$EVENT_CLS_ID" "$event_instance_id" "$EVENT_PROP_NAME" "$event_name"
            instance_set_property "$EVENT_CLS_ID" "$event_instance_id" "$EVENT_PROP_BOX_INSTANCE_ID" "$box_instance_id"
            instance_set_property "$EVENT_CLS_ID" "$event_instance_id" "$EVENT_PROP_SCRIPT" "$event_script"
        fi
    done

    #style
    box_fill=$(eval "echo \${${box_path}___fill}")
    if [ -z "$box_fill" ]; then
        box_fill=$LAYOUT_DEFAULT_BOX_FILL
    fi
    instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_FILL" "$box_fill"
    box_fill_color=$(eval "echo \${${box_path}___fill_color}")
    if [ -z "$box_fill_color" ]; then
        box_fill_color=$LAYOUT_DEFAULT_BOX_FILL_COLOR
    fi
    instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_FILL_COLOR" "$box_fill_color"
    box_fill_char=$(eval "echo \${${box_path}___fill_char}")
    if [ -z "$box_fill_char" ]; then
        box_fill_char=$LAYOUT_DEFAULT_BOX_FILL_CHAR
    fi
    instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_FILL_CHAR" "$box_fill_char"
    box_border_color=$(eval "echo \${${box_path}___border_color}")
    if [ -z "$box_border_color" ]; then
        box_border_color=$LAYOUT_DEFAULT_BOX_BORDER_COLOR
    fi
    instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_BORDER_COLOR" "$box_border_color"
    box_title=$(eval "echo \${${box_path}___title}")
    if [ -z "$box_title" ]; then
        box_title=$box_id
    fi
    instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_TITLE" "$box_title"
    box_title_color=$(eval "echo \${${box_path}___title_color}")
    if [ -z "$box_title_color" ]; then
        box_title_color=$LAYOUT_DEFAULT_BOX_TITLE_COLOR
    fi
    instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_TITLE_COLOR" "$box_title_color"
    box_text_color=$(eval "echo \${${box_path}___text_color}")
    if [ -z "$box_text_color" ]; then
        box_text_color=$LAYOUT_DEFAULT_BOX_TEXT_COLOR
    fi
    instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_TEXT_COLOR" "$box_text_color"

    if [ "$box_path" != "$root_path" ]; then
        local box_parent_path=$(get_parent_box_path "$prefix" "$box_path")
        local box_parent_id=$(eval "echo \${${box_parent_path}___id}")

        instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_PARENT_ID" "$box_parent_id"
        instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_PARENT_PATH" "$box_parent_path"

        instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_X1" "$(eval "echo \${${box_path}___position___x1}")"
        instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_Y1" "$(eval "echo \${${box_path}___position___y1}")"
        instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_X2" "$(eval "echo \${${box_path}___position___x2}")"
        instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_Y2" "$(eval "echo \${${box_path}___position___y2}")"

    else
        instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_X1" "0%"
        instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_Y1" "0%"
        instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_X2" "100%"
        instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_Y2" "100%"

        instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_ABS_X1" "0"
        instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_ABS_Y1" "0"
        instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_ABS_X2" "$(screen_width)"
        instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_ABS_Y2" "$(screen_height)"
    fi

    local j=1
    local current_child_path="${box_path}___children___${j}"
    log_debug "Loading children for box path '$box_path'"
    log_debug "Current child path: $current_child_path"

    # Ensure that the loop checks correctly if the current_id_path variable exists
    while eval "[[ -n \${${current_child_path}___id} ]]"; do
        # local child_id="${!current_id_path}"

        # if [ -z "$child_id" ]; then
        #     echo "Error: ID not found for path '$child_path'"
        #     return 1
        # fi

        # existing_path=$(map_get "$id_path_map_name" "${child_id}")
        # if [ -n "$existing_path" ]; then
        #     echo "Error: Duplicate ID detected: '${child_id}' already mapped to '$existing_path'"
        #     return 1
        # fi

        # Recursively draw its children

        load_layout "$layout_instance_id" "$prefix" "$current_child_path" "false"

        # Increment to the next child
        ((j++))
        current_child_path="${box_path}___children___${j}"
    done
}

start_layout() {
    ##log_trace "layout_lib.sh: start_layout()"
    local layout_id="$1"

    if [ -z "$layout_id" ]; then
        log_fatal "Usage: start_layout <layout-id>"
        return 1
    fi

    layout_instance_id=$(instance_get_by_property "$LAYOUT_CLS_ID" "$LAYOUT_PROP_ID" "$layout_id")

    if [ -z "$layout_instance_id" ]; then
        log_fatal "Error: Layout with ID '$layout_id' not found"
        return 1
    fi

    layout_root_box_id=$(instance_get_by_properties "$BOX_CLS_ID" "$BOX_PROP_LAYOUT_INSTANCE_ID" "$layout_instance_id" "$BOX_PROP_IS_ROOT" "true")

    if [ -z "$layout_root_box_id" ]; then
        log_fatal "Error: Root box not found for layout '$layout_id'"
        return 1
    fi

    # layout_prefix=$(cache_get_property "$LAYOUT_CLS_ID" "$layout_instance_id" "$LAYOUT_PROP_PREFIX")

    # Parse the YAML file into shell variables

    # ((BASH_VERSINFO[0] > 3)) &&
    #     read_flags=(-t 0.05)

    # ((${FFF_LS_COLORS:=1} == 1)) &&
    #     get_ls_colors

    # ((${FFF_HIDDEN:=0} == 1)) &&
    #     shopt -s dotglob

    # Create the trash and cache directory if they don't exist.
    # mkdir -p "${XDG_CACHE_HOME:=${HOME}/.cache}/fff" \
    #     "${FFF_TRASH:=${XDG_DATA_HOME:=${HOME}/.local/share}/fff/trash}"

    # 'nocaseglob': Glob case insensitively (Used for case insensitive search).
    # 'nullglob':   Don't expand non-matching globs to themselves.
    shopt -s nocaseglob nullglob

    # Trap the exit signal (we need to reset the terminal to a useable state.)
    trap 'reset_terminal' EXIT

    # Trap the window resize signal (handle window resize events).
    trap "get_term_size; redraw_box ""$layout_root_box_id""" SIGWINCH

    get_os
    get_term_size
    # setup_options
    setup_terminal

    event_loop "$layout_instance_id"
}

load_layouts() {
    # Convert the space-separated string into an array
    IFS=' ' read -r -a layouts <<<"$@"

    for layout_path in "${layouts[@]}"; do
        log_state "Loading layout from '$layout_path'"
        load_layout_yaml "$layout_path"
    done
}

initialize_layouts() {
    setup_layout_data

    layouts=$(list_directory_contents "$LAYOUTS_DIR")
    log_state "Found layouts: $layouts"
    load_layouts "${layouts[@]}"
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
