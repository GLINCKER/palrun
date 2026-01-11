# Palrun PowerShell Integration
# Add to your $PROFILE: Invoke-Expression (palrun init powershell | Out-String)

function Invoke-PalrunSearch {
    $output = palrun run 2>$null
    if ($output) {
        [Microsoft.PowerShell.PSConsoleReadLine]::RevertLine()
        [Microsoft.PowerShell.PSConsoleReadLine]::Insert($output)
    }
}

Set-PSReadLineKeyHandler -Chord 'Ctrl+p' -ScriptBlock {
    Invoke-PalrunSearch
}

# Alias for quick access
Set-Alias -Name pal -Value palrun
