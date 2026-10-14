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
if ([Environment]::OSVersion.Platform -eq [PlatformID]::Win32NT) {
    $fixture = $null
    try {
        $fixture = Start-Process -FilePath (Join-Path $PSHOME 'pwsh.exe') -ArgumentList @('-NoProfile', '-Command', 'Start-Sleep -Seconds 30') -WindowStyle Hidden -PassThru
        $report = (& "$PSScriptRoot/measure.ps1" -RootProcessId $fixture.Id -State paused -WarmupSeconds 0 -DurationSeconds 1 -IntervalSeconds 0.1) | ConvertFrom-Json
        Assert (-not $report.ResourceBudgetPassed) 'A short tooling smoke check must not pass the acceptance budget.'
        Assert (-not $report.PlaybackStateVerified) 'The sampler must not claim it verified playback.'
        Assert ($report.Failures -contains 'acceptance_duration_not_met') 'A short sample must retain the acceptance-duration failure.'
        Assert ($report.Samples.Count -gt 1) 'Capture real Windows process counters more than once.'
        $startTicks = $fixture.StartTime.ToUniversalTime().Ticks
        foreach ($sample in $report.Samples) {
            $members = @($sample.Processes)
            Assert ($members.Count -eq $sample.ProcessIds.Count) 'Every selected PID must have memory details, even when unreadable.'
            Assert (($members.ProcessId -join ',') -eq ($sample.ProcessIds -join ',')) 'Per-process details must match the selected tree in PID order.'
            Assert (($members.PrivateBytes | Measure-Object -Sum).Sum -eq $sample.PrivateBytes) 'Per-process private bytes must sum to the aggregate.'
            $rootDetails = @($members | Where-Object ProcessId -eq $fixture.Id)
            Assert ($rootDetails.Count -eq 1) 'Report the fixture root exactly once.'
            Assert ($rootDetails[0].StartTicks -eq $startTicks) 'Memory details must retain process identity against PID reuse.'
            Assert ($rootDetails[0].Name -eq 'pwsh') 'Report the executable name without its command line or path.'
            Assert ($rootDetails[0].PrivateBytes -gt 0) 'Report actual private memory for the Windows fixture.'
            Assert ($null -ne $rootDetails[0].ParentProcessId) 'Retain parent IDs for process coverage review.'
        }
    } finally {
        if ($null -ne $fixture) {
            if (-not $fixture.HasExited) { $fixture.Kill(); $fixture.WaitForExit() }
            $fixture.Dispose()
        }
    }
}
'Measurement self-check passed.'
