#!/usr/bin/env bash

# Initializes a new stack with the given name
stack_init() {
    local stack_name="$1"
    if [ -z "$stack_name" ]; then
        echo "Error: stack name is required" >&2
        return 1
    fi

    #check if the stack is already initialized
    if [ -n "$(eval echo \${"$stack_name"[@]})" ]; then
        echo "Error: stack $stack_name is already initialized" >&2
        return 1
    fi

    eval "${stack_name}=()"
}

# Pushes an item onto the specified stack
stack_push() {
    local stack_name="$1"
    local value="$2"
    if [ -z "$stack_name" ] || [ -z "$value" ]; then
        echo "Error: both stack name and value are required" >&2
        return 1
    fi
    eval "${stack_name}+=(\"$value\")"
}

# Pops an item off the specified stack and returns it
stack_pop() {
    local stack_name="$1"
    if [ -z "$stack_name" ]; then
        echo "Error: stack name is required" >&2
        return 1
    fi

    local stack=()
    local stack_size=0
    eval "stack=(\"\${${stack_name}[@]}\")"
    eval "stack_size=\${#${stack_name}[@]}"
    if [ "$stack_size" -gt 0 ]; then
        local last_index
        local value
        eval "last_index=\$((stack_size - 1))"
        eval "value=\${${stack_name}[$last_index]}"
        eval "unset ${stack_name}[$last_index]"
        eval "${stack_name}=(\"\${${stack_name}[@]}\")"
        echo "$value"
        return 0
    else
        echo "Error: ${stack_name} is empty" >&2
        return 1
    fi
}

# Stack Top: Peek the last element without removing it
stack_top() {
    local stack_name="$1"
    if [ -z "$stack_name" ]; then
        echo "Error: stack name is required" >&2
        return 1
    fi

    local stack_size
    eval "stack_size=\${#${stack_name}[@]}"
    if [ "$stack_size" -gt 0 ]; then
        eval "echo \${${stack_name}[$((stack_size - 1))]}"
    else
        echo "Error: ${stack_name} is empty" >&2
        return 1
    fi
}

# Clears all elements from the specified stack
stack_clear() {
    local stack_name="$1"
    if [ -z "$stack_name" ]; then
        echo "Error: stack name is required" >&2
        return 1
    fi
    eval "${stack_name}=()"
}

# Prints all elements in the specified stack
stack_print() {
    local stack_name="$1"
    if [ -z "$stack_name" ]; then
        echo "Error: stack name is required" >&2
        return 1
    fi

    local stack=()
    eval "stack=(\"\${${stack_name}[@]}\")"
    if [ ${#stack[@]} -gt 0 ]; then
        echo "Contents of ${stack_name}:"
        for item in "${stack[@]}"; do
            echo "$item"
        done
    else
        echo "Error: ${stack_name} is empty" >&2
    fi
}

# Returns the size of the specified stack
stack_size() {
    local stack_name="$1"
    if [ -z "$stack_name" ]; then
        echo "Error: stack name is required" >&2
        return 1
    fi
    eval "echo \${#${stack_name}[@]}"
}

# Checks if the specified stack is empty
stack_is_empty() {
    local stack_name="$1"
    if [ -z "$stack_name" ]; then
        echo "Error: stack name is required" >&2
        return 1
    fi
    eval "[ \${#${stack_name}[@]} -eq 0 ]" && return 0 || return 1
}

stack_duplicate() {
    local stack_name="$1"
    local new_stack_name="$2"
    if [ -z "$stack_name" ] || [ -z "$new_stack_name" ]; then
        echo "Error: both stack name and new stack name are required" >&2
        return 1
    fi

    if [ -n "$(eval echo \${"$new_stack_name"[@]})" ]; then
        echo "Error: stack $new_stack_name is already initialized" >&2
        return 1
    fi

    #check stack_name exists
    if [ -z "$(eval echo \${"$stack_name"[@]})" ]; then
        echo "Error: stack $stack_name is not initialized" >&2
        return 1
    fi

    local stack=()
    eval "stack=(\"\${${stack_name}[@]}\")"
    eval "${new_stack_name}=()"
    for item in "${stack[@]}"; do
        eval "${new_stack_name}+=(\"$item\")"
    done
}
