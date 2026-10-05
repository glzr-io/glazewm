# One-line installer for GlazeWM on Windows.
#
# Usage:
#   irm https://raw.githubusercontent.com/hairbui76/glazewm/main/resources/scripts/install.ps1 | iex
#
# Configurable through environment variables, since arguments can't be
# passed to a script that's piped into `iex`:
#   $env:GLAZEWM_VERSION = '3.11.0'          # Version to install. Defaults to latest.
#   $env:GLAZEWM_REPO    = 'owner/repo'      # Repository to install from.
#   $env:GLAZEWM_SILENT  = '1'               # Install without any installer UI ('1'/'true'/'yes').

#Requires -Version 5.1

$ErrorActionPreference = 'Stop'

function Install-GlazeWM {
  [CmdletBinding()]
  param(
    # Repository to resolve releases from, in `owner/repo` format.
    [string]$Repo = $(
      if ($env:GLAZEWM_REPO) { $env:GLAZEWM_REPO } else { 'hairbui76/glazewm' }
    ),

    # Version to install (e.g. `3.11.0`). Defaults to the latest release.
    [string]$Version = $env:GLAZEWM_VERSION,

    # Whether to run the installer without any UI.
    [switch]$Silent = ($env:GLAZEWM_SILENT -in @('1', 'true', 'yes'))
  )

  # Older PowerShell hosts default to TLS 1.0, which GitHub rejects.
  [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

  # Progress rendering makes `Invoke-WebRequest` downloads an order of
  # magnitude slower.
  $originalProgress = $ProgressPreference
  $ProgressPreference = 'SilentlyContinue'

  $downloadDir = Join-Path $env:TEMP 'glazewm-install'
  $installerPath = $null

  try {
    $tag = Resolve-GlazeWMTag -Repo $Repo -Version $Version

    # Named by the release pipeline after the tag of the release.
    $installerName = "glazewm-$tag.exe"
    $installerUrl = "https://github.com/$Repo/releases/download/$tag/$installerName"

    Write-Host "Installing GlazeWM $tag." -ForegroundColor Cyan

    New-Item -ItemType Directory -Force -Path $downloadDir | Out-Null
    $installerPath = Join-Path $downloadDir $installerName

    Write-Host "Downloading $installerName."

    try {
      Invoke-WebRequest -Uri $installerUrl -OutFile $installerPath -UseBasicParsing
    }
    catch {
      throw "Unable to download '$installerUrl'. Either release $tag doesn't exist, or its installer isn't attached yet. $($_.Exception.Message)"
    }

    # The installer is per-machine, so it self-elevates through UAC.
    $installerArgs = if ($Silent) { @('/quiet', '/norestart') } else { @('/passive', '/norestart') }

    Write-Host 'Running installer. Accept the UAC prompt to continue.'
    $process = Start-Process -FilePath $installerPath -ArgumentList $installerArgs -PassThru -Wait

    switch ($process.ExitCode) {
      0 {
        Write-Host "GlazeWM $tag installed." -ForegroundColor Green
      }
      3010 {
        Write-Host "GlazeWM $tag installed. Restart to complete setup." -ForegroundColor Green
      }
      default {
        throw "Installer exited with code $($process.ExitCode)."
      }
    }

    Write-Host ''
    Write-Host 'Launch it from the Start menu, or run `glazewm start` in a new terminal.'
  }
  finally {
    $ProgressPreference = $originalProgress

    if ($installerPath -and (Test-Path $installerPath)) {
      Remove-Item -Path $installerPath -Force -ErrorAction SilentlyContinue
    }
  }
}

function Resolve-GlazeWMTag {
  [CmdletBinding()]
  param(
    # Repository to resolve the release from, in `owner/repo` format.
    [Parameter(Mandatory)]
    [string]$Repo,

    # Version to resolve. Resolves the latest release when empty.
    [string]$Version
  )

  if ($Version) {
    return "v$($Version.TrimStart('v'))"
  }

  # The `releases/latest` page redirects to the latest release, whose URL
  # ends in its tag. The REST API is avoided on purpose: it limits
  # unauthenticated requests to 60 an hour per IP address, and addresses
  # are routinely shared between many people, so that limit is often spent
  # before the first request is made.
  $url = "https://github.com/$Repo/releases/latest"

  try {
    $response = Invoke-WebRequest -Uri $url -Method Head -UseBasicParsing
  }
  catch {
    throw "Unable to resolve the latest GlazeWM release from '$Repo'. $($_.Exception.Message)"
  }

  # Windows PowerShell exposes the URL that redirects led to as
  # `ResponseUri`, whereas PowerShell 7 exposes it on the request message.
  $finalUri = if ($response.BaseResponse.ResponseUri) {
    $response.BaseResponse.ResponseUri
  } else {
    $response.BaseResponse.RequestMessage.RequestUri
  }

  if ("$finalUri" -notmatch '/releases/tag/([^/?#]+)') {
    throw "'$Repo' has no published release."
  }

  $Matches[1]
}

Install-GlazeWM
