# Wit terminal integration for Bash
# Source this in .bashrc: [ -n "$WIT_TERM" ] && source /path/to/wit.bash

# Report CWD to terminal via OSC 7
__wit_osc7() {
    local cwd
    cwd=$(pwd)
    printf '\e]7;file://%s%s\a' "${HOSTNAME}" "${cwd}"
}

# Report command start via OSC 133
__wit_preexec() {
    printf '\e]133;C\a'
}

__wit_precmd() {
    printf '\e]133;D\a'
    __wit_osc7
    printf '\e]133;A\a'
}

if [[ -n "$WIT_TERM" ]]; then
    PROMPT_COMMAND="__wit_precmd${PROMPT_COMMAND:+;$PROMPT_COMMAND}"
fi
