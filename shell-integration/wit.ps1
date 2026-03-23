# Wit terminal integration for PowerShell
# Source this in profile: if ($env:WIT_TERM) { . /path/to/wit.ps1 }

# Guard against double-sourcing
if ($env:__WIT_INTEGRATION_ACTIVE) { return }
$env:__WIT_INTEGRATION_ACTIVE = "1"

# Level 1: Report CWD via OSC 7
function __wit_osc7 {
    $cwd = (Get-Location).Path
    $host_name = [System.Net.Dns]::GetHostName()
    # Convert Windows paths to URI format
    $uri_path = $cwd -replace '\\', '/'
    if ($uri_path -match '^([A-Z]):') {
        $uri_path = '/' + $uri_path
    }
    [Console]::Write("`e]7;file://$host_name$uri_path`a")
}

if ($env:WIT_TERM) {
    # Track last exit code for Level 3
    $global:__wit_last_exit = 0

    # Override prompt for Levels 1, 2, and 3
    $__wit_original_prompt = $function:prompt
    function prompt {
        $exit_code = if ($?) { 0 } else { $LASTEXITCODE }
        if ($null -eq $exit_code) { $exit_code = 0 }

        # Level 3: Report previous command finished with exit code
        [Console]::Write("`e]133;D;$exit_code`a")

        # Level 1: Report CWD
        __wit_osc7

        # Level 2: Report prompt start
        [Console]::Write("`e]133;A`a")

        # Run original prompt
        $result = & $__wit_original_prompt

        # Level 2: Report command input start (after prompt)
        [Console]::Write("`e]133;B`a")

        return $result
    }

    # PowerShell doesn't have native preexec, but we can use PSReadLine
    # to detect when Enter is pressed
    if (Get-Module -Name PSReadLine -ErrorAction SilentlyContinue) {
        Set-PSReadLineKeyHandler -Key Enter -ScriptBlock {
            # Level 2: Report command start (output begins)
            [Console]::Write("`e]133;C`a")
            [Microsoft.PowerShell.PSConsoleReadLine]::AcceptLine()
        }
    }
}
