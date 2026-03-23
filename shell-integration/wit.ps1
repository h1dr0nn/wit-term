# Wit terminal integration for PowerShell
# Source this in profile: if ($env:WIT_TERM) { . /path/to/wit.ps1 }

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
    # Override prompt to include OSC 7
    $__wit_original_prompt = $function:prompt
    function prompt {
        __wit_osc7
        & $__wit_original_prompt
    }
}
