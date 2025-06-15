$guid = "0x$(((New-Guid).Guid -replace '-', '').ToUpper())"
Write-Host $guid
Set-Clipboard $guid