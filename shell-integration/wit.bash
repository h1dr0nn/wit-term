# Wit terminal integration for Bash
# Source this in .bashrc: [ -n "$WIT_TERM" ] && source /path/to/wit.bash

# Guard against double-sourcing
[[ -n "$__WIT_INTEGRATION_ACTIVE" ]] && return
__WIT_INTEGRATION_ACTIVE=1

# Level 1: Report CWD to terminal via OSC 7
__wit_osc7() {
    local cwd
    cwd=$(pwd)
    printf '\e]7;file://%s%s\a' "${HOSTNAME}" "${cwd}"
}

# Level 3: Report command finished with exit code via OSC 133;D
# Level 1: Report CWD via OSC 7
# Level 2: Report prompt start via OSC 133;A
__wit_precmd() {
    local exit_code=$?
    printf '\e]133;D;%d\a' "$exit_code"
    __wit_osc7
    printf '\e]133;A\a'
}

# Level 2: Report command start (output begins) via OSC 133;C
__wit_preexec() {
    printf '\e]133;C\a'
}

if [[ -n "$WIT_TERM" ]]; then
    PROMPT_COMMAND="__wit_precmd${PROMPT_COMMAND:+;$PROMPT_COMMAND}"

    # Use trap DEBUG for preexec since Bash doesn't have native preexec
    __wit_in_prompt_command=0
    __wit_trap_debug() {
        # Don't fire for PROMPT_COMMAND itself or internal functions
        if [[ "$__wit_in_prompt_command" == 1 ]]; then
            return
        fi
        if [[ "$BASH_COMMAND" == "__wit_"* ]] || \
           [[ "$BASH_COMMAND" == "$PROMPT_COMMAND" ]]; then
            return
        fi
        __wit_preexec
    }
    trap '__wit_trap_debug' DEBUG

    # Wrap PROMPT_COMMAND to set guard
    __wit_wrapped_precmd() {
        __wit_in_prompt_command=1
        __wit_precmd
        __wit_in_prompt_command=0
    }
    PROMPT_COMMAND="__wit_wrapped_precmd${PROMPT_COMMAND:+;${PROMPT_COMMAND/__wit_precmd/}}"

    # Level 2: Append prompt end marker after PS1
    # OSC 133;B marks where the command input starts (after prompt)
    PS1="${PS1}\[\e]133;B\a\]"
fi
