#requires -Version 7.0
param([string]$Binary='target/apple-probe/debug/applifast-playback-probe.exe')
$ErrorActionPreference='Stop'
if(-not $IsWindows){throw 'The native memory-policy check requires Windows.'}
$root=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
Push-Location $root
$process=$null
$profile=$null
try {
    $binaryPath=(Resolve-Path -LiteralPath $Binary).Path
    $cache=Join-Path $root '.cache'
    New-Item -ItemType Directory -Force -Path $cache | Out-Null
    $profile=Join-Path $cache ('native-memory-policy-'+[guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $profile | Out-Null
    for($attempt=1;$attempt -le 2;$attempt++){
        $info=[Diagnostics.ProcessStartInfo]::new($binaryPath,'--self-check')
        $info.UseShellExecute=$false;$info.CreateNoWindow=$true
        $info.RedirectStandardInput=$true;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
        $info.Environment['LOCALAPPDATA']=$profile
        $process=[Diagnostics.Process]::Start($info)
        $stderr=$process.StandardError.ReadToEndAsync()
        $events=[Collections.Generic.List[object]]::new()
        function Read-Event([string]$kind){
            do {
                $pending=$process.StandardOutput.ReadLineAsync()
                if(-not $pending.Wait(45000)){throw 'Native memory-policy event timed out.'}
                $line=$pending.GetAwaiter().GetResult()
                if($null -eq $line){throw 'Native host exited before its expected event.'}
                $event=$line | ConvertFrom-Json
                $events.Add($event)
                if($event.type -eq 'error'){throw 'Native memory-policy check reported an error.'}
            }while($event.type -ne $kind)
            return $event
        }
        [void](Read-Event 'hostReady')
        foreach($command in @('show','hide','show','hide')){
            $process.StandardInput.WriteLine($command)
            [void](Read-Event 'memoryPolicy')
        }
        $process.StandardInput.WriteLine('{"type":"shutdown"}')
        $process.StandardInput.Close()
        if(-not $process.WaitForExit(10000)){throw 'Native memory-policy shutdown timed out.'}
        if($process.ExitCode -ne 0 -or $stderr.GetAwaiter().GetResult()){throw 'Native memory-policy process failed.'}
        $policies=@($events | Where-Object {$_.type -eq 'memoryPolicy'})
        if($policies.Count -ne 5){throw 'Expected hidden-active and repeated show/hide checks.'}
        $targets=@(1,0,1,0,1);$active=@($true,$true,$false,$true,$false);$shown=@($false,$true,$false,$true,$false)
        for($index=0;$index -lt 5;$index++){
            if($policies[$index].targetLevel -ne $targets[$index] -or $policies[$index].rendererVisible -ne $active[$index] -or $policies[$index].parentVisible -ne $shown[$index]){throw 'Unexpected native visibility or memory target.'}
        }
        [pscustomobject]@{Attempt=$attempt;Policies=$policies;Runtime=@($events | Where-Object {$_.type -eq 'runtime'})} | ConvertTo-Json -Depth 5 -Compress
        $process.Dispose();$process=$null
    }
    Write-Output 'Native memory-policy checks passed. No Apple navigation, credentials or audio.'
} finally {
    try {
    if($process){
        if(-not $process.HasExited){$process.Kill($true);$process.WaitForExit()}
        $process.Dispose()
    }
    if($profile -and (Test-Path -LiteralPath $profile)){
        $resolved=(Resolve-Path -LiteralPath $profile).Path
        if(-not $resolved.StartsWith($cache+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase) -or (Split-Path -Leaf $resolved) -notmatch '^native-memory-policy-[a-f0-9]{32}$'){throw 'Unsafe native test cleanup path.'}
        for($retry=0;;$retry++){
            $items=@(Get-Item -LiteralPath $resolved -Force)+@(Get-ChildItem -LiteralPath $resolved -Recurse -Force)
            if($items | Where-Object {$_.Attributes -band [IO.FileAttributes]::ReparsePoint}){throw 'Refusing native test reparse-point cleanup.'}
            try {Remove-Item -LiteralPath $resolved -Recurse -Force;break}
            catch {if($retry -ge 49){throw};Start-Sleep -Milliseconds 100}
        }
    }
    } finally { Pop-Location }
}
