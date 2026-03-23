# Wit terminal integration for Fish
# Source this in config.fish: if set -q WIT_TERM; source /path/to/wit.fish; end

function __wit_osc7 --on-variable PWD
    printf '\e]7;file://%s%s\a' (hostname) $PWD
end

if set -q WIT_TERM
    __wit_osc7
end
