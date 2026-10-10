param([string]$Binary='target/debug/spotifast.exe')
$ErrorActionPreference='Stop'
$root=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$scratch=Join-Path $root ('.cache/bundled-preview-test-'+[Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $scratch | Out-Null
Push-Location $root
try {
    function Encode([byte[]]$bytes){[Convert]::ToBase64String($bytes).TrimEnd('=').Replace('+','-').Replace('/','_')}
    function DummyToken([long]$expiry,[string[]]$origin=@('https://applifast.invalid')){
        $header=Encode ([Text.Encoding]::UTF8.GetBytes('{"alg":"ES256","kid":"DUMMYKEY12"}'))
        $claims=@{iss='DUMMYTEAM1';iat=[DateTimeOffset]::UtcNow.ToUnixTimeSeconds()-20;exp=$expiry}
        if($origin){$claims.origin=@($origin)}
        $payload=Encode ([Text.Encoding]::UTF8.GetBytes(($claims | ConvertTo-Json -Compress)))
        $signature=Encode ([byte[]]::new(64))
        "$header.$payload.$signature"
    }
    function Reject([string]$file,[string]$label){
        $failed=$false
        try {& "$PSScriptRoot/applifast-preview.ps1" -Binary $Binary -DeveloperToken $file -OutputDirectory (Join-Path $scratch $label) *> $null}
        catch {$failed=$true}
        if(-not $failed){throw "Invalid $label package succeeded."}
        if(Get-ChildItem -LiteralPath $scratch -Filter '*.zip' -Recurse){throw 'Invalid input produced an archive.'}
    }
    $path=Join-Path $scratch 'developer-token.txt'
    $now=[DateTimeOffset]::UtcNow.ToUnixTimeSeconds()
    [IO.File]::WriteAllText($path,(DummyToken ($now-1)))
    Reject $path 'expired'
    [IO.File]::WriteAllText($path,(DummyToken ($now+3600) 'https://other.invalid'))
    Reject $path 'wrong-origin'
    [IO.File]::WriteAllText($path,(DummyToken ($now+3600) ''))
    Reject $path 'missing-origin'
    [IO.File]::WriteAllText($path,(DummyToken ($now+3600) @('https://applifast.invalid','https://other.invalid')))
    Reject $path 'extra-origin'
    [IO.File]::WriteAllText($path,('x'*32769))
    Reject $path 'oversized'
    [IO.File]::WriteAllText($path,'-----BEGIN PRIVATE KEY-----')
    Reject $path 'private-key-text'
    Reject (Join-Path $scratch 'missing.p8') 'signing-key-path'
    $token=DummyToken ($now+3600)
    [IO.File]::WriteAllText($path,$token)
    $failed=$false
    try {& "$PSScriptRoot/applifast-preview.ps1" -Binary $Binary -DeveloperToken $path -OutputDirectory 'packaging/unignored-token-test' *> $null}
    catch {$failed=$_.Exception.Message -like '*ignored directory*'}
    if(-not $failed -or (Test-Path -LiteralPath 'packaging/unignored-token-test')){throw 'Unignored package output was not rejected before writing.'}
    & "$PSScriptRoot/applifast-preview.ps1" -Binary $Binary -DeveloperToken $path -OutputDirectory (Join-Path $scratch 'valid') *> $null
    $zip=@(Get-ChildItem -LiteralPath (Join-Path $scratch 'valid') -Filter '*.zip')
    if($zip.Count -ne 1){throw 'Expected one valid fixture archive.'}
    $hash=(Get-FileHash -LiteralPath $zip[0].FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    if(-not (Get-Content -LiteralPath ($zip[0].FullName+'.sha256') -Raw).StartsWith($hash+'  ')){throw 'Checksum mismatch.'}
    $failed=$false
    try {& "$PSScriptRoot/applifast-preview.ps1" -Binary $Binary -DeveloperToken $path -OutputDirectory (Join-Path $scratch 'valid') *> $null}
    catch {$failed=$_.Exception.Message -like '*already exists*'}
    if(-not $failed -or (Get-FileHash -LiteralPath $zip[0].FullName).Hash.ToLowerInvariant() -ne $hash){throw 'Existing package was changed.'}
    $archive=[IO.Compression.ZipFile]::OpenRead($zip[0].FullName)
    try {
        if($archive.Entries.Count -ne 6){throw 'Unexpected archive size.'}
        $reader=[IO.StreamReader]::new($archive.GetEntry('developer-token.txt').Open())
        try {if($reader.ReadToEnd().Trim() -ne $token){throw 'Packaged app token changed.'}}
        finally {$reader.Dispose()}
        $reader=[IO.StreamReader]::new($archive.GetEntry('TESTING.md').Open())
        try {
            $guide=$reader.ReadToEnd()
            if($guide.Contains('(windows-save-evidence.md)') -or -not $guide.Contains('/docs/applifast/windows-save-evidence.md)')){throw 'Packaged save-evidence link is broken.'}
        } finally {$reader.Dispose()}
    } finally {$archive.Dispose()}
    if(Get-ChildItem -LiteralPath $scratch -Filter '.token-validation-*' -Recurse -Force){throw 'Token validation scratch was retained.'}
    Write-Output 'Preview package checks pass using dummy JWTs only; no Apple signature/access claim.'
} finally {
    Pop-Location
    if(-not $scratch.StartsWith((Join-Path $root '.cache/'),[StringComparison]::OrdinalIgnoreCase)){throw 'Unsafe scratch cleanup.'}
    $items=@(Get-Item -LiteralPath $scratch -Force)+@(Get-ChildItem -LiteralPath $scratch -Force -Recurse)
    if($items | Where-Object {$_.Attributes -band [IO.FileAttributes]::ReparsePoint}){throw 'Refusing scratch cleanup with reparse points.'}
    Remove-Item -LiteralPath $scratch -Recurse -Force
}
