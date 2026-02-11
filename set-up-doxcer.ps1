<#
set-up-doxcer.ps1

Purpose:
- Ensure Microsoft ODBC Driver 18 and Azure CLI are available (admin only when installation is needed).
- Select the newest dist release by parsing the timestamp in the folder name.
- Copy the newest doxcer.exe into %LOCALAPPDATA%\doxcer\bin\doxcer.exe.
- Regenerate config/system.env with ABSOLUTE_DOXCER_PATH.
- In admin mode, enforce one managed doxcer path entry in user PATH.

Supported release folder name:
- doxcer-windows-x64-yyyy-MM-dd_HH-mm-ss
#>

[CmdletBinding()]
param(
    # Relative path from repo root to the dist folder.
    [string]$RelativeDist = "dist",

    # Relative path from repo root to generated system env file.
    [string]$RelativeSystemEnv = "config\system.env",

    # Show verbose output for Copy-Item.
    [switch]$VerboseCopy
)

$ScriptName = if ($PSCommandPath) { Split-Path -Leaf $PSCommandPath } else { "set-up-doxcer.ps1" }


# ============================
# Generic helpers
# ============================

function Write-Log {
    param(
        [Parameter(Mandatory)]
        [ValidateSet("INF", "WRN", "ERR", "SUC")]
        [string]$Level,

        [Parameter(Mandatory)]
        [string]$Message
    )

    $color = switch ($Level) {
        "INF" { "Cyan" }
        "WRN" { "Yellow" }
        "ERR" { "Red" }
        "SUC" { "Green" }
    }

    Write-Host "[$Level] - $Message" -ForegroundColor $color
}

function Write-Info {
    param([Parameter(Mandatory)][string]$Message)
    Write-Log -Level "INF" -Message $Message
}

function Write-WarnLog {
    param([Parameter(Mandatory)][string]$Message)
    Write-Log -Level "WRN" -Message $Message
}

function Write-ErrorLog {
    param([Parameter(Mandatory)][string]$Message)
    Write-Log -Level "ERR" -Message $Message
}

function Write-Success {
    param([Parameter(Mandatory)][string]$Message)
    Write-Log -Level "SUC" -Message $Message
}

function Write-Section {
    param([Parameter(Mandatory)][string]$Title)
    Write-Info "=== $Title ==="
}

function Test-IsAdmin {
    $currentIdentity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentIdentity)
    return $principal.IsInRole([Security.Principal.WindowsBuiltinRole]::Administrator)
}

function Get-RepoRoot {
    if ($PSScriptRoot) {
        return $PSScriptRoot
    }

    if ($PSCommandPath) {
        return (Split-Path -Parent $PSCommandPath)
    }

    return (Get-Location).Path
}

function Normalize-PathValue {
    param([Parameter(Mandatory)][string]$PathValue)

    $expanded = [Environment]::ExpandEnvironmentVariables($PathValue.Trim())
    try {
        return ([IO.Path]::GetFullPath($expanded)).TrimEnd('\')
    }
    catch {
        return $expanded.TrimEnd('\')
    }
}

function Test-PathEquals {
    param(
        [string]$Left,
        [string]$Right
    )

    if ([string]::IsNullOrWhiteSpace($Left) -or [string]::IsNullOrWhiteSpace($Right)) {
        return $false
    }

    $leftNorm = Normalize-PathValue -PathValue $Left
    $rightNorm = Normalize-PathValue -PathValue $Right
    return [string]::Equals($leftNorm, $rightNorm, [System.StringComparison]::OrdinalIgnoreCase)
}

function Split-PathEntries {
    param([string]$PathValue)

    if ([string]::IsNullOrWhiteSpace($PathValue)) {
        return @()
    }

    return $PathValue.Split(';') |
        ForEach-Object { $_.Trim() } |
        Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
}

function Read-SystemEnvValues {
    param([Parameter(Mandatory)][string]$SystemEnvPath)

    $values = @{}
    if (-not (Test-Path -Path $SystemEnvPath -PathType Leaf)) {
        return $values
    }

    foreach ($line in Get-Content -Path $SystemEnvPath -ErrorAction Stop) {
        $trimmed = $line.Trim()
        if ([string]::IsNullOrWhiteSpace($trimmed) -or $trimmed.StartsWith("#")) {
            continue
        }

        $equalsIndex = $trimmed.IndexOf("=")
        if ($equalsIndex -lt 1) {
            continue
        }

        $key = $trimmed.Substring(0, $equalsIndex).Trim()
        $value = $trimmed.Substring($equalsIndex + 1).Trim()
        if ($value.Length -ge 2) {
            if (
                ($value.StartsWith("'") -and $value.EndsWith("'")) -or
                ($value.StartsWith('"') -and $value.EndsWith('"'))
            ) {
                $value = $value.Substring(1, $value.Length - 2)
            }
        }

        $values[$key] = $value
    }

    return $values
}


# ============================
# Dist release helpers
# ============================

function Get-ReleaseTimestampFromFolderName {
    param([Parameter(Mandatory)][string]$FolderName)

    if ($FolderName -notmatch "^doxcer-windows-x64-(?<Timestamp>\d{4}-\d{2}-\d{2}_\d{2}-\d{2}-\d{2})$") {
        return $null
    }

    $timestampText = $Matches.Timestamp
    try {
        return [DateTime]::ParseExact(
            $timestampText,
            "yyyy-MM-dd_HH-mm-ss",
            [System.Globalization.CultureInfo]::InvariantCulture,
            [System.Globalization.DateTimeStyles]::AssumeLocal
        )
    }
    catch {
        return $null
    }
}

function Get-LatestDoxcerRelease {
    param([Parameter(Mandatory)][string]$DistDir)

    $foldersWithExe = Get-ChildItem -Path $DistDir -Directory -ErrorAction Stop |
        Where-Object { Test-Path -Path (Join-Path $_.FullName "doxcer.exe") -PathType Leaf }

    if (-not $foldersWithExe) {
        throw "No build folders with doxcer.exe found in '$DistDir'."
    }

    $candidates = foreach ($folder in $foldersWithExe) {
        $timestamp = Get-ReleaseTimestampFromFolderName -FolderName $folder.Name
        if ($null -ne $timestamp) {
            [PSCustomObject]@{
                Folder    = $folder
                Timestamp = $timestamp
            }
        }
    }

    if (-not $candidates) {
        $folderNames = ($foldersWithExe.Name -join "', '")
        throw "No dist folder matches expected format 'doxcer-windows-x64-yyyy-MM-dd_HH-mm-ss'. Found: '$folderNames'."
    }

    return ($candidates | Sort-Object -Property Timestamp -Descending | Select-Object -First 1)
}

function Get-ManagedBinDir {
    $localAppData = $env:LOCALAPPDATA
    if ([string]::IsNullOrWhiteSpace($localAppData)) {
        $localAppData = [Environment]::GetFolderPath([Environment+SpecialFolder]::LocalApplicationData)
    }

    if ([string]::IsNullOrWhiteSpace($localAppData)) {
        throw "Could not resolve LOCALAPPDATA for the current user."
    }

    return (Join-Path $localAppData "doxcer\bin")
}

function Install-LatestDoxcerExe {
    param(
        [Parameter(Mandatory)][string]$RepoRoot,
        [Parameter(Mandatory)][string]$RelativeDist,
        [Parameter(Mandatory)][string]$TargetBinDir,
        [switch]$VerboseCopy
    )

    $distDir = Join-Path $RepoRoot $RelativeDist
    $targetDir = Normalize-PathValue -PathValue $TargetBinDir
    $targetExe = Join-Path $targetDir "doxcer.exe"

    Write-Info "Repo root : $RepoRoot"
    Write-Info "Dist dir  : $distDir"
    Write-Info "Target dir: $targetDir"

    if (-not (Test-Path -Path $distDir -PathType Container)) {
        throw "Cannot find dist directory at: $distDir"
    }

    $latest = Get-LatestDoxcerRelease -DistDir $distDir
    $sourceExe = Join-Path $latest.Folder.FullName "doxcer.exe"

    if (-not (Test-Path -Path $sourceExe -PathType Leaf)) {
        throw "Expected doxcer.exe not found at: $sourceExe"
    }

    if (-not (Test-Path -Path $targetDir -PathType Container)) {
        Write-Info "Creating $targetDir ..."
        New-Item -ItemType Directory -Path $targetDir -Force | Out-Null
    }

    $copyParams = @{
        Path        = $sourceExe
        Destination = $targetExe
        Force       = $true
        ErrorAction = "Stop"
    }

    if ($VerboseCopy) {
        $copyParams["Verbose"] = $true
    }

    for ($attempt = 1; $attempt -le 3; $attempt++) {
        try {
            Copy-Item @copyParams
            break
        }
        catch {
            if ($attempt -eq 3) {
                throw "Failed to install doxcer.exe to '$targetExe'. Ensure no running doxcer.exe process is locking the file. Inner error: $($_.Exception.Message)"
            }
            Start-Sleep -Milliseconds 500
        }
    }

    Write-Info "Selected release : $($latest.Folder.Name)"
    Write-Info "Release timestamp: $($latest.Timestamp.ToString("yyyy-MM-dd HH:mm:ss"))"
    Write-Success "Installed        : $targetExe"
}


# ============================
# system.env and PATH helpers
# ============================

function Write-SystemEnvFile {
    param(
        [Parameter(Mandatory)][string]$RepoRoot,
        [Parameter(Mandatory)][string]$SystemEnvPath
    )

    $configDir = Split-Path -Parent $SystemEnvPath
    if (-not (Test-Path -Path $configDir -PathType Container)) {
        New-Item -Path $configDir -ItemType Directory -Force | Out-Null
    }

    $creationIso = Get-Date -Format "o"
    $repoRootNormalized = Normalize-PathValue -PathValue $RepoRoot

    $lines = @(
        "##########################################################",
        "# AUTHOR: Stefan B. J. Meeuwessen",
        "# CREATION: $creationIso",
        "##########################################################",
        "",
        "",
        "###############################",
        "# AUTOMATICALLY GENERATED FILE",
        "###############################",
        "",
        "ABSOLUTE_DOXCER_PATH=$repoRootNormalized"
    )

    Set-Content -Path $SystemEnvPath -Value $lines -Encoding UTF8
    Write-Success "Generated       : $SystemEnvPath"
}

function Set-AbsoluteDoxcerPathUserEnv {
    param([Parameter(Mandatory)][string]$RepoRoot)

    $repoRootNormalized = Normalize-PathValue -PathValue $RepoRoot
    [Environment]::SetEnvironmentVariable("ABSOLUTE_DOXCER_PATH", $repoRootNormalized, "User")
    $env:ABSOLUTE_DOXCER_PATH = $repoRootNormalized
    Write-Success "Set user env    : ABSOLUTE_DOXCER_PATH=$repoRootNormalized"
}

function Remove-LegacyVenvDoxcerExe {
    param(
        [Parameter(Mandatory)][string]$RepoRoot,
        [string]$PreviousRepoRoot
    )

    $repoCandidates = New-Object System.Collections.Generic.List[string]
    $repoCandidates.Add($RepoRoot)
    if ($PreviousRepoRoot -and -not (Test-PathEquals -Left $RepoRoot -Right $PreviousRepoRoot)) {
        $repoCandidates.Add($PreviousRepoRoot)
    }

    $seenRepos = New-Object 'System.Collections.Generic.HashSet[string]' ([System.StringComparer]::OrdinalIgnoreCase)
    foreach ($repoCandidate in $repoCandidates) {
        if ([string]::IsNullOrWhiteSpace($repoCandidate)) {
            continue
        }

        $normalizedRepo = Normalize-PathValue -PathValue $repoCandidate
        if (-not $seenRepos.Add($normalizedRepo)) {
            continue
        }

        $legacyExe = Join-Path (Join-Path $normalizedRepo ".venv\Scripts") "doxcer.exe"
        if (Test-Path -Path $legacyExe -PathType Leaf) {
            try {
                Remove-Item -Path $legacyExe -Force -ErrorAction Stop
                Write-Info "Removed legacy executable: $legacyExe"
            }
            catch {
                Write-WarnLog "Failed to remove legacy executable '$legacyExe': $($_.Exception.Message)"
            }
        }
    }
}

function Ensure-UserPathForDoxcer {
    param(
        [Parameter(Mandatory)][string]$ManagedBinDir,
        [Parameter(Mandatory)][string]$RepoRoot,
        [string]$PreviousRepoRoot
    )

    if (-not (Test-IsAdmin)) {
        Write-WarnLog "Skipping PATH cleanup/update because this run is not elevated."
        Write-WarnLog "Run set-up-doxcer.ps1 as Administrator to enforce one doxcer location on PATH."
        return
    }

    $managedBin = Normalize-PathValue -PathValue $ManagedBinDir
    if (-not (Test-Path -Path $managedBin -PathType Container)) {
        New-Item -Path $managedBin -ItemType Directory -Force | Out-Null
    }

    $knownLegacy = New-Object 'System.Collections.Generic.HashSet[string]' ([System.StringComparer]::OrdinalIgnoreCase)
    [void]$knownLegacy.Add((Normalize-PathValue -PathValue (Join-Path $RepoRoot ".venv\Scripts")))
    if ($PreviousRepoRoot -and -not (Test-PathEquals -Left $RepoRoot -Right $PreviousRepoRoot)) {
        [void]$knownLegacy.Add((Normalize-PathValue -PathValue (Join-Path $PreviousRepoRoot ".venv\Scripts")))
    }

    $userPathRaw = [Environment]::GetEnvironmentVariable("PATH", "User")
    $entries = Split-PathEntries -PathValue $userPathRaw

    $cleanedEntries = New-Object System.Collections.Generic.List[string]
    $seenEntries = New-Object 'System.Collections.Generic.HashSet[string]' ([System.StringComparer]::OrdinalIgnoreCase)
    $staleEntries = New-Object 'System.Collections.Generic.HashSet[string]' ([System.StringComparer]::OrdinalIgnoreCase)
    $staleExecutables = New-Object 'System.Collections.Generic.HashSet[string]' ([System.StringComparer]::OrdinalIgnoreCase)

    foreach ($entry in $entries) {
        $entryNorm = Normalize-PathValue -PathValue $entry

        if (Test-PathEquals -Left $entryNorm -Right $managedBin) {
            if ($seenEntries.Add($entryNorm)) {
                $cleanedEntries.Add($entryNorm)
            }
            continue
        }

        $isLegacy = $knownLegacy.Contains($entryNorm)
        $entryExe = Join-Path $entryNorm "doxcer.exe"
        $hasDoxcerExe = Test-Path -Path $entryExe -PathType Leaf

        if ($isLegacy -or $hasDoxcerExe) {
            [void]$staleEntries.Add($entryNorm)
            if ($hasDoxcerExe) {
                [void]$staleExecutables.Add($entryExe)
            }
            continue
        }

        if ($seenEntries.Add($entryNorm)) {
            $cleanedEntries.Add($entryNorm)
        }
    }

    if ($seenEntries.Add($managedBin)) {
        $cleanedEntries.Add($managedBin)
        Write-Info "Added managed bin to user PATH: $managedBin"
    }

    $newUserPath = ($cleanedEntries -join ";")
    [Environment]::SetEnvironmentVariable("PATH", $newUserPath, "User")

    $machinePath = [Environment]::GetEnvironmentVariable("PATH", "Machine")
    if ([string]::IsNullOrWhiteSpace($machinePath)) {
        $env:PATH = $newUserPath
    }
    else {
        $env:PATH = "$machinePath;$newUserPath"
    }

    foreach ($staleEntry in $staleEntries) {
        Write-Info "Removed stale PATH entry: $staleEntry"
    }

    foreach ($staleExe in $staleExecutables) {
        try {
            Remove-Item -Path $staleExe -Force -ErrorAction Stop
            Write-Info "Removed stale executable: $staleExe"
        }
        catch {
            Write-WarnLog "Failed to remove stale executable '$staleExe': $($_.Exception.Message)"
        }
    }

    Write-Success "User PATH now points to one doxcer executable location."
}


# ============================
# ODBC Driver 18 installer
# ============================

function Test-OdbcDriver18Installed {
    # Primary: registry check (64-bit ODBC)
    try {
        $drivers64 = Get-ItemProperty "HKLM:\SOFTWARE\ODBC\ODBCINST.INI\ODBC Drivers" -ErrorAction Stop
        if (
            $drivers64.PSObject.Properties.Name -contains "ODBC Driver 18 for SQL Server" -and
            $drivers64."ODBC Driver 18 for SQL Server" -eq "Installed"
        ) {
            return $true
        }
    }
    catch { }

    # Secondary: Get-OdbcDriver (if available)
    try {
        $found = Get-OdbcDriver -ErrorAction SilentlyContinue |
            Where-Object { $_.Name -match "ODBC Driver 18|Microsoft ODBC Driver 18" }
        if ($found) {
            return $true
        }
    }
    catch { }

    return $false
}

function Install-OdbcDriver18 {
    Write-WarnLog "Microsoft ODBC Driver 18 for SQL Server not found. Attempting installation..."

    if (-not (Test-IsAdmin)) {
        Write-WarnLog "You are not running PowerShell as Administrator."
        Write-WarnLog "Installing ODBC Driver 18 requires elevation."
        Write-WarnLog "Open an elevated PowerShell and rerun '.\$ScriptName',"
        Write-WarnLog "or have IT deploy the driver via Intune/SCCM."
        return $false
    }

    # Microsoft fwlink for ODBC Driver 18 (x64)
    $installerUrl = "https://go.microsoft.com/fwlink/?linkid=2345415"
    $tempMsi = Join-Path ([IO.Path]::GetTempPath()) "msodbcsql18_x64.msi"
    $logPath = Join-Path ([IO.Path]::GetTempPath()) "msodbcsql18-install.log"
    $previousProgressPreference = $ProgressPreference

    try {
        Write-Info "Downloading ODBC Driver 18 installer from $installerUrl ..."
        $ProgressPreference = "SilentlyContinue"
        Invoke-WebRequest -Uri $installerUrl -OutFile $tempMsi -UseBasicParsing -ErrorAction Stop

        Write-Info "Running ODBC Driver 18 installer silently (log: $logPath)..."
        $proc = Start-Process -FilePath "msiexec.exe" `
            -ArgumentList "/i", "`"$tempMsi`"",
                          "IACCEPTMSODBCSQLLICENSETERMS=YES",
                          "ADDLOCAL=ALL",
                          "/qn",
                          "/l*v", "`"$logPath`"" `
            -Wait -PassThru

        if ($proc.ExitCode -eq 3010) {
            Write-WarnLog "ODBC Driver 18 installed, but a reboot is required (exit code 3010)."
            return $true
        }

        if ($proc.ExitCode -ne 0) {
            Write-WarnLog "ODBC Driver 18 installation failed with exit code $($proc.ExitCode)."
            Write-WarnLog "Last log lines:"
            try {
                Get-Content -Path $logPath -Tail 40 | ForEach-Object { Write-WarnLog $_ }
            }
            catch { }
            return $false
        }

        if (Test-OdbcDriver18Installed) {
            Write-Success "ODBC Driver 18 installed successfully."
            return $true
        }

        Write-WarnLog "Installer reported success, but ODBC Driver 18 is still not detected."
        Write-WarnLog "Check log: $logPath"
        return $false
    }
    catch {
        Write-ErrorLog "Failed to download or install ODBC Driver 18: $($_.Exception.Message)"
        return $false
    }
    finally {
        if (Test-Path -Path $tempMsi -PathType Leaf) {
            Remove-Item -Path $tempMsi -ErrorAction SilentlyContinue
        }
        $ProgressPreference = $previousProgressPreference
    }
}

function Ensure-OdbcDriver18Ready {
    Write-Section "Ensuring Microsoft ODBC Driver 18 for SQL Server is installed"

    if (Test-OdbcDriver18Installed) {
        Write-Success "ODBC Driver 18 is already installed."
        return
    }

    $installed = Install-OdbcDriver18
    if (-not $installed) {
        Write-WarnLog "ODBC Driver 18 is not available."
        Write-WarnLog "Fabric/SQL features may fail until the driver is installed."
    }
}


# ============================
# Azure CLI installer
# ============================

function Test-AzCliInstalled {
    try {
        $cmd = Get-Command az -ErrorAction SilentlyContinue
        return [bool]$cmd
    }
    catch {
        return $false
    }
}

function Install-AzureCli {
    Write-WarnLog "Azure CLI not found. Attempting installation..."

    if (-not (Test-IsAdmin)) {
        Write-WarnLog "You are not running PowerShell as Administrator."
        Write-WarnLog "The Azure CLI MSI installer typically requires elevation."
        Write-WarnLog "Open an elevated PowerShell and rerun '.\$ScriptName',"
        Write-WarnLog "or install manually via https://aka.ms/installazurecliwindows"
        return $false
    }

    $installerUrl = "https://aka.ms/installazurecliwindowsx64"
    $tempMsi = Join-Path ([IO.Path]::GetTempPath()) "AzureCLI.msi"
    $previousProgressPreference = $ProgressPreference

    try {
        Write-Info "Downloading Azure CLI installer from $installerUrl ..."
        $ProgressPreference = "SilentlyContinue"
        Invoke-WebRequest -Uri $installerUrl -OutFile $tempMsi -UseBasicParsing -ErrorAction Stop

        Write-Info "Running Azure CLI installer..."
        $proc = Start-Process -FilePath "msiexec.exe" `
            -ArgumentList "/I", "`"$tempMsi`"", "/quiet" `
            -Wait -PassThru

        if ($proc.ExitCode -ne 0) {
            Write-WarnLog "Azure CLI installation failed with exit code $($proc.ExitCode)."
            Write-WarnLog "Install manually from https://aka.ms/installazurecliwindows"
            return $false
        }

        Write-Success "Azure CLI installed successfully."
        return $true
    }
    catch {
        Write-ErrorLog "Failed to download or install Azure CLI: $($_.Exception.Message)"
        Write-WarnLog "Install manually from https://aka.ms/installazurecliwindows"
        return $false
    }
    finally {
        if (Test-Path -Path $tempMsi -PathType Leaf) {
            Remove-Item -Path $tempMsi -ErrorAction SilentlyContinue
        }
        $ProgressPreference = $previousProgressPreference
    }
}

function Test-AzCliLoggedIn {
    if (-not (Test-AzCliInstalled)) {
        return $false
    }

    try {
        & az account show --only-show-errors --output none 2>$null
        return ($LASTEXITCODE -eq 0)
    }
    catch {
        return $false
    }
}

function Ensure-AzCliReady {
    Write-Section "Ensuring Azure CLI is installed and logged in"

    if (-not (Test-AzCliInstalled)) {
        $installed = Install-AzureCli
        if (-not $installed) {
            Write-WarnLog "Skipping Azure login because Azure CLI is not available."
            Write-WarnLog "Doxcer may fail to fetch Key Vault secrets until Azure CLI is installed and logged in."
            return
        }

        # Refresh PATH in this process after MSI installation.
        $env:PATH = [System.Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" +
                    [System.Environment]::GetEnvironmentVariable("PATH", "User")
    }
    else {
        Write-Success "Azure CLI is already installed."
    }

    if (Test-AzCliLoggedIn) {
        Write-Success "Azure CLI is already logged in."
        return
    }

    Write-Info "Azure CLI is not logged in."
    Write-Info "A browser window may open for login..."

    try {
        & az login
        if ($LASTEXITCODE -eq 0 -and (Test-AzCliLoggedIn)) {
            Write-Success "Azure CLI login successful."
        }
        else {
            Write-WarnLog "Azure CLI login did not complete successfully."
            Write-WarnLog "Try manually: az login"
        }
    }
    catch {
        Write-ErrorLog "Failed to start 'az login': $($_.Exception.Message)"
        Write-WarnLog "Try manually: az login"
    }
}


# ============================
# Main flow
# ============================

$repoRoot = Normalize-PathValue -PathValue (Get-RepoRoot)
$systemEnvPath = Join-Path $repoRoot $RelativeSystemEnv
$previousValues = Read-SystemEnvValues -SystemEnvPath $systemEnvPath
$previousRepoRoot = $previousValues["ABSOLUTE_DOXCER_PATH"]
$managedBinDir = Get-ManagedBinDir

# Admin is only required if installation is needed.
Ensure-OdbcDriver18Ready
Ensure-AzCliReady

Write-Section "Installing latest doxcer release into %LOCALAPPDATA%\doxcer\bin"
try {
    Install-LatestDoxcerExe -RepoRoot $repoRoot -RelativeDist $RelativeDist -TargetBinDir $managedBinDir -VerboseCopy:$VerboseCopy
}
catch {
    Write-ErrorLog $_.Exception.Message
    exit 1
}

Write-Section "Generating config/system.env"
try {
    Write-SystemEnvFile -RepoRoot $repoRoot -SystemEnvPath $systemEnvPath
    Set-AbsoluteDoxcerPathUserEnv -RepoRoot $repoRoot
}
catch {
    Write-ErrorLog $_.Exception.Message
    exit 1
}

Write-Section "Cleaning legacy .venv\\Scripts installation"
Remove-LegacyVenvDoxcerExe -RepoRoot $repoRoot -PreviousRepoRoot $previousRepoRoot

Write-Section "Updating user PATH in admin mode"
Ensure-UserPathForDoxcer -ManagedBinDir $managedBinDir -RepoRoot $repoRoot -PreviousRepoRoot $previousRepoRoot

Write-Success "Setup complete. Open a new terminal and run 'doxcer --help'."
