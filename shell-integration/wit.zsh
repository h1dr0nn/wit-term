# Wit terminal integration for Zsh
# Source this in .zshrc: [ -n "$WIT_TERM" ] && source /path/to/wit.zsh

__wit_osc7() {
    printf '\e]7;file://%s%s\a' "${HOST}" "${PWD}"
}

__wit_precmd() {
    __wit_osc7
}

if [[ -n "$WIT_TERM" ]]; then
    autoload -Uz add-zsh-hook
    add-zsh-hook precmd __wit_precmd
fi
