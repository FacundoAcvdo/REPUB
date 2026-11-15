param(
    [string]$Version = "v0.1.0"
)

$ErrorActionPreference = "Stop"
$RootDir = (Resolve-Path "$PSScriptRoot\..\..").Path
$DistDir = Join-Path $RootDir "dist"
$StageDir = Join-Path $DistDir "repub-$Version-windows-x86_64"

if (!(Test-Path $DistDir)) { New-Item -ItemType Directory -Path $DistDir | Out-Null }
if (Test-Path $StageDir) { Remove-Item -Recurse -Force $StageDir }
New-Item -ItemType Directory -Path $StageDir | Out-Null

$ExePath = Join-Path $RootDir "target\release\repub.exe"
if (!(Test-Path $ExePath)) {
    Push-Location $RootDir
    cargo build --release --bin repub
    Pop-Location
}

Copy-Item $ExePath (Join-Path $StageDir "repub.exe")
Copy-Item (Join-Path $RootDir "assets\app_icon.ico") (Join-Path $StageDir "repub.ico")
Copy-Item (Join-Path $RootDir "README.md") (Join-Path $StageDir "README.md")

Get-ChildItem (Join-Path $RootDir "target\release") -Filter "*.dll" | ForEach-Object {
    Copy-Item $_.FullName $StageDir
}

$ZipPath = Join-Path $DistDir "repub-$Version-windows-x86_64.zip"
if (Test-Path $ZipPath) { Remove-Item -Force $ZipPath }
Compress-Archive -Path "$StageDir\*" -DestinationPath $ZipPath

Write-Host "Paquete generado: $ZipPath"
