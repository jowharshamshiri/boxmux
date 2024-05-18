#!/usr/bin/env bash

setup_layout_data() {
    ##log_trace "layout_lib.sh: setup_data()"
    if [ -n "$LAYOUT_DATA_INITIALIZED" ] && [ "$LAYOUT_DATA_INITIALIZED" == "true" ]; then
        log_debug "Layout data already initialized."
        return 0
    fi
    # initialize_db

    LAYOUT_CLS_ID=$(class_new "layout")
    set_env_var "LAYOUT_CLS_ID" "$LAYOUT_CLS_ID"

    LAYOUT_PROP_ID=$(class_add_property "$LAYOUT_CLS_ID" "id" "$DATATYPE_ID_TEXT")
    set_env_var "LAYOUT_PROP_ID" "$LAYOUT_PROP_ID"

    LAYOUT_PROP_FILE_PATH=$(class_add_property "$LAYOUT_CLS_ID" "file_path" "$DATATYPE_ID_TEXT")
    set_env_var "LAYOUT_PROP_FILE_PATH" "$LAYOUT_PROP_FILE_PATH"

    LAYOUT_PROP_PREFIX=$(class_add_property "$LAYOUT_CLS_ID" "prefix" "$DATATYPE_ID_TEXT")
    set_env_var "LAYOUT_PROP_PREFIX" "$LAYOUT_PROP_PREFIX"

    BOX_CLS_ID=$(class_new "box")
    set_env_var "BOX_CLS_ID" "$BOX_CLS_ID"

    BOX_PROP_ID=$(class_add_property "$BOX_CLS_ID" "id" "$DATATYPE_ID_TEXT")
    set_env_var "BOX_PROP_ID" "$BOX_PROP_ID"

    BOX_PROP_PATH=$(class_add_property "$BOX_CLS_ID" "path" "$DATATYPE_ID_TEXT")
    set_env_var "BOX_PROP_PATH" "$BOX_PROP_PATH"

    BOX_PROP_LAYOUT_INSTANCE_ID=$(class_add_property "$BOX_CLS_ID" "layout_id" "$DATATYPE_ID_TEXT")
    set_env_var "BOX_PROP_LAYOUT_INSTANCE_ID" "$BOX_PROP_LAYOUT_INSTANCE_ID"

    BOX_PROP_IS_ROOT=$(class_add_property "$BOX_CLS_ID" "is_root" "$DATATYPE_ID_TEXT")
    set_env_var "BOX_PROP_IS_ROOT" "$BOX_PROP_IS_ROOT"
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
    BOX_PROP_TITLE_COLOR=$(class_add_property "$BOX_CLS_ID" "title_color" "$DATATYPE_ID_TEXT")
    set_env_var "BOX_PROP_TITLE_COLOR" "$BOX_PROP_TITLE_COLOR"
    BOX_PROP_OUTPUT=$(class_add_property "$BOX_CLS_ID" "output" "$DATATYPE_ID_TEXT")
    set_env_var "BOX_PROP_OUTPUT" "$BOX_PROP_OUTPUT"
    BOX_PROP_TEXT_COLOR=$(class_add_property "$BOX_CLS_ID" "text_color" "$DATATYPE_ID_TEXT")
    set_env_var "BOX_PROP_TEXT_COLOR" "$BOX_PROP_TEXT_COLOR"

    EVENT_CLS_ID=$(class_new "box_event")
    set_env_var "EVENT_CLS_ID" "$EVENT_CLS_ID"
    EVENT_PROP_NAME=$(class_add_property "$EVENT_CLS_ID" "name" "$DATATYPE_ID_TEXT")
    set_env_var "EVENT_PROP_NAME" "$EVENT_PROP_NAME"
    EVENT_PROP_BOX_INSTANCE_ID=$(class_add_property "$EVENT_CLS_ID" "box_instance_id" "$DATATYPE_ID_TEXT")
    set_env_var "EVENT_PROP_BOX_INSTANCE_ID" "$EVENT_PROP_BOX_INSTANCE_ID"
    EVENT_PROP_SCRIPT=$(class_add_property "$EVENT_CLS_ID" "script" "$DATATYPE_ID_TEXT")
    set_env_var "EVENT_PROP_SCRIPT" "$EVENT_PROP_SCRIPT"

    set_env_var "LAYOUT_DATA_INITIALIZED" "true"
}

get_box_instance_id() {
    ##log_trace "layout_lib.sh: get_box_instance_id(box_id=$1)"
    local box_id="$1"
    instance_get_by_property "$BOX_CLS_ID" "$BOX_PROP_ID" "$box_id"
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
    instance_get_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_TITLE_COLOR"
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
    cache_get_property "$BOX_CLS_ID" "$box_instance_id" "$BOX_PROP_INTERVAL"
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
