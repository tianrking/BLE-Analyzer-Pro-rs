param(
    [Parameter(Mandatory = $true)]
    [string]$Distro,

    [Parameter(Mandatory = $true)]
    [string[]]$BusIds
)

$ErrorActionPreference = "Stop"

Write-Host "Available WSL distributions:"
wsl.exe -l -v
Write-Host ""
Write-Host "Current usbipd devices:"
usbipd list -u
Write-Host ""

foreach ($busid in $BusIds) {
    Write-Host "Attaching WCH BLE Analyzer MCU $busid to $Distro"
    usbipd attach --wsl $Distro --busid $busid
}

Write-Host ""
usbipd list -u
