param(
    [string]$Distro = "Ubuntu-26.04",
    [string[]]$BusIds = @("3-1", "3-3", "3-4")
)

$ErrorActionPreference = "Stop"

foreach ($busid in $BusIds) {
    Write-Host "Attaching WCH BLE Analyzer MCU $busid to $Distro"
    usbipd attach --wsl $Distro --busid $busid
}

usbipd list -u
