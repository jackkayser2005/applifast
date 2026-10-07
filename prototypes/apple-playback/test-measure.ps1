$ErrorActionPreference = 'Stop'
. "$PSScriptRoot/measure.ps1" -RootProcessId 10 -State paused
function Assert($condition, $message) { if (-not $condition) { throw $message } }
Assert ((Get-OverallCpuPercent 0.8 10 8) -eq 1) 'CPU must be normalized across all logical processors.'
Assert ((Get-OverallCpuPercent 0 10 8) -eq 0) 'Zero CPU must remain zero.'
$invalidRejected = $false
try { Get-OverallCpuPercent -1 10 8 } catch { $invalidRejected = $true }
Assert $invalidRejected 'Regressed CPU counters must fail.'
function Link($id, $parent, $start) {
    [pscustomobject]@{ ProcessId = $id; ParentProcessId = $parent; StartTicks = $start }
}
$tree = @(Link 10 1 100; Link 11 10 110; Link 12 11 120; Link 20 1 200)
$selected = @(Select-ProcessTree $tree 10 @{ 10 = 100 })
Assert (($selected.ProcessId | Sort-Object) -join ',' -eq '10,11,12') 'Include children and grandchildren, exclude unrelated processes.'
$reparented = @(Link 10 1 100; Link 11 1 110; Link 12 11 120)
$selected = @(Select-ProcessTree $reparented 10 @{ 10 = 100; 11 = 110 })
Assert (($selected.ProcessId | Sort-Object) -join ',' -eq '10,11,12') 'Retain surviving reparented children.'
$reused = @(Link 10 1 100; Link 11 1 999; Link 12 11 1000)
$selected = @(Select-ProcessTree $reused 10 @{ 10 = 100; 11 = 110 })
Assert (($selected.ProcessId | Sort-Object) -join ',' -eq '10') 'Reused PIDs must not inherit old membership.'
$reusedRoot = @(Link 10 1 999; Link 11 10 1000)
$selected = @(Select-ProcessTree $reusedRoot 10 @{ 10 = 100 })
Assert ($selected.Count -eq 0) 'A reused root must not match.'
'Measurement self-check passed.'
