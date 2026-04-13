# Simple syntax test for install.ps1
try {
    $content = Get-Content "install.ps1" -Raw
    $scriptBlock = [scriptblock]::Create($content)
    Write-Host "✅ PowerShell syntax validation passed!" -ForegroundColor Green
} catch {
    Write-Host "❌ PowerShell syntax error:" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Yellow

    if ($_.Exception.Message -match "line:(\d+)") {
        $lineNumber = [int]$matches[1]
        Write-Host "Issue appears to be around line $lineNumber" -ForegroundColor Yellow
    }
}
