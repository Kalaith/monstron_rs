param(
    [Parameter(Mandatory=$false)]
    [string]$PromptManifestPath = "assets\generated\monster_art\monster_art_prompts.json",

    [Parameter(Mandatory=$false)]
    [string]$Id = "sample_slime_rillfin_lineage",

    [Parameter(Mandatory=$false)]
    [switch]$All = $false,

    [Parameter(Mandatory=$false)]
    [switch]$Test = $false,

    [Parameter(Mandatory=$false)]
    [string]$OutputDir = "assets\generated\monster_art",

    [Parameter(Mandatory=$false)]
    [string]$ComfyUIServer = "127.0.0.1:8188",

    [Parameter(Mandatory=$false)]
    [string]$ComfyGenerateScript = "H:\VideoGeneration\image_tools\art_pipeline\generate-image.ps1"
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

if (-not (Test-Path -LiteralPath $PromptManifestPath)) {
    node tools\export_monster_art_prompts.mjs $PromptManifestPath
}

if (-not (Test-Path -LiteralPath $PromptManifestPath)) {
    throw "Prompt manifest was not found: $PromptManifestPath"
}

if (-not (Test-Path -LiteralPath $ComfyGenerateScript)) {
    throw "ComfyUI generator script was not found: $ComfyGenerateScript"
}

try {
    Invoke-WebRequest -UseBasicParsing -Uri "http://$ComfyUIServer/" -TimeoutSec 10 | Out-Null
} catch {
    throw "Cannot reach ComfyUI at $ComfyUIServer. Start ComfyUI before generating art."
}

$manifest = Get-Content -LiteralPath $PromptManifestPath -Raw | ConvertFrom-Json
$items = @($manifest.image_prompts)
if (-not $All) {
    $items = @($items | Where-Object { $_.id -eq $Id })
}

if ($items.Count -eq 0) {
    throw "No monster art prompt matched id '$Id'."
}

if (-not (Test-Path -LiteralPath $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
}

foreach ($item in $items) {
    $filename = if ($item.filename) { [string]$item.filename } else { [string]$item.id }
    $outputPath = Join-Path $OutputDir "$filename.png"
    $width = if ($item.Width) { [int]$item.Width } else { [int]$manifest.comfyui_defaults.width }
    $height = if ($item.Height) { [int]$item.Height } else { [int]$manifest.comfyui_defaults.height }
    $steps = if ($item.Steps) { [int]$item.Steps } else { [int]$manifest.comfyui_defaults.steps }
    $cfg = if ($item.CFG) { [double]$item.CFG } else { [double]$manifest.comfyui_defaults.cfg }
    $seed = if ($null -ne $item.Seed) { [int]$item.Seed } else { -1 }
    $model = if ($item.Model) { [string]$item.Model } else { [string]$manifest.comfyui_defaults.model }
    $sampler = if ($item.Sampler) { [string]$item.Sampler } else { [string]$manifest.comfyui_defaults.sampler }
    $scheduler = if ($item.Scheduler) { [string]$item.Scheduler } else { [string]$manifest.comfyui_defaults.scheduler }

    Write-Host "Monster art: $($item.id)" -ForegroundColor Cyan
    Write-Host "Output: $outputPath" -ForegroundColor Magenta
    if ($Test) {
        Write-Host "Prompt: $($item.Prompt)" -ForegroundColor Yellow
        Write-Host "Negative: $($item.NegativePrompt)" -ForegroundColor DarkYellow
        continue
    }

    & $ComfyGenerateScript `
        -Prompt ([string]$item.Prompt) `
        -NegativePrompt ([string]$item.NegativePrompt) `
        -OutputPath $outputPath `
        -ComfyUIServer $ComfyUIServer `
        -Width $width `
        -Height $height `
        -Steps $steps `
        -CFG $cfg `
        -Seed $seed `
        -Model $model `
        -Sampler $sampler `
        -Scheduler $scheduler

    if (-not (Test-Path -LiteralPath $outputPath)) {
        throw "ComfyUI generation did not create '$outputPath'."
    }
}
