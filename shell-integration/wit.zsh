# Wit terminal integration for Zsh
# Source this in .zshrc: [ -n "$WIT_TERM" ] && source /path/to/wit.zsh

# Guard against double-sourcing
[[ -n "$__WIT_INTEGRATION_ACTIVE" ]] && return
__WIT_INTEGRATION_ACTIVE=1

# Level 1: Report CWD via OSC 7
__wit_osc7() {
    printf '\e]7;file://%s%s\a' "${HOST}" "${PWD}"
}

# Level 2+3: precmd — runs before each prompt
__wit_precmd() {
    local exit_code=$?
    # Level 3: Report command finished with exit code
    printf '\e]133;D;%d\a' "$exit_code"
    # Level 1: Report CWD
    __wit_osc7
    # Level 2: Report prompt start
    printf '\e]133;A\a'
}

# Level 2: preexec — runs just before command execution
__wit_preexec() {
    # Report command start (output begins)
    printf '\e]133;C\a'
}

if [[ -n "$WIT_TERM" ]]; then
    autoload -Uz add-zsh-hook
    add-zsh-hook precmd __wit_precmd
    add-zsh-hook preexec __wit_preexec
fi
