param(
    [Parameter(Mandatory = $true)]
    [string] $Target,

    [Parameter(Mandatory = $true)]
    [string] $PackageName
)

$ErrorActionPreference = "Stop"

$profileDir = Join-Path -Path "target" -ChildPath "$Target\release"
$distDir = "dist"
$root = Join-Path -Path $distDir -ChildPath $PackageName

if (Test-Path $root) {
    Remove-Item -Recurse -Force $root
}

New-Item -ItemType Directory -Force -Path `
    (Join-Path $root "bin"), `
    (Join-Path $root "lib"), `
    (Join-Path $root "include"), `
    (Join-Path $root "python"), `
    (Join-Path $root "examples"), `
    (Join-Path $root "docs") | Out-Null

Copy-Item (Join-Path $profileDir "ble-analyzer-pro.exe") (Join-Path $root "bin")
Copy-Item (Join-Path $profileDir "ble_analyzer_pro.dll") (Join-Path $root "lib")
Copy-Item "include\ble_analyzer_pro.h" (Join-Path $root "include")
Copy-Item "python\ble_analyzer_pro.py" (Join-Path $root "python")
Copy-Item "examples\*.py" (Join-Path $root "examples")
Copy-Item "docs\*.md" (Join-Path $root "docs")
Copy-Item "README.md", "README.zh-CN.md", "LICENSE", "Cargo.toml", "99-wch-ble-analyzer.rules" $root

$libusbCandidates = @(
    "$env:VCPKG_ROOT\installed\x64-windows\bin\libusb-1.0.dll",
    "$env:VCPKG_INSTALLATION_ROOT\installed\x64-windows\bin\libusb-1.0.dll",
    "C:\vcpkg\installed\x64-windows\bin\libusb-1.0.dll"
) | Where-Object { $_ -and (Test-Path $_) }

if ($libusbCandidates.Count -gt 0) {
    Copy-Item $libusbCandidates[0] (Join-Path $root "bin")
}

$archive = Join-Path $distDir "$PackageName.zip"
if (Test-Path $archive) {
    Remove-Item -Force $archive
}
Compress-Archive -Path $root -DestinationPath $archive -Force

$hash = Get-FileHash -Algorithm SHA256 $archive
$hashLine = "{0}  {1}" -f $hash.Hash.ToLowerInvariant(), (Split-Path $archive -Leaf)
Set-Content -Path "$archive.sha256" -Value $hashLine

Write-Host "wrote $archive"
