param(
    [Parameter(Mandatory=$true)][string]$DeveloperToken,
    [string]$Binary='target/debug/spotifast.exe',
    [string]$TokenChecker='target/apple-probe/debug/applifast-playback-probe.exe',
    [string]$OutputDirectory='dist'
)
$ErrorActionPreference='Stop'
$root=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
Push-Location $root
$validationFile=$null
try {
    $output=[IO.Path]::GetFullPath($(if([IO.Path]::IsPathRooted($OutputDirectory)){$OutputDirectory}else{Join-Path $root $OutputDirectory}))
    $ignoredProbe=Join-Path $output '.applifast-package-output-check'
    & git check-ignore -q -- $ignoredProbe
    if($LASTEXITCODE -ne 0){throw 'Package output must be in an ignored directory, such as dist or .cache.'}
    if([IO.Path]::GetExtension($DeveloperToken) -ieq '.p8'){throw 'Package a signed developer token, never the .p8 signing key.'}
    if(-not (Test-Path -LiteralPath $Binary -PathType Leaf) -or -not (Test-Path -LiteralPath $TokenChecker -PathType Leaf)){throw 'Build the app and playback probe before packaging.'}
    $TokenChecker=(Resolve-Path -LiteralPath $TokenChecker).Path
    if((Get-Item -LiteralPath $Binary).VersionInfo.ProductName -ne 'Applifast'){throw 'Expected the Applifast Windows executable.'}
    $stream=[IO.File]::OpenRead((Resolve-Path -LiteralPath $DeveloperToken).Path)
    try {
        $reader=[IO.BinaryReader]::new($stream)
        $bytes=$reader.ReadBytes(32769)
    } finally {$stream.Dispose()}
    if($bytes.Length -gt 32768){throw 'Developer token file exceeds 32 KiB.'}
    $token=[Text.UTF8Encoding]::new($false,$true).GetString($bytes).Trim()
    New-Item -ItemType Directory -Path $output -Force | Out-Null
    $validationFile=Join-Path $output ('.token-validation-'+[Guid]::NewGuid().ToString('N')+'.txt')
    [IO.File]::WriteAllText($validationFile,$token,[Text.UTF8Encoding]::new($false))
    & $TokenChecker --check-token-file $validationFile
    if($LASTEXITCODE -ne 0){throw 'Developer token failed local checks; package not created.'}
    $payload=$token.Split('.')[1].Replace('-','+').Replace('_','/')
    $payload=$payload.PadRight($payload.Length+((4-$payload.Length%4)%4),'=')
    $claims=[Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($payload)) | ConvertFrom-Json
    if(@($claims.origin).Count -ne 1 -or @($claims.origin)[0] -ne 'https://applifast.invalid'){throw 'Packaged tokens must restrict their origin to https://applifast.invalid.'}
    $expires=[DateTimeOffset]::FromUnixTimeSeconds([long]$claims.exp).UtcDateTime.ToString('u')
    $source=(& git rev-parse HEAD).Trim()
    if($LASTEXITCODE -ne 0){throw 'Cannot identify package source.'}
    $dirty=[bool](& git status --porcelain)
    $name='applifast-preview-'+$source.Substring(0,7)+$(if($dirty){'-dirty'})
    $directory=Join-Path $output $name
    $zip=Join-Path $output ($name+'.zip')
    if((Test-Path -LiteralPath $directory) -or (Test-Path -LiteralPath $zip)){throw 'Package destination already exists.'}
    New-Item -ItemType Directory -Path $directory -Force | Out-Null
    Copy-Item -LiteralPath $Binary -Destination (Join-Path $directory 'Applifast.exe')
    Copy-Item -LiteralPath 'LICENSE' -Destination (Join-Path $directory 'LICENSE')
    Copy-Item -LiteralPath 'packaging/applifast-preview.txt' -Destination (Join-Path $directory 'README.txt')
    $guide=(Get-Content -LiteralPath 'docs/applifast/testing.md' -Raw).Replace('(listening-slice.md)',"(https://github.com/jackkayser2005/applifast/blob/$source/docs/applifast/listening-slice.md)").Replace('(windows-save-evidence.md)',"(https://github.com/jackkayser2005/applifast/blob/$source/docs/applifast/windows-save-evidence.md)")
    [IO.File]::WriteAllText((Join-Path $directory 'TESTING.md'),$guide,[Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText((Join-Path $directory 'developer-token.txt'),($token+"`n"),[Text.UTF8Encoding]::new($false))
    $exeHash=(Get-FileHash -LiteralPath (Join-Path $directory 'Applifast.exe') -Algorithm SHA256).Hash.ToLowerInvariant()
    $build="Source: $source`nDirty source: $dirty`nWindows x64 development preview, not a public release`nInherited version: spotifast 0.12.0`nApplifast.exe SHA256: $exeHash`nApp developer token expires (UTC): $expires`nLocal token checks cover format, dates and origin. Apple verifies signature and access.`nFresh-account authorization and runtime release acceptance remain required.`nEarlier intermittent Windows cache/session replacement failures remain undiagnosed.`n"
    [IO.File]::WriteAllText((Join-Path $directory 'BUILD.txt'),$build,[Text.UTF8Encoding]::new($false))
    $allowed=@('Applifast.exe','BUILD.txt','LICENSE','README.txt','TESTING.md','developer-token.txt')
    Compress-Archive -LiteralPath ($allowed | ForEach-Object {Join-Path $directory $_}) -DestinationPath $zip -CompressionLevel Optimal
    $archive=[IO.Compression.ZipFile]::OpenRead($zip)
    try {
        if((@($archive.Entries.FullName | Sort-Object) -join '|') -ne (($allowed | Sort-Object) -join '|')){throw 'Unexpected package entries.'}
    } finally {$archive.Dispose()}
    $zipHash=(Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash.ToLowerInvariant()
    [IO.File]::WriteAllText(($zip+'.sha256'),($zipHash+'  '+[IO.Path]::GetFileName($zip)+"`n"),[Text.UTF8Encoding]::new($false))
    Write-Output ("Created $zip`nSHA256: $zipHash`nApp token expires (UTC): $expires")
} finally {
    if($validationFile -and (Test-Path -LiteralPath $validationFile)){Remove-Item -LiteralPath $validationFile -Force}
    Pop-Location
}
