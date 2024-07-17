# Usage: ./resources/scripts/package.ps1 -VERSION_NUMBER 1.0.0
param(
  [Parameter(Mandatory=$true)]
  [string]$VERSION_NUMBER
)

Write-Output "Packaging with version number: $VERSION_NUMBER"

Write-Output "Creating output directory"
New-Item -ItemType Directory -Force -Path "out"

Write-Output "Downloading latest Zebar MSI's"
$latestRelease = 'https://api.github.com/repos/glzr-io/zebar/releases/latest'
$latestInstallers = Invoke-RestMethod $latestRelease | % assets | ? name -like "*.msi"
$latestInstallers | % { Invoke-WebRequest $_.browser_download_url -OutFile (Join-Path "out" $_.name) }

# Rust targets to build for (x64 and arm64).
$rustTargets = @("x86_64-pc-windows-msvc", "aarch64-pc-windows-msvc")

foreach ($target in $rustTargets) {
  $outDir = if ($target -eq "x86_64-pc-windows-msvc") { "x64" } else { "arm64" }
  $sourceDir = "target/$target/release"

  Write-Output "Creating output directories for executables"
  New-Item -ItemType Directory -Force -Path "out/$outDir/noconsole", "out/$outDir/console"

  Write-Output "Building for $target (windows subsystem)"
  cargo build --locked --release --target $target --features no_console
  Move-Item -Path "$sourceDir/wm.exe", "$sourceDir/watcher.exe" -Destination "out/$outDir/noconsole"

  Write-Output "Building for $target (console subsystem)"
  cargo build --locked --release --target $target
  Move-Item -Path "$sourceDir/wm.exe", "$sourceDir/watcher.exe" -Destination "out/$outDir/console"
}

# WiX architectures to create installers for (x64 and arm64).
$wixArchs = @("x64", "arm64")

foreach ($arch in $wixArchs) {
  Write-Output "Creating MSI installer ($arch)"
  wix build -arch $arch -ext WixToolset.UI.wixext -ext WixToolset.Util.wixext `
    -out "./out/installer-$arch.msi" "./resources/wix/standalone.wxs" "./resources/wix/standalone-ui.wxs" `
    -d VERSION_NUMBER="$VERSION_NUMBER"
}

Write-Output "Creating universal installer"
wix build -arch "x64" -ext WixToolset.BootstrapperApplications.wixext `
  -out "./out/installer-universal.exe" "./resources/wix/bundle.wxs" `
  -d VERSION_NUMBER="$VERSION_NUMBER"
