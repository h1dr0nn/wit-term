# Wit terminal integration for Fish
# Source this in config.fish: if set -q WIT_TERM; source /path/to/wit.fish; end

# Guard against double-sourcing
if set -q __WIT_INTEGRATION_ACTIVE
    exit 0
end
set -g __WIT_INTEGRATION_ACTIVE 1

# Level 1: Report CWD via OSC 7
function __wit_osc7 --on-variable PWD
    printf '\e]7;file://%s%s\a' (hostname) $PWD
end

# Level 2+3: Run before each prompt
function __wit_postexec --on-event fish_postexec
    set -l exit_code $status
    # Level 3: Report command finished with exit code
    printf '\e]133;D;%d\a' $exit_code
end

# Level 2: Run before prompt display
function __wit_prompt --on-event fish_prompt
    # Level 1: Report CWD
    __wit_osc7
    # Level 2: Report prompt start
    printf '\e]133;A\a'
    # Level 2: Report command input start (after prompt)
    printf '\e]133;B\a'
end

# Level 2: Run just before command execution
function __wit_preexec --on-event fish_preexec
    # Report command start (output begins)
    printf '\e]133;C\a'
end

if set -q WIT_TERM
    __wit_osc7
end
