@echo off
echo ========================================
echo    QuantumCoin Web Server Starting...
echo ========================================
echo.
echo Starting PowerShell web server...
echo Open your browser to: http://localhost:8000
echo.
powershell -Command "cd 'C:\Users\chris\quantumcoin-ui-'; Start-Process 'http://localhost:8000'; Add-Type -AssemblyName System.Web; $listener = New-Object System.Net.HttpListener; $listener.Prefixes.Add('http://localhost:8000/'); $listener.Start(); Write-Host 'QuantumCoin server running at http://localhost:8000'; while ($listener.IsListening) { $context = $listener.GetContext(); $request = $context.Request; $response = $context.Response; $path = $request.Url.LocalPath; if ($path -eq '/') { $path = '/index.html' }; $filepath = Join-Path $PWD $path.TrimStart('/'); if (Test-Path $filepath) { $content = [System.IO.File]::ReadAllBytes($filepath); $response.ContentLength64 = $content.Length; $response.OutputStream.Write($content, 0, $content.Length); } else { $response.StatusCode = 404; $notfound = [Text.Encoding]::UTF8.GetBytes('File not found'); $response.OutputStream.Write($notfound, 0, $notfound.Length); }; $response.Close(); }"
