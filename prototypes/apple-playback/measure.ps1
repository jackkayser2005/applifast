[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidateRange(1, 2147483647)][int]$RootProcessId,
    [Parameter(Mandatory)][ValidateSet('playing', 'paused')][string]$State,
    [ValidateRange(0, 86400)][int]$WarmupSeconds = 120,
    [ValidateRange(1, 86400)][int]$DurationSeconds = 300,
    [ValidateRange(0.1, 60)][double]$IntervalSeconds = 1,
    [string]$OutputPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# Retain surviving reparented children, but never retain a reused numeric PID.
function Select-ProcessTree {
    param([object[]]$Inventory, [int]$RootId, [hashtable]$Known)
    $selected = @{}
    foreach ($entry in $Inventory) {
        if ($entry.ProcessId -eq $RootId -or
            ($Known.ContainsKey($entry.ProcessId) -and
             $Known[$entry.ProcessId] -eq $entry.StartTicks)) {
            if (-not $Known.ContainsKey($entry.ProcessId) -or
                $Known[$entry.ProcessId] -eq $entry.StartTicks) {
                $selected[$entry.ProcessId] = $entry
            }
        }
    }
    do {
        $added = $false
        foreach ($entry in $Inventory) {
            if (-not $selected.ContainsKey($entry.ProcessId) -and
                $selected.ContainsKey($entry.ParentProcessId)) {
                $selected[$entry.ProcessId] = $entry
                $added = $true
            }
        }
    } while ($added)
    return @($selected.Values)
}

function Get-OverallCpuPercent {
    param([double]$CpuSeconds, [double]$ElapsedSeconds, [int]$LogicalProcessors)
    if ($CpuSeconds -lt 0 -or $ElapsedSeconds -le 0 -or $LogicalProcessors -le 0) {
        throw 'Invalid CPU counters or elapsed interval.'
    }
    return 100 * $CpuSeconds / ($ElapsedSeconds * $LogicalProcessors)
}

# Dot sourcing makes the two pure functions available to the focused self-check.
if ($MyInvocation.InvocationName -eq '.') { return }
if ([Environment]::OSVersion.Platform -ne [PlatformID]::Win32NT) {
    throw 'This sampler requires Windows.'
}

if (-not ('ApplifastProcessSnapshot' -as [type])) {
    Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Runtime.InteropServices;
public static class ApplifastProcessSnapshot {
    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    private struct Entry {
        public uint Size, Usage, ProcessId;
        public UIntPtr DefaultHeap;
        public uint ModuleId, Threads, ParentProcessId;
        public int BasePriority;
        public uint Flags;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 260)] public string Exe;
    }
    public struct ProcessLink { public int ProcessId, ParentProcessId; }
    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern IntPtr CreateToolhelp32Snapshot(uint flags, uint id);
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern bool Process32FirstW(IntPtr snapshot, ref Entry entry);
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern bool Process32NextW(IntPtr snapshot, ref Entry entry);
    [DllImport("kernel32.dll")] private static extern bool CloseHandle(IntPtr handle);
    public static ProcessLink[] Read() {
        IntPtr handle = CreateToolhelp32Snapshot(2, 0);
        if (handle == new IntPtr(-1)) throw new Win32Exception();
        try {
            Entry entry = new Entry { Size = (uint)Marshal.SizeOf(typeof(Entry)) };
            if (!Process32FirstW(handle, ref entry)) throw new Win32Exception();
            var links = new List<ProcessLink>();
            do {
                links.Add(new ProcessLink { ProcessId = (int)entry.ProcessId,
                    ParentProcessId = (int)entry.ParentProcessId });
            } while (Process32NextW(handle, ref entry));
            int error = Marshal.GetLastWin32Error();
            if (error != 18) throw new Win32Exception(error);
            return links.ToArray();
        } finally { CloseHandle(handle); }
    }
}
'@
}

function Read-Inventory {
    foreach ($link in [ApplifastProcessSnapshot]::Read()) {
        $process = $null
        $ticks = $null
        $cpu = $null
        $memory = $null
        try {
            $process = [Diagnostics.Process]::GetProcessById($link.ProcessId)
            $ticks = $process.StartTime.ToUniversalTime().Ticks
            $cpu = $process.TotalProcessorTime.TotalSeconds
            $memory = $process.PrivateMemorySize64
        } catch {
            # An unreadable unrelated system process is harmless. Selected ones fail.
        } finally {
            if ($null -ne $process) { $process.Dispose() }
        }
        [pscustomobject]@{
            ProcessId = $link.ProcessId
            ParentProcessId = $link.ParentProcessId
            StartTicks = $ticks
            CpuSeconds = $cpu
            PrivateBytes = $memory
        }
    }
}

$root = [Diagnostics.Process]::GetProcessById($RootProcessId)
try { $rootStart = $root.StartTime.ToUniversalTime().Ticks } finally { $root.Dispose() }
$known = @{ $RootProcessId = $rootStart }
$failures = [Collections.Generic.HashSet[string]]::new()
$samples = [Collections.Generic.List[object]]::new()
$clock = [Diagnostics.Stopwatch]::StartNew()

# Check identity during warmup as well as measurement; an exited root cannot pass.
while ($clock.Elapsed.TotalSeconds -lt $WarmupSeconds) {
    $root = $null
    try {
        $root = [Diagnostics.Process]::GetProcessById($RootProcessId)
        if ($root.StartTime.ToUniversalTime().Ticks -ne $rootStart -or $root.HasExited) {
            throw 'Root identity changed.'
        }
    } catch { throw 'Root process exited during warmup.' }
    finally { if ($null -ne $root) { $root.Dispose() } }
    Start-Sleep -Milliseconds ([int][Math]::Min(1000, $IntervalSeconds * 1000))
}

$clock.Restart()
$previous = @{}
$previousElapsed = $null
$previousUtc = [DateTime]::UtcNow.Ticks
$totalCpu = 0.0
$maxMemory = 0L
$measurementStart = $null
$lastSampleElapsed = 0.0
while ($true) {
    $elapsed = $clock.Elapsed.TotalSeconds
    try {
        $inventory = @(Read-Inventory)
        $selected = @(Select-ProcessTree $inventory $RootProcessId $known)
    } catch {
        [void]$failures.Add('inventory_failed')
        break
    }
    $nowUtc = [DateTime]::UtcNow.Ticks
    $elapsed = $clock.Elapsed.TotalSeconds
    if ($null -eq $measurementStart) { $measurementStart = $elapsed }
    $current = @{}
    $memory = 0L
    $cpuDelta = 0.0
    foreach ($entry in $selected) {
        if ($null -eq $entry.StartTicks -or $null -eq $entry.CpuSeconds -or
            $null -eq $entry.PrivateBytes) {
            [void]$failures.Add('selected_process_unreadable')
            continue
        }
        $identity = '{0}:{1}' -f $entry.ProcessId, $entry.StartTicks
        $current[$identity] = $entry
        $known[$entry.ProcessId] = $entry.StartTicks
        $memory += $entry.PrivateBytes
        if ($null -ne $previousElapsed) {
            if ($previous.ContainsKey($identity)) {
                $delta = $entry.CpuSeconds - $previous[$identity].CpuSeconds
                if ($delta -lt 0) { [void]$failures.Add('cpu_counter_regressed') }
                else { $cpuDelta += $delta }
            } elseif ($entry.StartTicks -ge $previousUtc) {
                $cpuDelta += $entry.CpuSeconds
            } else {
                [void]$failures.Add('new_process_prior_cpu_unknown')
            }
        }
    }
    $rootIdentity = '{0}:{1}' -f $RootProcessId, $rootStart
    if (-not $current.ContainsKey($rootIdentity)) {
        [void]$failures.Add('root_exited_or_unreadable')
        break
    }
    $interval = 0.0
    $cpuPercent = 0.0
    if ($null -ne $previousElapsed) {
        $interval = $elapsed - $previousElapsed
        if ($interval -gt ($IntervalSeconds * 2)) { [void]$failures.Add('sampling_gap') }
        foreach ($identity in $previous.Keys) {
            if (-not $current.ContainsKey($identity)) {
                # Polling cannot recover CPU used between the last read and exit.
                [void]$failures.Add('departed_process_final_cpu_unknown')
            }
        }
        $cpuPercent = Get-OverallCpuPercent $cpuDelta $interval ([Environment]::ProcessorCount)
        $totalCpu += $cpuDelta
    }
    $maxMemory = [Math]::Max($maxMemory, $memory)
    $samples.Add([pscustomobject]@{
        ElapsedSeconds = [Math]::Round($elapsed, 3)
        IntervalSeconds = [Math]::Round($interval, 3)
        PrivateBytes = $memory
        OverallCpuPercent = [Math]::Round($cpuPercent, 4)
        ProcessIds = @($selected.ProcessId | Sort-Object)
    })
    $lastSampleElapsed = $elapsed
    if (($elapsed - $measurementStart) -ge $DurationSeconds) { break }
    $previous = $current
    $previousElapsed = $elapsed
    $previousUtc = $nowUtc
    Start-Sleep -Milliseconds ([int]($IntervalSeconds * 1000))
}

$measuredSeconds = 0.0
if ($samples.Count -gt 1) {
    $measuredSeconds = $lastSampleElapsed - $measurementStart
}
if ($WarmupSeconds -lt 120 -or $measuredSeconds -lt 300) {
    [void]$failures.Add('acceptance_duration_not_met')
}
$averageCpu = $null
if ($measuredSeconds -gt 0) {
    $averageCpu = Get-OverallCpuPercent $totalCpu $measuredSeconds ([Environment]::ProcessorCount)
}
$cpuLimit = if ($State -eq 'playing') { 1.0 } else { 0.2 }
$result = [ordered]@{
    StateTag = $State
    PlaybackStateVerified = $false
    RootProcessId = $RootProcessId
    LogicalProcessors = [Environment]::ProcessorCount
    WarmupSeconds = $WarmupSeconds
    MeasuredSeconds = [Math]::Round($measuredSeconds, 3)
    IntervalSeconds = $IntervalSeconds
    MaxAggregatePrivateMiB = [Math]::Round($maxMemory / 1MB, 3)
    AverageOverallCpuPercent = $averageCpu
    AccountingComplete = ($failures.Count -eq 0)
    SamplingLimitation = 'Processes created and exited entirely between snapshots are unobservable. Verify transient helper coverage separately.'
    ResourceBudgetPassed = ($failures.Count -eq 0 -and $maxMemory -le 250MB -and
        $null -ne $averageCpu -and $averageCpu -lt $cpuLimit)
    Failures = @($failures | Sort-Object)
    Samples = @($samples.ToArray())
}
$json = $result | ConvertTo-Json -Depth 6
if ($OutputPath) { [IO.File]::WriteAllText($ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($OutputPath), $json) }
$json
