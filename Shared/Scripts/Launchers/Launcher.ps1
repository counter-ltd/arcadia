$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RootDir = Resolve-Path "$ScriptDir/../.."

function Invoke-Arcadia {
    param(
        [Parameter(Mandatory = $true)]
        [bool]$Release,
        [Parameter(Mandatory = $true)]
        [string]$Feature
    )

    $projectRoot = Resolve-Path (Join-Path $RootDir "..")
    $manifestPath = "Desktop/Cargo.toml"

    Push-Location $projectRoot
    try {
        if ($Release) {
            Write-Host ""
            Write-Host "Running: cargo run --manifest-path $manifestPath --target-dir target --release --features $Feature"
            cargo run --manifest-path $manifestPath --target-dir target --release --features $Feature
        }
        else {
            Write-Host ""
            Write-Host "Running: cargo run --manifest-path $manifestPath --target-dir target --features $Feature"
            cargo run --manifest-path $manifestPath --target-dir target --features $Feature
        }
    }
    finally {
        Pop-Location
    }
}

function Invoke-IosDeviceDeploy {
    param(
        [Parameter(Mandatory = $true)]
        [bool]$Release
    )

    $configuration = if ($Release) { "Release" } else { "Debug" }
    $projectPath = Join-Path $RootDir "../Mobile/iOS/ArcadiaApp.xcodeproj"
    $sharedBuildScript = Join-Path $RootDir "Scripts/Builds/build-ios-framework.sh"
    $derivedDataPath = Join-Path (Join-Path $RootDir "..") "build/ios-device"
    $bundleId = "com.stacknode.arcadia"
    $preferredDeviceName = $env:ARCADIA_IOS_DEVICE_NAME
    $destinations = ""
    $deviceUdid = ""

    if (-not (Get-Command xcodebuild -ErrorAction SilentlyContinue)) {
        throw "xcodebuild not found. Install Xcode command line tools."
    }

    if (-not (Test-Path $projectPath)) {
        throw "iOS project not found at $projectPath"
    }

    if (-not (Test-Path $sharedBuildScript)) {
        throw "Shared iOS build script not found at $sharedBuildScript"
    }

    $destinations = & xcodebuild `
        -project $projectPath `
        -scheme "ArcadiaApp" `
        -showdestinations 2>$null

    if (-not [string]::IsNullOrWhiteSpace($preferredDeviceName)) {
        $deviceUdid = $destinations |
            rg "platform:iOS, arch:arm64, id:[^,]+, name:$preferredDeviceName" |
            rg -o "id:[^,]+" |
            ForEach-Object { ($_ -split ":", 2)[1] } |
            Select-Object -First 1
    }

    if ([string]::IsNullOrWhiteSpace($deviceUdid)) {
        $deviceUdid = $destinations |
            rg "platform:iOS, arch:arm64, id:" |
            rg -o "id:[^,]+" |
            ForEach-Object { ($_ -split ":", 2)[1] } |
            Select-Object -First 1
    }

    if ([string]::IsNullOrWhiteSpace($deviceUdid)) {
        throw "No connected physical iOS device found. Hint: set ARCADIA_IOS_DEVICE_NAME to your device name."
    }

    Write-Host ""
    Write-Host "Building shared iOS artifacts..."
    & bash $sharedBuildScript
    if ($LASTEXITCODE -ne 0) {
        throw "Shared iOS artifact build failed."
    }

    Push-Location (Join-Path $RootDir "..")
    try {
        Write-Host ""
        Write-Host "Running: xcodebuild -project Mobile/iOS/ArcadiaApp.xcodeproj -scheme ArcadiaApp -configuration $configuration -destination id=$deviceUdid build"
        & xcodebuild `
            -project "Mobile/iOS/ArcadiaApp.xcodeproj" `
            -scheme "ArcadiaApp" `
            -configuration $configuration `
            -destination "id=$deviceUdid" `
            -derivedDataPath $derivedDataPath `
            build
    }
    finally {
        Pop-Location
    }

    $appPath = Join-Path $derivedDataPath "Build/Products/$configuration-iphoneos/ArcadiaApp.app"
    if (-not (Test-Path $appPath)) {
        throw "Built app not found at $appPath"
    }

    Write-Host ""
    Write-Host "Installing app on device $deviceUdid..."
    if ($env:ARCADIA_IOS_FORCE_UNINSTALL -eq "1") {
        Write-Host "Removing existing app (if installed)..."
        & xcrun devicectl device uninstall app --device $deviceUdid $bundleId 2>$null
    }
    & xcrun devicectl device install app --device $deviceUdid $appPath
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to install app on device."
    }

    Write-Host "Launching app ($bundleId) on device $deviceUdid..."
    & xcrun devicectl device process launch --device $deviceUdid $bundleId
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to launch app on device."
    }
}

while ($true) {
    Clear-Host
    Write-Host "=================================="
    Write-Host " Arcadia Launcher"
    Write-Host "=================================="
    Write-Host "Choose option (type two keys, no Enter needed):"
    Write-Host "  1A) Launch GUI Release"
    Write-Host "  1B) Launch GUI Debug"
    Write-Host "  2A) Launch Headless Release"
    Write-Host "  2B) Launch Headless Debug"
    Write-Host "  3A) Deploy iOS Release on Device"
    Write-Host "  3B) Deploy iOS Debug on Device"
    Write-Host "  0X) Exit"
    Write-Host ""
    Write-Host -NoNewline "Enter choice: "

    $first = [System.Console]::ReadKey($true).KeyChar
    $second = [System.Console]::ReadKey($true).KeyChar
    $choice = ("{0}{1}" -f $first, $second).ToUpperInvariant()
    Write-Host $choice

    switch ($choice) {
        "1A" {
            Invoke-Arcadia -Release $true -Feature "gui"
            Read-Host "Press Enter to continue..."
        }
        "1B" {
            Invoke-Arcadia -Release $false -Feature "gui"
            Read-Host "Press Enter to continue..."
        }
        "2A" {
            Invoke-Arcadia -Release $true -Feature "headless"
            Read-Host "Press Enter to continue..."
        }
        "2B" {
            Invoke-Arcadia -Release $false -Feature "headless"
            Read-Host "Press Enter to continue..."
        }
        "3A" {
            Invoke-IosDeviceDeploy -Release $true
            Read-Host "Press Enter to continue..."
        }
        "3B" {
            Invoke-IosDeviceDeploy -Release $false
            Read-Host "Press Enter to continue..."
        }
        { $_ -in @("0X", "00", "0A", "0B") } {
            Write-Host "Goodbye."
            exit 0
        }
        default {
            Write-Host ""
            Write-Host "Invalid option. Use 1A, 1B, 2A, 2B, 3A, 3B, or 0X."
            Read-Host "Press Enter to continue..."
        }
    }
}
