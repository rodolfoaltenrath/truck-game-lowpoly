# Compila localmente quando o proprietário decidir; sem modificar o PATH global.
[CmdletBinding()]
param(
    [ValidateSet('Full', 'Run', 'Check')]
    [string]$Mode = 'Full',
    [switch]$Dev,
    [switch]$Smoke
)
$ErrorActionPreference = 'Stop'
if ($Mode -ne 'Full' -and ($Dev -or $Smoke)) {
    throw '-Dev e -Smoke são opções do modo Full.'
}
$root = Split-Path -Parent $PSScriptRoot
$previousPath = $env:Path
$previousFlags = $env:CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS
$previousJobs = $env:CARGO_BUILD_JOBS

function Invoke-Cargo {
    param([string[]]$CargoArgs)
    & cargo @CargoArgs
    if ($LASTEXITCODE -ne 0) { throw "cargo $($CargoArgs -join ' ') falhou ($LASTEXITCODE)." }
}

Push-Location $root
try {
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        throw 'Instale Rust com rustup e reabra o PowerShell. Consulte docs/WINDOWS_DEV.md.'
    }
    $rustInfo = & rustc -vV
    if ($LASTEXITCODE -ne 0) { throw 'Não foi possível identificar o toolchain Rust.' }
    if ($rustInfo -match 'host: .*windows-gnu') {
        $mingw = Join-Path $env:LOCALAPPDATA 'truck-build-tools/w64devkit/bin'
        if (Test-Path -LiteralPath (Join-Path $mingw 'gcc.exe')) {
            $env:Path = "$mingw;$env:Path"
        }
        if (-not (Get-Command gcc -ErrorAction SilentlyContinue)) {
            throw 'Toolchain GNU requer MinGW/w64devkit no PATH. Consulte docs/WINDOWS_DEV.md.'
        }
        if (-not $env:CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS) {
            $env:CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS = '-Clink-self-contained=yes'
        }
    }
    if (-not $env:CARGO_BUILD_JOBS) { $env:CARGO_BUILD_JOBS = '2' }
    if ($Mode -eq 'Run') {
        Invoke-Cargo -CargoArgs @('run', '--locked', '-p', 'truck_client', '--', '--solo')
        return
    }
    if ($Mode -eq 'Check') {
        Invoke-Cargo -CargoArgs @('check', '--locked', '-p', 'truck_client', '--tests')
        return
    }
    Invoke-Cargo -CargoArgs @('fmt', '--all', '--check')
    Invoke-Cargo -CargoArgs @('test', '--locked', '--workspace')
    Invoke-Cargo -CargoArgs @('clippy', '--locked', '--workspace', '--all-targets', '--', '-D', 'warnings')
    $packageArgs = @('run', '--locked', '-p', 'xtask', '--', 'dist')
    if ($Dev) { $packageArgs += '--dev' }
    Invoke-Cargo -CargoArgs $packageArgs
    if ($Smoke) {
        $binary = Join-Path $root 'dist/sul-do-brasil-windows-x86_64/truck_client.exe'
        $review = Join-Path $root 'target/native-review'
        New-Item -ItemType Directory -Force -Path $review | Out-Null
        & $binary --solo --smoke-frames 600 --smoke-drive `
            --data-dir (Join-Path $review 'userdata') `
            --report (Join-Path $review 'report.json') `
            --screenshot (Join-Path $review 'screenshot.png')
        if ($LASTEXITCODE -ne 0) { throw "Smoke gráfico falhou ($LASTEXITCODE)." }
        Write-Host "Inspecione a imagem e o relatório em $review. O smoke não substitui uma sessão manual."
    }
    Write-Host 'Validação concluída. Pacote portátil em dist/.'
} finally {
    $env:Path = $previousPath
    $env:CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS = $previousFlags
    $env:CARGO_BUILD_JOBS = $previousJobs
    Pop-Location
}
