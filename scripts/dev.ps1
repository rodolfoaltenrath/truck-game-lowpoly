# Ciclo rápido: conserva perfil, flags e target/ usados pelo desenvolvimento.
[CmdletBinding()]
param([switch]$Check)
$ErrorActionPreference = 'Stop'
$mode = if ($Check) { 'Check' } else { 'Run' }
& (Join-Path $PSScriptRoot 'build-native.ps1') -Mode $mode
