param(
    [Parameter(Mandatory=$true)][string]$SetupCompiler,
    [string]$Binary='target/debug/spotifast.exe'
)
$ErrorActionPreference='Stop'
$root=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$id=[Guid]::NewGuid().ToString('N')
$scratch=Join-Path $root ('.cache/windows-installer-test-'+$id)
$identity='Applifast.InstallerQA.'+$id
$registration='HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\'+$identity+'_is1'
$installed=Join-Path $scratch 'installed'
$uninstaller=Join-Path $installed 'unins000.exe'
$uninstalled=$false
$shortcutName='Applifast installer QA '+$identity+'.lnk'
$shortcuts=@(
    (Join-Path ([Environment]::GetFolderPath('Programs')) $shortcutName),
    (Join-Path ([Environment]::GetFolderPath('DesktopDirectory')) $shortcutName)
)
New-Item -ItemType Directory -Path $scratch | Out-Null
Push-Location $root
function RunSetup([string]$file,[string[]]$arguments){
    # Wait for the whole installer process tree, including self-removal helpers.
    $process=Start-Process -FilePath $file -ArgumentList $arguments -WindowStyle Hidden -Wait -PassThru
    if($process.ExitCode -ne 0){throw ('QA installer failed: '+$process.ExitCode)}
}
try {
    $SetupCompiler=(Resolve-Path -LiteralPath $SetupCompiler).Path
    $script=Get-Content -LiteralPath "$PSScriptRoot/applifast.iss" -Raw
    if($script -match '\[(Registry|UninstallDelete|Code)\]' -or $script.Contains('FCED1EA0-EBF5-4C32-BA3B-A3AD724BACC3')){throw 'Installer must not register links, delete app data or reuse upstream identity.'}
    function Encode([string]$text){[Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($text)).TrimEnd('=').Replace('+','-').Replace('/','_')}
    $header=Encode '{"alg":"ES256","kid":"DUMMYKEY12"}'
    $now=[DateTimeOffset]::UtcNow.ToUnixTimeSeconds()
    $claims=@{iss='DUMMYTEAM1';iat=$now-20;exp=$now+3600;origin=@('https://applifast.invalid')}
    $token=$header+'.'+(Encode ($claims|ConvertTo-Json -Compress))+'.'+([Convert]::ToBase64String([byte[]]::new(64)).TrimEnd('=').Replace('+','-').Replace('/','_'))
    $tokenFile=Join-Path $scratch 'developer-token.txt'
    [IO.File]::WriteAllText($tokenFile,$token)
    $missing=Join-Path $scratch 'missing-compiler.exe'
    $failed=$false
    try {& "$PSScriptRoot/applifast-preview.ps1" -Binary $Binary -DeveloperToken $tokenFile -OutputDirectory (Join-Path $scratch 'missing') -SetupCompiler $missing *> $null}
    catch {$failed=$true}
    if(-not $failed -or (Get-ChildItem -LiteralPath $scratch -Filter '*.zip' -Recurse)){throw 'Missing compiler produced a package.'}
    $armBinary=Join-Path $scratch 'wrong-architecture.exe'
    Copy-Item -LiteralPath $Binary -Destination $armBinary
    $pe=[IO.BinaryReader]::new([IO.File]::OpenRead($armBinary))
    try {$pe.BaseStream.Position=0x3c; $machine=$pe.ReadUInt32()+4} finally {$pe.Dispose()}
    $stream=[IO.File]::OpenWrite($armBinary)
    try {$stream.Position=$machine; $stream.WriteByte(0x64); $stream.WriteByte(0xaa)} finally {$stream.Dispose()}
    $failed=$false
    try {& "$PSScriptRoot/applifast-preview.ps1" -Binary $armBinary -DeveloperToken $tokenFile -OutputDirectory (Join-Path $scratch 'wrong-architecture') -SetupCompiler $SetupCompiler *> $null}
    catch {$failed=$_.Exception.Message -like '*Windows x64 executable*'}
    if(-not $failed -or (Get-ChildItem -LiteralPath $scratch -Filter '*.zip' -Recurse)){throw 'Wrong-architecture installer was accepted.'}
    $badCompiler=Join-Path $scratch 'failed-compiler.ps1'
    [IO.File]::WriteAllText($badCompiler,'exit 1')
    $failureOutput=Join-Path $scratch 'failed-compile'
    $failed=$false
    try {& "$PSScriptRoot/applifast-preview.ps1" -Binary $Binary -DeveloperToken $tokenFile -OutputDirectory $failureOutput -SetupCompiler $badCompiler *> $null}
    catch {$failed=$_.Exception.Message -like '*Installer compilation failed*'}
    if(-not $failed -or @(Get-ChildItem -LiteralPath $failureOutput -Filter '*.zip').Count -ne 1 -or (Get-ChildItem -LiteralPath $failureOutput -Filter '*-setup.exe')){throw 'Compiler failure did not preserve only the portable ZIP.'}
    & "$PSScriptRoot/applifast-preview.ps1" -Binary $Binary -DeveloperToken $tokenFile -OutputDirectory $scratch -SetupCompiler $SetupCompiler *> (Join-Path $scratch 'package.log')
    $setup=@(Get-ChildItem -LiteralPath $scratch -Filter '*-setup.exe')
    if($setup.Count -ne 1){throw 'Expected one preview installer.'}
    $hash=(Get-FileHash -LiteralPath $setup[0].FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    if(-not (Get-Content -LiteralPath ($setup[0].FullName+'.sha256') -Raw).StartsWith($hash+'  ')){throw 'Installer checksum mismatch.'}
    $existing=Join-Path $scratch 'existing-installer'
    New-Item -ItemType Directory -Path $existing | Out-Null
    $existingSetup=Join-Path $existing $setup[0].Name
    [IO.File]::WriteAllText($existingSetup,'preserve-existing-installer')
    $failed=$false
    try {& "$PSScriptRoot/applifast-preview.ps1" -Binary $Binary -DeveloperToken $tokenFile -OutputDirectory $existing -SetupCompiler $SetupCompiler *> $null}
    catch {$failed=$_.Exception.Message -like '*already exists*'}
    if(-not $failed -or (Get-Content -LiteralPath $existingSetup -Raw) -ne 'preserve-existing-installer' -or (Get-ChildItem -LiteralPath $existing -Filter '*.zip')){throw 'Existing installer was overwritten.'}
    $payload=@(Get-ChildItem -LiteralPath $scratch -Directory | Where-Object Name -like 'applifast-preview-*')[0].FullName
    $version=(Get-Item -LiteralPath $Binary).VersionInfo.FileVersion
    $arguments=@("/DPayloadDir=$payload","/DOutputDir=$scratch",'/DOutputName=qa',"/DVersion=$version",'/DSourceRevision=qa',"/DTestIdentity=$identity","$PSScriptRoot/applifast.iss")
    & $SetupCompiler @arguments *> (Join-Path $scratch 'compile.log')
    if($LASTEXITCODE -ne 0){throw 'QA installer did not compile.'}
    if(Test-Path -LiteralPath $registration){throw 'QA uninstall identity already exists.'}
    $settings=Join-Path $scratch 'settings.ron'
    [IO.File]::WriteAllText($settings,'preserve-settings')
    $installArgs=@('/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART','/TASKS=desktopicon',('/DIR="'+$installed+'"'))
    RunSetup (Join-Path $scratch 'qa-setup.exe') $installArgs
    $entry=Get-ItemProperty -LiteralPath $registration
    if($entry.DisplayName -notlike 'Applifast installer QA*' -or $entry.InstallLocation.TrimEnd('\') -ne $installed){throw 'Wrong QA installation identity or directory.'}
    $files=@('Applifast.exe','BUILD.txt','LICENSE','README.txt','TESTING.md','developer-token.txt')
    foreach($name in $files){
        if((Get-FileHash -LiteralPath (Join-Path $payload $name)).Hash -ne (Get-FileHash -LiteralPath (Join-Path $installed $name)).Hash){throw ('Installed file mismatch: '+$name)}
    }
    $shell=New-Object -ComObject WScript.Shell
    foreach($shortcut in $shortcuts){
        if(-not (Test-Path -LiteralPath $shortcut) -or $shell.CreateShortcut($shortcut).TargetPath -ne (Join-Path $installed 'Applifast.exe')){throw 'Start/desktop shortcut target is incorrect.'}
    }
    $personal=Join-Path $installed 'personal.txt'
    [IO.File]::WriteAllText($personal,'preserve-personal-file')
    [IO.File]::WriteAllText((Join-Path $payload 'README.txt'),'updated-preview-guide')
    $claims.exp=$now+7200
    $renewed=$header+'.'+(Encode ($claims|ConvertTo-Json -Compress))+'.'+([Convert]::ToBase64String([byte[]]::new(64)).TrimEnd('=').Replace('+','-').Replace('/','_'))
    [IO.File]::WriteAllText((Join-Path $payload 'developer-token.txt'),$renewed)
    & $SetupCompiler @arguments *> (Join-Path $scratch 'upgrade-compile.log')
    if($LASTEXITCODE -ne 0){throw 'QA upgrade did not compile.'}
    RunSetup (Join-Path $scratch 'qa-setup.exe') $installArgs
    if((Get-Content -LiteralPath (Join-Path $installed 'README.txt') -Raw) -ne 'updated-preview-guide' -or (Get-Content -LiteralPath (Join-Path $installed 'developer-token.txt') -Raw) -ne $renewed){throw 'Upgrade did not refresh guide and app token.'}
    RunSetup $uninstaller @('/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART')
    $uninstalled=$true
    foreach($name in $files){if(Test-Path -LiteralPath (Join-Path $installed $name)){throw 'Uninstall retained an owned payload file.'}}
    foreach($shortcut in $shortcuts){if(Test-Path -LiteralPath $shortcut){throw 'Uninstall retained a QA shortcut.'}}
    if(Test-Path -LiteralPath $registration){throw 'Uninstall retained the QA registration.'}
    if((Get-Content -LiteralPath $settings -Raw) -ne 'preserve-settings' -or (Get-Content -LiteralPath $personal -Raw) -ne 'preserve-personal-file'){throw 'Uninstall changed unrelated files.'}
    Write-Output 'Installer compile, six-file payload, checksum, per-user install, Start/desktop shortcuts, upgrade and uninstall checks passed. Dummy JWTs only; app never launched.'
} finally {
    Pop-Location
    if(-not $uninstalled -and (Test-Path -LiteralPath $uninstaller)){RunSetup $uninstaller @('/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART')}
    if(Test-Path -LiteralPath $registration){throw 'QA registration remains; preserve scratch for recovery.'}
    foreach($shortcut in $shortcuts){if(Test-Path -LiteralPath $shortcut){throw 'QA shortcut remains; preserve scratch for recovery.'}}
    if(-not [IO.Path]::GetFullPath($scratch).StartsWith((Join-Path $root '.cache/'),[StringComparison]::OrdinalIgnoreCase)){throw 'Unsafe installer QA cleanup.'}
    $items=@(Get-Item -LiteralPath $scratch -Force)+@(Get-ChildItem -LiteralPath $scratch -Recurse -Force)
    if($items | Where-Object {$_.Attributes -band [IO.FileAttributes]::ReparsePoint}){throw 'Refusing installer QA cleanup with reparse points.'}
    Remove-Item -LiteralPath $scratch -Recurse -Force
}
