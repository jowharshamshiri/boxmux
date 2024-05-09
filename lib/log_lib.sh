log() {
    local msg="$1"
    local level="${2:-$LOG_LEVEL_STATE}"

    if [ -z "$msg" ]; then
        echo "log: missing required argument"
        return 1
    fi

    full_msg=""

    case $level in
        $LOG_LEVEL_TRACE)
            full_msg="$(date +'%Y-%m-%d %H:%M:%S') [TRACE] $msg"
            [[ $LOG_LEVEL -ge $LOG_LEVEL_TRACE ]] && echo -e "$full_msg" >&2
            ;;
        $LOG_LEVEL_DEBUG)
            full_msg="$(date +'%Y-%m-%d %H:%M:%S') [DEBUG] $msg"
            [[ $LOG_LEVEL -ge $LOG_LEVEL_DEBUG ]] && echo -e "$full_msg" >&2
            ;;
        $LOG_LEVEL_STATE)
            full_msg="$(date +'%Y-%m-%d %H:%M:%S') [STATE] $msg"
            [[ $LOG_LEVEL -ge $LOG_LEVEL_STATE ]] && echo -e "$full_msg" >&2
            ;;
        $LOG_LEVEL_ALERT)
            full_msg="$(date +'%Y-%m-%d %H:%M:%S') [ALERT] $msg"
            [[ $LOG_LEVEL -ge $LOG_LEVEL_ALERT ]] && echo -e "$full_msg" >&2
            ;;
        $LOG_LEVEL_ERROR)
            full_msg="$(date +'%Y-%m-%d %H:%M:%S') [ERROR] $msg"
            [[ $LOG_LEVEL -ge $LOG_LEVEL_ERROR ]] && echo -e "$full_msg" >&2
            ;;
        $LOG_LEVEL_FATAL)
            full_msg="$(date +'%Y-%m-%d %H:%M:%S') [FATAL] $msg"
            [[ $LOG_LEVEL -ge $LOG_LEVEL_FATAL ]] && echo -e "$full_msg" >&2
            ;;
        *)
            full_msg="$(date +'%Y-%m-%d %H:%M:%S') [invalid log level: $level] $msg"
            echo -e "$full_msg" >&2
            return 1
            ;;
    esac

    echo -e "$full_msg" >> "$RUN_LOG_FILE"
}

log_trace() {
    if [ -z "$1" ]; then
        echo "log_trace: missing required argument"
        return 1
    fi
    log "$1" $LOG_LEVEL_TRACE
}

log_debug() {
    if [ -z "$1" ]; then
        echo "log_debug: missing required argument"
        return 1
    fi
    log "$1" $LOG_LEVEL_DEBUG
}

log_state() {
    if [ -z "$1" ]; then
        echo "log_state: missing required argument"
        return 1
    fi
    log "$1" $LOG_LEVEL_STATE
}

log_alert() {
    if [ -z "$1" ]; then
        echo "log_alert: missing required argument"
        return 1
    fi
    log "$1" $LOG_LEVEL_ALERT
}

log_error() {
    if [ -z "$1" ]; then
        echo "log_error: missing required argument"
        return 1
    fi
    log "$1" $LOG_LEVEL_ERROR
}

log_fatal() {
    if [ -z "$1" ]; then
        echo "log_fatal: missing required argument"
        return 1
    fi
    log "$1" $LOG_LEVEL_FATAL
}

log_level() {
    local level="$1"

    if [ -z "$level" ]; then
        level_name="invalid"
        case $LOG_LEVEL in
            $LOG_LEVEL_TRACE)
                level_name="trace"
                ;;
            $LOG_LEVEL_DEBUG)
                level_name="debug"
                ;;
            $LOG_LEVEL_STATE)
                level_name="state"
                ;;
            $LOG_LEVEL_ALERT)
                level_name="alert"
                ;;
            $LOG_LEVEL_ERROR)
                level_name="error"
                ;;
            $LOG_LEVEL_FATAL)
                level_name="fatal"
                ;;
            *)
                echo "Invalid log level: $level, resetting to state"
                set_env_var "LOG_LEVEL" "$LOG_LEVEL_STATE"
                return 1
                ;;
        esac

        echo "Current log level: $level_name"

    else

        case $level in
            trace)
                LOG_LEVEL=$LOG_LEVEL_TRACE
                ;;
            debug)
                LOG_LEVEL=$LOG_LEVEL_DEBUG
                ;;
            state)
                LOG_LEVEL=$LOG_LEVEL_STATE
                ;;
            alert)
                LOG_LEVEL=$LOG_LEVEL_ALERT
                ;;
            error)
                LOG_LEVEL=$LOG_LEVEL_ERROR
                ;;
            fatal)
                LOG_LEVEL=$LOG_LEVEL_FATAL
                ;;
            *)
                echo "Invalid log level: $level"
                return 1
                ;;
        esac

        set_env_var "LOG_LEVEL" "$LOG_LEVEL"
    fi
}
