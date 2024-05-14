#!/bin/bash

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
    for (( i=0; i<${#input}; i++ )); do
        char="${input:$i:1}"
        char=$(get_char "$char")

        if (( ${#char} % 3 != 0 )); then
            echo "Error: Font data for '$char' is not correctly formatted."
            continue
        fi

        local segment_length=$(( ${#char} / 3 ))

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
    for (( i=0; i<num_lines; i++ )); do
        echo -ne "\033[1A\033[2K"  # Move up and clear line
    done
}

title_marquee() {
    local input="$1"
    local width=${2:-$(console_width)}
    local speed=${3:-0.1}
    local compact=${4:-true}
    local separation=${5:-5}  # Control the separation between repetitions of the marquee
    local prefix=${6:-"[ "}  # Prefix that stays at the beginning
    local suffix=${7:-" ]"}  # Suffix that stays at the end

    local text_width=$((width - ${#prefix} - ${#suffix}))

    # Create a separator string of spaces based on the desired separation
    local separator=$(printf '%*s' $separation '')

    # Create a repeating string that is long enough to fill the width multiple times
    local repeat_factor=$((width / (${#input} + separation) + 2))
    local marquee_text=""
    local i
    for (( i=0; i<repeat_factor; i++ )); do
        marquee_text+="${input}${separator}"  # Add separator between repetitions
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
        sleep $speed
    done
}

marquee() {
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
    local separator=$(printf '%*s' $separation '')

    # Create a repeating string that is long enough to fill the width multiple times
    local repeat_factor=$((width / (${#input} + separation) + 2))
    local marquee_text=""
    local i
    for (( i=0; i<repeat_factor; i++ )); do
        marquee_text+="${input}${separator}"  # Add separator between repetitions
    done

    # The marquee buffer that includes enough text to scroll smoothly
    local buffer="${marquee_text}${marquee_text}"

    # Main loop to move marquee
    local offset=0
    local length=${#buffer}

    local x=0
    while [ $x -lt $iterations ]; do
        # Print a slice of the marquee buffer based on the current offset and width
        if [ $x -gt 0 ] || [ $clear_first == "true" ]; then
            clear_lines 1
        fi

        echo "$prefix${buffer:offset:text_width}$suffix"

        # Update offset for scrolling effect
        buffer="${buffer:1}${buffer:0:1}"

        # Sleep to control the speed of the marquee
        sleep $speed
        x=$((x+1))
    done
}

console_height() {
    tput lines
}
# Function to get the current console width
console_width() {
    tput cols
}

# Initial setup of variables
LAST_RECORDED_WIDTH=$(console_width)
LAST_WIDTH_UPDATE_TIME=$(date +%s)

# Configuration for the threshold in seconds
CONSOLE_WIDTH_CHANGE_DETECTION_THRESHOLD_SECONDS=3

# Function to update and check the console width
update_and_check_console_width() {
    local new_width=$(console_width)
    local current_time=$(date +%s)
    local time_since_last_update=$((current_time - LAST_WIDTH_UPDATE_TIME))

    # Check if the threshold time has passed
    if [ "$time_since_last_update" -ge "$CONSOLE_WIDTH_CHANGE_DETECTION_THRESHOLD_SECONDS" ]; then
        LAST_RECORDED_WIDTH=$new_width
        LAST_WIDTH_UPDATE_TIME=$current_time
        echo "Width baseline updated to $LAST_RECORDED_WIDTH at $LAST_WIDTH_UPDATE_TIME" >&2
    fi

    echo "$LAST_RECORDED_WIDTH"
}

# Function to report if there has been a width change within the threshold
console_width_just_changed() {
    local new_width=$(console_width)
    local last_checked_width=$(update_and_check_console_width)

    local width_difference=$(($new_width - $last_checked_width))

    # Report change if any
    if [ "$width_difference" -ne 0 ]; then
        echo "Width changed by $width_difference columns since last reset." >&2
        echo "$width_difference"
        return 0  # Change detected
    else
        echo "No change detected." >&2
        echo "0"
        return 1  # No change
    fi
}


ceil() {
    local a="$1"
    local b="$2"

    # Perform integer division
    local quotient=$(( a / b ))
    local remainder=$(( a % b ))

    # If there's any remainder, increment quotient to achieve ceiling
    if [ "$remainder" -ne 0 ]; then
        quotient=$(( quotient + 1 ))
    fi

    # Return the result
    echo "$quotient"
}

floor() {
    local a="$1"
    local b="$2"

    # Perform integer division
    local quotient=$(( a / b ))

    # Return the result
    echo "$quotient"
}

lines_to_clear() {
    local change=$(console_width_just_changed)

    echo "change: $change" >&2

    if [ $change -eq 0 ]; then
        echo "1"
    else
        current_width=$(console_width)
        old_width=$(($current_width - $change))
        local reduction=$(ceil $old_width $current_width)
        echo "reduction: $reduction" >&2
        echo "$reduction"
    fi
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

    if [ $percentage -lt 0 ] || [ $percentage -gt 100 ]; then
        echo "Error: Percentage must be between 0 and 100."
        return
    fi

    local filled_length=$((percentage * bar_width / 100))

    # Construct filled and unfilled segments of the progress bar
    local bar_filled=$(printf '%*s' $filled_length '' | tr ' ' "$filled_char")
    local bar_empty=$(printf '%*s' $((bar_width - filled_length)) '' | tr ' ' "$empty_char")

    if [ $percentage -lt 10 ]; then
        suffix="$suffix   %$percentage"
    elif [ $percentage -lt 100 ]; then
        suffix="$suffix  %$percentage"
    else
        suffix="$suffix %$percentage"
    fi

    if [ ! $percentage -eq 0 ]; then
        # clear_lines 1
        # if console_width_just_changed; then
        #     clear_lines 1
        # fi
        reduction_multiple=$(lines_to_clear)
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

marquee_progress_bar_percentage(){
    local percentage=$1

    if [ -z "$percentage" ]; then
        echo "Usage: print_marquee_progress_bar_percentage <percentage>"
        return
    fi

    if [ $percentage -lt 0 ] || [ $percentage -gt 100 ]; then
        echo "Error: Percentage must be between 0 and 100."
        return
    fi

    # create suffix
    local suffix="  ]"
    if [ $percentage -lt 10 ]; then
        suffix="$suffix   %$percentage"
    elif [ $percentage -lt 100 ]; then
        suffix="$suffix  %$percentage"
    else
        suffix="$suffix% $percentage"
    fi

    local clear_first=true
    if [ $percentage -eq 0 ]; then
        clear_first=false
    fi

    print_marquee "--" 40 0.1 2 3 $clear_first "[  " "$suffix"
}

i=0

while [ $i -le 100 ]; do
    progress_bar $i
    i=$((i+1))
    sleep 0.2
done

