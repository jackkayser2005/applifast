#requires -Version 7.0
$ErrorActionPreference='Stop'
if(-not $IsWindows){throw 'The native browser-profile check requires Windows.'}
$root=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$previous=$env:APPLIFAST_PROFILE_CLEAR_TEST_DIR
$profile=$null
Push-Location $root
try {
    $cache=Join-Path $root '.cache'
    New-Item -ItemType Directory -Force -Path $cache | Out-Null
    $profile=Join-Path $cache ('native-profile-clear-'+[guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $profile | Out-Null
    $env:APPLIFAST_PROFILE_CLEAR_TEST_DIR=$profile
    & cargo test --locked --manifest-path prototypes/apple-playback/Cargo.toml windows::tests::native_profile_cookie_clear_round_trip -- --ignored --exact --nocapture
    if($LASTEXITCODE -ne 0){throw 'Native browser-profile cleanup check failed.'}
} finally {
    try {
        if($profile -and (Test-Path -LiteralPath $profile)){
            $resolved=(Resolve-Path -LiteralPath $profile).Path
            if(-not $resolved.StartsWith($cache+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase) -or (Split-Path -Leaf $resolved) -notmatch '^native-profile-clear-[a-f0-9]{32}$'){throw 'Unsafe native profile cleanup path.'}
            for($retry=0;;$retry++){
                $items=@(Get-Item -LiteralPath $resolved -Force)+@(Get-ChildItem -LiteralPath $resolved -Recurse -Force)
                if($items | Where-Object {$_.Attributes -band [IO.FileAttributes]::ReparsePoint}){throw 'Refusing native profile reparse-point cleanup.'}
                try {Remove-Item -LiteralPath $resolved -Recurse -Force;break}
                catch {if($retry -ge 49){throw};Start-Sleep -Milliseconds 100}
            }
        }
    } finally {
        $env:APPLIFAST_PROFILE_CLEAR_TEST_DIR=$previous
        Pop-Location
    }
}
