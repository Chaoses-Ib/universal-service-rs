if (-not (Test-Path "$(Get-Location)\target\debug\examples\service.exe")) {
    Write-Error "need to build in unprivileged window"
    Exit 1
}

$name = "testservice"

Get-Service -Name $name -ErrorAction SilentlyContinue | Remove-Service
New-Service -Name $name -BinaryPathName "$(Get-Location)\target\debug\examples\service.exe"
Start-Service -Name $name
Start-Sleep -Milliseconds 500
Stop-Service -Name $name
Remove-Service -Name $name

Write-Output "> output"
Get-Content C:\Windows\Temp\test.txt