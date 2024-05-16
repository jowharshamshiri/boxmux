#!/bin/bash
#DO NOT USE ECHO, INSTEAD USE PRINTF
#MUST STAY COMPATIBLE WITH BASH 3

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

setup_data() {
    ##log_trace "layout_lib.sh: setup_data()"
    if [ -n "$LAYOUT_DATA_INITIALIZED" ] && [ "$LAYOUT_DATA_INITIALIZED" == "true" ]; then
        log_debug "Layout data already initialized."
        return 0
    fi
    initialize_db

    BOX_CLS_ID=$(class_new "box")
    set_env_var "BOX_CLS_ID" "$BOX_CLS_ID"

    BOX_PROP_ID=$(class_add_property "$BOX_CLS_ID" "id" "$DATATYPE_ID_TEXT")
    set_env_var "BOX_PROP_ID" "$BOX_PROP_ID"

    BOX_PROP_PATH=$(class_add_property "$BOX_CLS_ID" "path" "$DATATYPE_ID_TEXT")
    set_env_var "BOX_PROP_PATH" "$BOX_PROP_PATH"

    BOX_PROP_PARENT_ID=$(class_add_property "$BOX_CLS_ID" "parent_id" "$DATATYPE_ID_TEXT")
    set_env_var "BOX_PROP_PARENT_ID" "$BOX_PROP_PARENT_ID"
    BOX_PROP_PARENT_PATH=$(class_add_property "$BOX_CLS_ID" "parent_path" "$DATATYPE_ID_TEXT")
    set_env_var "BOX_PROP_PARENT_PATH" "$BOX_PROP_PARENT_PATH"
    BOX_PROP_INTERVAL=$(class_add_property "$BOX_CLS_ID" "interval" "$DATATYPE_ID_INTEGER")
    set_env_var "BOX_PROP_INTERVAL" "$BOX_PROP_INTERVAL"
    BOX_PROP_X1=$(class_add_property "$BOX_CLS_ID" "x1" "$DATATYPE_ID_INTEGER")
    set_env_var "BOX_PROP_X1" "$BOX_PROP_X1"
    BOX_PROP_Y1=$(class_add_property "$BOX_CLS_ID" "y1" "$DATATYPE_ID_INTEGER")
    set_env_var "BOX_PROP_Y1" "$BOX_PROP_Y1"
    BOX_PROP_X2=$(class_add_property "$BOX_CLS_ID" "x2" "$DATATYPE_ID_INTEGER")
    set_env_var "BOX_PROP_X2" "$BOX_PROP_X2"
    BOX_PROP_Y2=$(class_add_property "$BOX_CLS_ID" "y2" "$DATATYPE_ID_INTEGER")
    set_env_var "BOX_PROP_Y2" "$BOX_PROP_Y2"
    BOX_PROP_ABS_X1=$(class_add_property "$BOX_CLS_ID" "abs_x1" "$DATATYPE_ID_INTEGER")
    set_env_var "BOX_PROP_ABS_X1" "$BOX_PROP_ABS_X1"
    BOX_PROP_ABS_Y1=$(class_add_property "$BOX_CLS_ID" "abs_y1" "$DATATYPE_ID_INTEGER")
    set_env_var "BOX_PROP_ABS_Y1" "$BOX_PROP_ABS_Y1"
    BOX_PROP_ABS_X2=$(class_add_property "$BOX_CLS_ID" "abs_x2" "$DATATYPE_ID_INTEGER")
    set_env_var "BOX_PROP_ABS_X2" "$BOX_PROP_ABS_X2"
    BOX_PROP_ABS_Y2=$(class_add_property "$BOX_CLS_ID" "abs_y2" "$DATATYPE_ID_INTEGER")
    set_env_var "BOX_PROP_ABS_Y2" "$BOX_PROP_ABS_Y2"
    BOX_PROP_FILL=$(class_add_property "$BOX_CLS_ID" "fill" "$DATATYPE_ID_BOOLEAN")
    set_env_var "BOX_PROP_FILL" "$BOX_PROP_FILL"
    BOX_PROP_FILL_COLOR=$(class_add_property "$BOX_CLS_ID" "fill_color" "$DATATYPE_ID_TEXT")
    set_env_var "BOX_PROP_FILL_COLOR" "$BOX_PROP_FILL_COLOR"
    BOX_PROP_FILL_CHAR=$(class_add_property "$BOX_CLS_ID" "fill_char" "$DATATYPE_ID_TEXT")
    set_env_var "BOX_PROP_FILL_CHAR" "$BOX_PROP_FILL_CHAR"
    BOX_PROP_BORDER_COLOR=$(class_add_property "$BOX_CLS_ID" "border_color" "$DATATYPE_ID_TEXT")
    set_env_var "BOX_PROP_BORDER_COLOR" "$BOX_PROP_BORDER_COLOR"
    BOX_PROP_TITLE=$(class_add_property "$BOX_CLS_ID" "title" "$DATATYPE_ID_TEXT")
    set_env_var "BOX_PROP_TITLE" "$BOX_PROP_TITLE"
    BOX_PROP_TITLE_color=$(class_add_property "$BOX_CLS_ID" "title_color" "$DATATYPE_ID_TEXT")
    set_env_var "BOX_PROP_TITLE_color" "$BOX_PROP_TITLE_color"
    BOX_PROP_OUTPUT=$(class_add_property "$BOX_CLS_ID" "output" "$DATATYPE_ID_TEXT")
    set_env_var "BOX_PROP_OUTPUT" "$BOX_PROP_OUTPUT"
    BOX_PROP_TEXT_COLOR=$(class_add_property "$BOX_CLS_ID" "text_color" "$DATATYPE_ID_TEXT")
    set_env_var "BOX_PROP_TEXT_COLOR" "$BOX_PROP_TEXT_COLOR"

    set_env_var "LAYOUT_DATA_INITIALIZED" "true"
}

get_box_instance_id() {
    ##log_trace "layout_lib.sh: get_box_instance_id(box_id=$1)"
    local box_id="$1"
    class_get_by_property "$BOX_CLS_ID" "$BOX_PROP_ID" "$box_id"
}

get_box_id() {
    ##log_trace "layout_lib.sh: get_box_id(box_instance_id=$1)"
    local box_instance_id="$1"
    instance_get_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_ID"
}

get_box_fill() {
    local box_instance_id="$1"
    instance_get_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_FILL"
}

get_box_fill_color() {
    local box_instance_id="$1"
    instance_get_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_FILL_COLOR"
}

get_box_fill_char() {
    local box_instance_id="$1"
    instance_get_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_FILL_CHAR"
}

get_box_border_color() {
    local box_instance_id="$1"
    instance_get_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_BORDER_COLOR"
}

get_box_title() {
    local box_instance_id="$1"
    instance_get_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_TITLE"
}

get_box_title_color() {
    local box_instance_id="$1"
    instance_get_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_TITLE_color"
}

get_box_output() {
    local box_instance_id="$1"
    instance_get_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_OUTPUT"
}

get_box_text_color() {
    local box_instance_id="$1"
    instance_get_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_TEXT_COLOR"
}

get_box_x1() {
    local box_instance_id="$1"
    instance_get_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_X1"
}

get_box_y1() {
    local box_instance_id="$1"
    instance_get_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_Y1"
}

get_box_x2() {
    local box_instance_id="$1"
    instance_get_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_X2"
}

get_box_y2() {
    local box_instance_id="$1"
    instance_get_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_Y2"
}

get_box_abs_x1() {
    local box_instance_id="$1"
    instance_get_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_ABS_X1"
}

get_box_abs_y1() {
    local box_instance_id="$1"
    instance_get_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_ABS_Y1"
}

get_box_abs_x2() {
    local box_instance_id="$1"
    instance_get_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_ABS_X2"
}

get_box_abs_y2() {
    local box_instance_id="$1"
    instance_get_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_ABS_Y2"
}

get_box_interval() {
    local box_instance_id="$1"
    instance_get_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_INTERVAL"
}

get_box_parent_id() {
    local box_instance_id="$1"
    instance_get_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_PARENT_ID"
}

get_box_parent_path() {
    local box_instance_id="$1"
    instance_get_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_PARENT_PATH"
}

get_box_path() {
    local box_instance_id="$1"
    instance_get_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_PATH"
}

change_color() {
    ##log_trace "layout_lib.sh: change_color($1)"
    local color=$1
    printf """""""$color"""""""
}

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

get_ls_colors() {
    ##log_trace "layout_lib.sh: get_ls_colors()"
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
    ##log_trace "layout_lib.sh: setup_options()"
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

reset_terminal() {
    ##log_trace "layout_lib.sh: reset_terminal()"
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
    ##log_trace "layout_lib.sh: clear_screen()"
    printf "\033[2J"
}

screen_height() {
    ##log_trace "layout_lib.sh: screen_height()"
    tput lines
}

screen_width() {
    ##log_trace "layout_lib.sh: screen_width()"
    tput cols
}

# Function to move the cursor to a specific location (row, col)
move_cursor() {
    ##log_trace "layout_lib.sh: move_cursor()"
    printf "\033[%d;%dH" """""""""$1""""""""" """""""""$2"""""""""
}

print_at() {
    ##log_trace "layout_lib.sh: print_at(x=$2, y=$1, text=$3)"
    local y=$1
    local x=$2
    local text=$3

    move_cursor """""""""$y""""""""" """""""""$x"""""""""

    printf "%s" """""""$text"""""""
}

print_with_color_at() {
    ##log_trace "layout_lib.sh: print_with_color_at(x=$2, y=$1, text=$3, color=$4)"
    local y=$1
    local x=$2
    local text=$3
    local color=${4:-"black"}

    ansi_color=$(get_color "$color")

    move_cursor """""""$y""""""" """""""$x"""""""
    change_color """""""$ansi_color"""""""

    printf "%s" """""""$text"""""""
}

vertical_line() {
    ##log_trace "layout_lib.sh: vertical_line(x=$1, y1=$2, y2=$3, color=$4)"
    local x=$1
    local y1=$2
    local y2=$3
    local color=$4
    local character=${5:-"$VER_LINE_CHAR"}

    [ -n """""""$color""""""" ] && change_color """""""$color"""""""

    for ((i = y1; i <= y2; i++)); do
        move_cursor ""$i"" """""""$x"""""""
        printf "%s" """""""$character"""""""
    done
}

horizontal_line() {
    ##log_trace "layout_lib.sh: horizontal_line(y=$1, x1=$2, x2=$3, color=$4)"
    local y=$1
    local x1=$2
    local x2=$3
    local color=$4
    local character=${5:-"$HOR_LINE_CHAR"}

    [ -n """""""$color""""""" ] && change_color """""""$color"""""""
    for ((i = x1; i <= x2; i++)); do
        move_cursor """""""$y""""""" ""$i""
        printf "%s" """""""$character"""""""
    done
}

box() {
    ##log_trace "layout_lib.sh: box(x1=$1, y1=$2, x2=$3, y2=$4, color=$5)"
    local x1=$1
    local y1=$2
    local x2=$3
    local y2=$4
    local color=${5:-"white"}

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
    ##log_trace "layout_lib.sh: get_parent_box_path(path=$1)"
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
        get_root_elem
        return 0
    else
        echo """${path%___children___*}"""
        return 0
    fi
}

get_color() {
    ##log_trace "layout_lib.sh: get_color($1)"
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
    ##log_trace "layout_lib.sh: fill_box(x1=$1, y1=$2, x2=$3, y2=$4, color=$5, char=$6)"
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

layout_redraw() {
    ##log_trace "layout_lib.sh: layout_redraw(box_id=$1)"
    local box_id="$1"

    if [ -z "$box_id" ]; then
        clear_screen
        draw_boxes "$(get_root_elem)" 0 0 "$(screen_width)" "$(screen_height)"
    else
        local box_instance_id
        box_instance_id=$(get_box_instance_id "$box_id")

        if [ -z "$BOX_CLS_ID" ]; then
            echo "Error: Box with ID '$box_id' not found" >&2
            return 1
        fi

        local box_path
        box_path=$(get_box_path "$box_instance_id")

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
        IFS=' ' read -r box_absolute_x1 box_absolute_y1 box_absolute_x2 box_absolute_y2 <<<"$(calculate_absolute_position "$box_path" "$parent_abs_x1" "$parent_abs_y1" "$parent_abs_x2" "$parent_abs_y2")"

        draw_box "$box_id" "$box_path" "$box_absolute_x1" "$box_absolute_y1" "$box_absolute_x2" "$box_absolute_y2"
    fi
}

draw_box() {
    ##log_trace "layout_lib.sh: draw_box(box_id=$1, box_path=$2, x1=$3, y1=$4, x2=$5, y2=$6)"
    local box_id="$1"
    local box_path="$2"
    local absolute_x1="$3"
    local absolute_y1="$4"
    local absolute_x2="$5"
    local absolute_y2="$6"

    local box_instance_id
    box_instance_id=$(get_box_instance_id "$box_id")
    #log_trace "layout_lib.sh: draw_box(box_id=$box_id, box_instance_id=$box_instance_id, box_path=$box_path, absolute_x1=$absolute_x1, absolute_y1=$absolute_y1, absolute_x2=$absolute_x2, absolute_y2=$absolute_y2)"

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
    local box_path="$1"
    local parent_abs_x1="$2"
    local parent_abs_y1="$3"
    local parent_abs_x2="$4"
    local parent_abs_y2="$5"

    if [ "$box_path" == "$(get_root_elem)" ]; then
        echo "0 0 $(screen_width) $(screen_height)"
    else
        local box_id
        box_id=$(eval "echo \${${box_path}___id}")
        box_instance_id=$(get_box_instance_id "$box_id")
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
    local box_path=$1

    local parent_abs_x1=$2
    local parent_abs_y1=$3
    local parent_abs_x2=$4
    local parent_abs_y2=$5

    local box_id=$(eval "echo \${${box_path}___id}")

    local box_absolute_x1 box_absolute_y1 box_absolute_x2 box_absolute_y2
    IFS=' ' read -r box_absolute_x1 box_absolute_y1 box_absolute_x2 box_absolute_y2 <<<"$(calculate_absolute_position "$box_path" "$parent_abs_x1" "$parent_abs_y1" "$parent_abs_x2" "$parent_abs_y2")"

    draw_box "$box_id" "$box_path" "$box_absolute_x1" "$box_absolute_y1" "$box_absolute_x2" "$box_absolute_y2"

    local j=1
    local current_id_path="${box_path}___children___${j}___id"

    # Ensure that the loop checks correctly if the current_id_path variable exists
    while eval "[[ -n \${${current_id_path}} ]]"; do
        local child_box_path="${box_path}___children___${j}"

        draw_boxes "$child_box_path" "$box_absolute_x1" "$box_absolute_y1" "$box_absolute_x2" "$box_absolute_y2"
        # Increment to the next child
        ((j++))
        current_id_path="${box_path}___children___${j}___id"
    done
}

get_root_elem() {
    ##log_trace "layout_lib.sh: get_root_elem()"
    echo "$prefix$LAYOUT_ROOT_ELEMENT"
}

default_refresh_interval=1

default_box_fill=false
default_box_fill_color="black"
default_box_fill_char="█"
default_box_border_color="white"
default_box_title_color="yellow"
default_box_text_color="white"

setup_box_instances() {
    #log_trace "layout_lib.sh: setup_box_instances(box_path=$1)"
    local box_path=$1
    local root_path=$(get_root_elem)

    local root_refresh_interval=$(eval "echo \${${root_path}___refresh_interval}")

    if [ -z "$root_refresh_interval" ]; then
        root_refresh_interval=$default_refresh_interval
    fi

    box_id=$(eval "echo \${${box_path}___id}")

    # local child_path="${box_path}___children___${j}"
    local box_refresh_interval=$(eval "echo \${${box_path}___refresh_interval}")

    box_instance_id=$(class_create_instance "$BOX_CLS_ID")
    #log_trace "layout_lib.sh: setup_box_instances(box_path=$box_path, box_instance_id=$box_instance_id)"
    instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_ID" "$box_id"
    #log_trace "layout_lib.sh: setup_box_instances(box_path=$box_path, box_instance_id=$box_instance_id, box_id=$box_id)"
    instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_PATH" "$box_path"
    #log_trace "layout_lib.sh: setup_box_instances(box_path=$box_path, box_instance_id=$box_instance_id, box_path=$box_path)"
    instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_INTERVAL" "${box_refresh_interval:-$root_refresh_interval}"
    #log_trace "layout_lib.sh: setup_box_instances(box_path=$box_path, box_instance_id=$box_instance_id, box_refresh_interval=$box_refresh_interval)"

    box_fill=$(eval "echo \${${box_path}___fill}")
    if [ -z "$box_fill" ]; then
        box_fill=$default_box_fill
    fi
    instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_FILL" "$box_fill"
    box_fill_color=$(eval "echo \${${box_path}___fill_color}")
    if [ -z "$box_fill_color" ]; then
        box_fill_color=$default_box_fill_color
    fi
    instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_FILL_COLOR" "$box_fill_color"
    box_fill_char=$(eval "echo \${${box_path}___fill_char}")
    if [ -z "$box_fill_char" ]; then
        box_fill_char=$default_box_fill_char
    fi
    instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_FILL_CHAR" "$box_fill_char"
    box_border_color=$(eval "echo \${${box_path}___border_color}")
    if [ -z "$box_border_color" ]; then
        box_border_color=$default_box_border_color
    fi
    instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_BORDER_COLOR" "$box_border_color"
    box_title=$(eval "echo \${${box_path}___title}")
    if [ -z "$box_title" ]; then
        box_title=$box_id
    fi
    instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_TITLE" "$box_title"
    box_title_color=$(eval "echo \${${box_path}___title_color}")
    if [ -z "$box_title_color" ]; then
        box_title_color=$default_box_title_color
    fi
    instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_TITLE_color" "$box_title_color"
    box_text_color=$(eval "echo \${${box_path}___text_color}")
    if [ -z "$box_text_color" ]; then
        box_text_color=$default_box_text_color
    fi
    instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_TEXT_COLOR" "$box_text_color"

    if [ "$box_path" != "$root_path" ]; then
        local box_parent_path=$(get_parent_box_path "$box_path")
        local box_parent_id=$(eval "echo \${${box_parent_path}___id}")

        instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_PARENT_ID" "$box_parent_id"
        instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_PARENT_PATH" "$box_parent_path"

        instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_X1" "$(eval "echo \${${box_path}___position___x1}")"
        instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_Y1" "$(eval "echo \${${box_path}___position___y1}")"
        instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_X2" "$(eval "echo \${${box_path}___position___x2}")"
        instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_Y2" "$(eval "echo \${${box_path}___position___y2}")"

    else
        init_var_name="LAYOUT_${box_id}_SAVED"
        init_var_value=${!init_var_name}

        if [ -n "$init_var_value" ] && [ "$init_var_value" == "true" ]; then
            return 0
        fi

        set_env_var "$init_var_name" "true"
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
        setup_box_instances "$current_child_path"

        # Increment to the next child
        ((j++))
        current_child_path="${box_path}___children___${j}"
    done
}

trigger_event() {
    ##log_trace "layout_lib.sh: trigger_event(base_path=$1, event=$2)"
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

set_box_output() {
    ##log_trace "layout_lib.sh: set_box_output(box_id=$1, output=$2)"
    local instance_id="$1"
    local output="$2"
    instance_set_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_OUTPUT" "$output"
}

refresh_cycle() {
    ##log_trace "layout_lib.sh: refresh_cycle()"
    instances=$(class_list_instances "$BOX_CLS_ID")
    local total_keys=${#instances[@]}

    local cycle_start_time=$(date +%s)
    local current_time=0
    local next_event_time=0
    local next_event_index=0

    layout_redraw

    local instance_ids=()
    local next_event_times=()

    while true; do
        read -t 1 -n 1 key && handle_key "$key"
        local current_time=$(date +%s)
        local elapsed_time=$((current_time - cycle_start_time))

        # Refresh instance list if needed
        instances=$(class_list_instances "$BOX_CLS_ID") # Refresh list to capture any runtime changes
        total_keys=${#instances[@]}

        # Schedule next events for all instances
        for instance_id in $instances; do
            local interval=$(get_box_interval "$instance_id")
            next_event_time=$((current_time + interval))
            instance_ids+=("$instance_id")
            next_event_times+=("$next_event_time")
        done

        # Process events that are due
        for ((i = 0; i < total_keys; i++)); do
            if [ "${next_event_times[i]}" -le "$current_time" ]; then

                local box_path=$(get_box_path "${instance_ids[i]}")
                local output=$(trigger_event "$box_path" "refresh")

                if [ -n "$output" ]; then
                    set_box_output "${instance_ids[i]}" "$output"
                    layout_redraw "$box_id"
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
    r)
        layout_redraw
        ;;
    *)
        echo "Pressed: $key"
        ;;
    esac
}

layout() {
    local yaml_file=$1
    local separator="___"

    setup_data

    # Parse the YAML file into shell variables
    local parsed
    parsed="$(parse_yaml "$yaml_file" "$prefix" "$separator")"

    if [ -z "$parsed" ]; then
        echo "Error: Unable to parse YAML file. Check file path and structure."
        exit 1
    fi
    eval "$parsed"

    setup_box_instances "$(get_root_elem)"

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
    trap 'get_term_size; layout_redraw' WINCH

    get_os
    get_term_size
    setup_options
    setup_terminal

    class_sort_by_property "$BOX_CLS_ID" "$BOX_PROP_INTERVAL"
    class_cascade_subtract_property "$BOX_CLS_ID" "$BOX_PROP_INTERVAL"

    refresh_cycle
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
