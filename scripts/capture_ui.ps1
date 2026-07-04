<#
.SYNOPSIS
    Headless screenshot harness for Monstron (Hatchspire).

.DESCRIPTION
    Thin wrapper around the shared macroquad-toolkit capture script. Builds the
    debug exe and drives it through the env-var capture hook
    (MONSTRON_CAPTURE_*) provided by macroquad_toolkit::capture in
    src/main.rs. Scenes are seeded via Game::begin_capture_scene:
      - "mainmenu" -> boot state (main menu)
      - "town"     -> fresh save, town screen
      - "hatchery" -> fresh save, hatchery screen

.EXAMPLE
    ./scripts/capture_ui.ps1
    ./scripts/capture_ui.ps1 -Frames 60 -SkipBuild
#>
param(
    [string[]]$Scenes = @("mainmenu", "town", "hatchery"),
    [int]$Frames = 150,
    [string]$OutputDir = "docs\verification",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$gameDir = Split-Path -Parent $PSScriptRoot
$shared = Join-Path (Split-Path -Parent $gameDir) "macroquad-toolkit\scripts\capture_ui.ps1"

& $shared -GameDir $gameDir -Scenes $Scenes -Frames $Frames -OutputDir $OutputDir -SkipBuild:$SkipBuild
