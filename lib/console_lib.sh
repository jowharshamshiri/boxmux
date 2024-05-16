#!/usr/bin/env bash

char_A=("╔═╗╠═╣╩ ╩")
char_B=("╔╗ ╠╩╗╚═╝")
char_C=("╔═╗║  ╚═╝")
char_D=("╔╦╗ ║║═╩╝")
char_E=("╔═╗║╣ ╚═╝")
char_F=("╔═╗╠╣ ╚  ")
char_G=("╔═╗║ ╦╚═╝")
char_H=("╦ ╦╠═╣╩ ╩")
char_I=(" ╦  ║  ╩ ")
char_J=(" ╦  ║ ╚╝ ")
char_K=("╦╔═╠╩╗╩ ╩")
char_L=("╦  ║  ╩═╝")
char_M=("╔╦╗║║║╩ ╩")
char_N=("╔╗╔║║║╝╚╝")
char_O=("╔═╗║ ║╚═╝")
char_P=("╔═╗╠═╝╩  ")
char_Q=("╔═╗║═╬╚═╝")
char_R=("╦═╗╠╦╝╩╚═")
char_S=("╔═╗╚═╗╚═╝")
char_T=("╔╦╗ ║  ╩ ")
char_U=("╦ ╦║ ║╚═╝")
char_V=("╦  ╦╚╗╔╝ ╚╝ ")
char_W=("╦ ╦║║║╚╩╝")
char_X=("═╗ ╦╔╩╦╝╩ ╚═")
char_Y=("╦ ╦╚╦╝ ╩ ")
char_Z=("╔═╗╔═╝╚═╝")

# Corrected lowercase characters
char_a=("┌─┐├─┤┴ ┴")
char_b=("┌┐ ├┴┐└─┘")
char_c=("┌─┐│  └─┘")
char_d=("┌┬┐ ││─┴┘")
char_e=("┌─┐├┤ └─┘")
char_f=("┌─┐├┤ └  ")
char_g=("┌─┐│ ┬└─┘")
char_h=("┬ ┬├─┤┴ ┴")
char_i=(" ┬  │  ┴ ")
char_j=(" ┬  │ └┘ ")
char_k=("┬┌─├┴┐┴ ┴")
char_l=("┬  │  ┴─┘")
char_m=("┌┬┐│││┴ ┴")
char_n=("┌┐┌│││┘└┘")
char_o=("┌─┐│ │└─┘")
char_p=("┌─┐├─┘┴  ")
char_q=("┌─┐│─┼└─┘")
char_r=("┬─┐├┬┘┴└─")
char_s=("┌─┐└─┐└─┘")
char_t=("┌┬┐ │  ┴ ")
char_u=("┬  ┬└┐┌┘ └┘ ")
char_v=("┬  └┐┌ └┘")
char_w=("┬ ┬│││└┴┘")
char_x=("─┐ ┬┌┴┬┘┴ └─")
char_y=("┬ ┬└┬┘ ┴ ")
char_z=("┌─┐┌─┘└─┘")

# Indexed arrays for special characters
char_space=("         ")
char_exclamation=(" ┬  │  o ")
char_question=("┌─┐ ┌┘ o ")
char_underline=("      ───")
char_dash=("   ───   ")
char_comma=("       ┘ ")
char_period=("       o ")
char_bracket_open=("┌─ │  └─ ")
char_bracket_close=(" ─┐  │ ─┘")
char_at_sign=("┌─┐│└┘└──")
char_dollar_sign=("┌┼┐└┼┐└┼┘")
char_percent_sign=("O┬ ┌┘ ┴O ")
char_power_sign=(" /\      ")
char_ampersand=(" ┬ ┌┼─└┘ ")
char_asterisk=("\│/─ ─/│\\")
char_hashtag=("┼─┼│ │┼─┼")
char_single_quote=(" ┴       ")
char_double_quote=(" ┴┴      ")
char_forward_slash=("  / / /  ")
char_back_slash=("\   \   \\")
char_plus_sign=(" │ -+- | ")
char_equals_sign=("___   ───")
char_colon=("    o  o ")
char_semicolon=("    o  ┘ ")
char_tilde=("   /\/   ")
char_brace_open=(" ┌ <   └ ")
char_brace_close=(" ┐   > ┘ ")
char_pipe=(" |  |  | ")

get_char() {
    case "$1" in
    A) echo "${char_A[*]}" ;;
    B) echo "${char_B[*]}" ;;
    C) echo "${char_C[*]}" ;;
    D) echo "${char_D[*]}" ;;
    E) echo "${char_E[*]}" ;;
    F) echo "${char_F[*]}" ;;
    G) echo "${char_G[*]}" ;;
    H) echo "${char_H[*]}" ;;
    I) echo "${char_I[*]}" ;;
    J) echo "${char_J[*]}" ;;
    K) echo "${char_K[*]}" ;;
    L) echo "${char_L[*]}" ;;
    M) echo "${char_M[*]}" ;;
    N) echo "${char_N[*]}" ;;
    O) echo "${char_O[*]}" ;;
    P) echo "${char_P[*]}" ;;
    Q) echo "${char_Q[*]}" ;;
    R) echo "${char_R[*]}" ;;
    S) echo "${char_S[*]}" ;;
    T) echo "${char_T[*]}" ;;
    U) echo "${char_U[*]}" ;;
    V) echo "${char_V[*]}" ;;
    W) echo "${char_W[*]}" ;;
    X) echo "${char_X[*]}" ;;
    Y) echo "${char_Y[*]}" ;;
    Z) echo "${char_Z[*]}" ;;
    a) echo "${char_a[*]}" ;;
    b) echo "${char_b[*]}" ;;
    c) echo "${char_c[*]}" ;;
    d) echo "${char_d[*]}" ;;
    e) echo "${char_e[*]}" ;;
    f) echo "${char_f[*]}" ;;
    g) echo "${char_g[*]}" ;;
    h) echo "${char_h[*]}" ;;
    i) echo "${char_i[*]}" ;;
    j) echo "${char_j[*]}" ;;
    k) echo "${char_k[*]}" ;;
    l) echo "${char_l[*]}" ;;
    m) echo "${char_m[*]}" ;;
    n) echo "${char_n[*]}" ;;
    o) echo "${char_o[*]}" ;;
    p) echo "${char_p[*]}" ;;
    q) echo "${char_q[*]}" ;;
    r) echo "${char_r[*]}" ;;
    s) echo "${char_s[*]}" ;;
    t) echo "${char_t[*]}" ;;
    u) echo "${char_u[*]}" ;;
    v) echo "${char_v[*]}" ;;
    w) echo "${char_w[*]}" ;;
    x) echo "${char_x[*]}" ;;
    y) echo "${char_y[*]}" ;;
    z) echo "${char_z[*]}" ;;
    " ") echo "${char_space[*]}" ;;
    "!") echo "${char_exclamation[*]}" ;;
    "?") echo "${char_question[*]}" ;;
    "_") echo "${char_underline[*]}" ;;
    "-") echo "${char_dash[*]}" ;;
    ",") echo "${char_comma[*]}" ;;
    ".") echo "${char_period[*]}" ;;
    "[") echo "${char_bracket_open[*]}" ;;
    "]") echo "${char_bracket_close[*]}" ;;
    "@") echo "${char_at_sign[*]}" ;;
    "$") echo "${char_dollar_sign[*]}" ;;
    "%") echo "${char_percent_sign[*]}" ;;
    "^") echo "${char_power_sign[*]}" ;;
    "&") echo "${char_ampersand[*]}" ;;
    "*") echo "${char_asterisk[*]}" ;;
    "#") echo "${char_hashtag[*]}" ;;
    "'") echo "${char_single_quote[*]}" ;;
    "\"") echo "${char_double_quote[*]}" ;;
    "/") echo "${char_forward_slash[*]}" ;;
    "\\") echo "${char_back_slash[*]}" ;;
    "+") echo "${char_plus_sign[*]}" ;;
    "=") echo "${char_equals_sign[*]}" ;;
    ":") echo "${char_colon[*]}" ;;
    ";") echo "${char_semicolon[*]}" ;;
    "~") echo "${char_tilde[*]}" ;;
    "{") echo "${char_brace_open[*]}" ;;
    "}") echo "${char_brace_close[*]}" ;;
    "|") echo "${char_pipe[*]}" ;;
    *) echo -e "\n\n" ;;
    esac
}

is_all_space() {
    local segment="$1"
    [[ "$segment" =~ ^[[:space:]]+$ ]] && return 0 || return 1
}

print_title() {
    local input="$1"
    local compact=${2:-true}
    local line1=""
    local line2=""
    local line3=""

    local i
    for ((i = 0; i < ${#input}; i++)); do
        char="${input:$i:1}"
        char=$(get_char "$char")

        if ((${#char} % 3 != 0)); then
            echo "Error: Font data for '$char' is not correctly formatted."
            continue
        fi

        local segment_length=$((${#char} / 3))

        segment1="${char:0:$segment_length}"
        segment2="${char:$segment_length:$segment_length}"
        segment3="${char:2*$segment_length:$segment_length}"

        if is_all_space "$segment1" && is_all_space "$segment2" && is_all_space "$segment3"; then
            segment1=" "
            segment2=" "
            segment3=" "
        else
            # Trim leading spaces
            while [[ "${segment1:0:1}" == " " && "${segment2:0:1}" == " " && "${segment3:0:1}" == " " ]]; do
                segment1="${segment1:1}"
                segment2="${segment2:1}"
                segment3="${segment3:1}"
            done

            # Find the smallest length of non-space-ending segments to ensure uniform trimming
            local min_length=${#segment1}
            [[ ${#segment2} -lt $min_length ]] && min_length=${#segment2}
            [[ ${#segment3} -lt $min_length ]] && min_length=${#segment3}

            # Trim trailing spaces
            while [[ $min_length -gt 0 && "${segment1:min_length-1:1}" == " " && "${segment2:min_length-1:1}" == " " && "${segment3:min_length-1:1}" == " " ]]; do
                min_length=$((min_length - 1))
            done

            segment1="${segment1:0:min_length}"
            segment2="${segment2:0:min_length}"
            segment3="${segment3:0:min_length}"
        fi

        if [ "$compact" = true ]; then
            line1+="$segment1"
            line2+="$segment2"
            line3+="$segment3"
        else
            line1+=" $segment1"
            line2+=" $segment2"
            line3+=" $segment3"
        fi
    done

    echo -e "$line1\n$line2\n$line3"
}

clear_lines() {
    local num_lines=$1
    local i
    for ((i = 0; i < num_lines; i++)); do
        echo -ne "\033[1A\033[2K" # Move up and clear line
    done
}

title_marquee() {
    local input="$1"
    local width=${2:-$(console_width)}
    local speed=${3:-0.1}
    local compact=${4:-true}
    local separation=${5:-5} # Control the separation between repetitions of the marquee
    local prefix=${6:-"[ "}  # Prefix that stays at the beginning
    local suffix=${7:-" ]"}  # Suffix that stays at the end

    local text_width=$((width - ${#prefix} - ${#suffix}))

    # Create a separator string of spaces based on the desired separation
    local separator=$(printf '%*s' "$separation" '')

    # Create a repeating string that is long enough to fill the width multiple times
    local repeat_factor=$((width / (${#input} + separation) + 2))
    local marquee_text=""
    local i
    for ((i = 0; i < repeat_factor; i++)); do
        marquee_text+="${input}${separator}" # Add separator between repetitions
    done

    # The marquee buffer that includes enough text to scroll smoothly
    local buffer="${marquee_text}${marquee_text}"

    # Main loop to move marquee
    local offset=0
    local length=${#buffer}
    while true; do
        # Print a slice of the marquee buffer based on the current offset and width
        clear_lines 3

        print_title "$prefix${buffer:offset:text_width}$suffix" "$compact"

        # Update offset for scrolling effect
        buffer="${buffer:1}${buffer:0:1}"

        # Sleep to control the speed of the marquee
        sleep "$speed"
    done
}

print_marquee() {
    local input="$1"
    local width=${2:-$(console_width)}
    local speed=${3:-0.1}
    local separation=${4:-5}
    local iterations=${5:-10}
    local clear_first=${6:-true}
    local prefix=${7:-"[ "}
    local suffix=${8:-" ]"}

    local text_width=$((width - ${#prefix} - ${#suffix}))

    # Create a separator string of spaces based on the desired separation
    local separator=$(printf '%*s' "$separation" '')

    # Create a repeating string that is long enough to fill the width multiple times
    local repeat_factor=$((width / (${#input} + separation) + 2))
    local marquee_text=""
    local i
    for ((i = 0; i < repeat_factor; i++)); do
        marquee_text+="${input}${separator}" # Add separator between repetitions
    done

    # The marquee buffer that includes enough text to scroll smoothly
    local buffer="${marquee_text}${marquee_text}"

    # Main loop to move marquee
    local offset=0
    local length=${#buffer}

    local x=0
    while [ $x -lt "$iterations" ]; do
        # Print a slice of the marquee buffer based on the current offset and width
        if [ $x -gt 0 ] || [ "$clear_first" == "true" ]; then
            clear_lines 1
        fi

        echo "$prefix${buffer:offset:text_width}$suffix"

        # Update offset for scrolling effect
        buffer="${buffer:1}${buffer:0:1}"

        # Sleep to control the speed of the marquee
        sleep "$speed"
        x=$((x + 1))
    done
}

console_height() {
    tput lines
}
# Function to get the current console width
console_width() {
    tput cols
}

progress_bar() {
    local percentage=$1
    local width=${2:-$(console_width)}
    local prefix=${3:-"[ "}
    local suffix=${4:-" ]"}
    local empty_char=${5:-"-"}
    local filled_char=${6:-"x"}

    percentage_width=5
    bar_width=$((width - ${#prefix} - ${#suffix} - percentage_width))

    if [ -z "$percentage" ]; then
        echo "Usage: progress_bar <percentage> [width]"
        return
    fi

    if [ "$percentage" -lt 0 ] || [ "$percentage" -gt 100 ]; then
        echo "Error: Percentage must be between 0 and 100."
        return
    fi

    local filled_length=$((percentage * bar_width / 100))

    # Construct filled and unfilled segments of the progress bar
    local bar_filled=$(printf '%*s' $filled_length '' | tr ' ' "$filled_char")
    local bar_empty=$(printf '%*s' $((bar_width - filled_length)) '' | tr ' ' "$empty_char")

    if [ "$percentage" -lt 10 ]; then
        suffix="$suffix   %$percentage"
    elif [ "$percentage" -lt 100 ]; then
        suffix="$suffix  %$percentage"
    else
        suffix="$suffix %$percentage"
    fi

    if [ ! "$percentage" -eq 0 ]; then
        clear_lines 1
        # if console_width_just_changed; then
        #     clear_lines 1
        # fi
        # reduction_multiple=$(lines_to_clear)
        # echo "reduction_multiple: $reduction_multiple"
        # clear_lines $reduction_multiple +1

    else
        clear_lines 1
        # echo "" > /dev/null
        # echo "reduction_multiple: $reduction_multiple"
        # clear_lines $reduction_multiple
        # clear_lines $reduction_multiple
    fi

    # Display the progress bar
    echo -e "$prefix$bar_filled$bar_empty$suffix"
}

marquee_progress_bar_percentage() {
    local percentage=$1

    if [ -z "$percentage" ]; then
        echo "Usage: print_marquee_progress_bar_percentage <percentage>"
        return
    fi

    if [ "$percentage" -lt 0 ] || [ "$percentage" -gt 100 ]; then
        echo "Error: Percentage must be between 0 and 100."
        return
    fi

    # cre"ate suffix"
    local suffix="  ]"
    if [ "$percentage" -lt 10 ]; then
        suffix="$suffix   %$percentage"
    elif [ "$percentage" -lt 100 ]; then
        suffix="$suffix  %$percentage"
    else
        suffix="$suffix% $percentage"
    fi

    local clear_first=true
    if [ "$percentage" -eq 0 ]; then
        clear_first=false
    fi

    print_marquee "--" 40 0.1 2 3 $clear_first "[  " "$suffix"
}

marquee_wait() {
    while true; do
        print_marquee "----" 80 0.1
    done
}

add_wait_marquee() {
    marquee_wait &
    WAIT_MARQUEE_PID=$!
    set_env_var "WAIT_MARQUEE_PID" "$WAIT_MARQUEE_PID"

    trap "kill $WAIT_MARQUEE_PID; exit" INT
}

remove_wait_marquee() {
    if [ -n "$WAIT_MARQUEE_PID" ]; then
        kill "$WAIT_MARQUEE_PID" >/dev/null 2>&1
        unset_env_var "WAIT_MARQUEE_PID"
    fi
}

# Function to print a scrolling marquee while allowing logs to scroll above it
scrolling_marquee() {
    local message="$1"
    local width=${2:-$(console_width)}
    local speed=${3:-0.1}
    local lines_above=${4:-5} # Number of lines allowed to scroll above the marquee

    local message_length=${#message}
    local buffer="${message}$(printf '%*s' "$width" '' | tr ' ' ' ')"

    while true; do
        clear_lines $((lines_above + 1))

        for ((i = 0; i < message_length + width; i++)); do
            echo -ne "\033[1A\033[2K" # Move up and clear line
            echo -e "${buffer:i:width}"
            sleep "$speed"
        done

        # Allow logs to scroll above
        for ((i = 0; i < lines_above; i++)); do
            echo -e "\033[1A" # Move up
        done
    done
}

test() {
    # i=0

    # while [ $i -le 100 ]; do
    #     progress_bar $i
    #     i=$((i + 1))
    #     sleep 0.2
    # done
    message="$1"
    width="${2:-$(console_width)}"
    speed="${3:-0.1}"
    lines_above="${4:-5}"

    scrolling_marquee "$message" "$width" "$speed" "$lines_above"
}

change_color() {
    ##log_trace "layout_lib.sh: change_color($1)"
    local color=$1
    printf """""""$color"""""""
}

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
