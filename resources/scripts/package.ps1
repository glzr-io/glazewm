# Usage: ./resources/scripts/package.ps1 -VERSION_NUMBER 1.0.0
param(
  [Parameter(Mandatory=$true)]
  [string]$VERSION_NUMBER
)

function ExitOnError() {
  if ($LASTEXITCODE -ne 0) {
    Exit 1
  }
}

function SignFiles() {
  param(
    [Parameter(Mandatory)]
    [string[]]$filePaths
  )

  $secrets = @(
    $ENV:AZ_VAULT_URL,
    $ENV:AZ_CERT_NAME,
    $ENV:AZ_CLIENT_ID,
    $ENV:AZ_CLIENT_SECRET,
    $ENV:AZ_TENANT_ID,
    $ENV:RFC3161_TIMESTAMP_URL
  )

  foreach ($secret in $secrets) {
    if (!$secret) {
      Write-Output "Skipping signing due to missing secret."
      Return
    }
  }

  Write-Output "Signing $filePaths."
  azuresigntool sign -kvu $ENV:AZ_VAULT_URL `
    -kvc $ENV:AZ_CERT_NAME `
    -kvi $ENV:AZ_CLIENT_ID `
    -kvs $ENV:AZ_CLIENT_SECRET `
    -kvt $ENV:AZ_TENANT_ID `
    -tr $ENV:RFC3161_TIMESTAMP_URL `
    -td sha256 $filePaths

  ExitOnError
}

function DownloadZebarInstallers() {
  Write-Output "Downloading latest Zebar MSI's"

  $latestRelease = 'https://api.github.com/repos/glzr-io/zebar/releases/latest'
  $latestInstallers = Invoke-RestMethod $latestRelease | % assets | ? name -like "*.msi"
  $latestInstallers | % { Invoke-WebRequest $_.browser_download_url -OutFile (Join-Path "out" $_.name) }
}

function BuildExes() {
  # Rust targets to build for (x64 and arm64).
  $rustTargets = @("x86_64-pc-windows-msvc", "aarch64-pc-windows-msvc")

  # Set the version number as an environment variable for `cargo build`.
  $env:VERSION_NUMBER = $VERSION_NUMBER

  foreach ($target in $rustTargets) {
    $outDir = if ($target -eq "x86_64-pc-windows-msvc") { "out/x64" } else { "out/arm64" }
    $sourceDir = "target/$target/release"

    Write-Output "Creating output directories for executables"
    New-Item -ItemType Directory -Force -Path "$outDir/noconsole", "$outDir/console"

    Write-Output "Building for $target (windows subsystem)"
    cargo build --locked --release --target $target --features no_console,ui_access
    Move-Item -Force -Path "$sourceDir/glazewm.exe", "$sourceDir/glazewm-watcher.exe" -Destination "$outDir/noconsole"

    Write-Output "Building for $target (console subsystem)"
    cargo build --locked --release --target $target --features ui_access
    Move-Item -Force -Path "$sourceDir/glazewm.exe", "$sourceDir/glazewm-watcher.exe" -Destination "$outDir/console"

    SignFiles @(
      "$outDir/noconsole/glazewm.exe",
      "$outDir/noconsole/glazewm-watcher.exe",
      "$outDir/console/glazewm.exe",
      "$outDir/console/glazewm-watcher.exe"
    )
  }
}

function BuildInstallers() {
  # WiX architectures to create installers for (x64 and arm64).
  $wixArchs = @("x64", "arm64")

  foreach ($arch in $wixArchs) {
    Write-Output "Creating MSI installer ($arch)"
    wix build -arch $arch -ext WixToolset.UI.wixext -ext WixToolset.Util.wixext `
      -out "./out/installer-$arch.msi" "./resources/wix/standalone.wxs" "./resources/wix/standalone-ui.wxs" `
      -d VERSION_NUMBER="$VERSION_NUMBER" `
      -d EXE_DIR="out/$arch"
  }

  Write-Output "Creating universal installer"
  wix build -arch "x64" -ext WixToolset.BootstrapperApplications.wixext `
    -out "./out/installer-universal.exe" "./resources/wix/bundle.wxs" `
    -d VERSION_NUMBER="$VERSION_NUMBER"

  SignFiles @(
    "out/installer-x64.msi",
    "out/installer-arm64.msi",
    "out/installer-universal.exe"
  )
}

function Package() {
  Write-Output "Packaging with version number: $VERSION_NUMBER"

  Write-Output "Creating output directory"
  New-Item -ItemType Directory -Force -Path "out"

  DownloadZebarInstallers
  BuildExes
  BuildInstallers
}

Package
