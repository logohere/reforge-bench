param([string]$Package = "$PSScriptRoot/../dist/Reforge Bench")
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes

function Wait-Until($Description, [scriptblock]$Check) {
    $deadline = [DateTime]::UtcNow.AddSeconds(45)
    do {
        $value = & $Check
        if ($value) { return $value }
        Start-Sleep -Milliseconds 200
    } while ([DateTime]::UtcNow -lt $deadline)
    throw "Timed out: $Description"
}

$script:Sequence = 0
function Send-CDP($Method, $Params = @{}) {
    $id = ++$script:Sequence
    $bytes = [Text.Encoding]::UTF8.GetBytes((@{id=$id; method=$Method; params=$Params} | ConvertTo-Json -Depth 20 -Compress))
    $timeout = [Threading.CancellationTokenSource]::new(10000)
    try {
        $segment = [ArraySegment[byte]]::new($bytes)
        $socket.SendAsync($segment, [Net.WebSockets.WebSocketMessageType]::Text, $true, $timeout.Token).GetAwaiter().GetResult()
        do {
            $message = [IO.MemoryStream]::new()
            try {
                do {
                    $buffer = [byte[]]::new(65536)
                    $received = $socket.ReceiveAsync([ArraySegment[byte]]::new($buffer), $timeout.Token).GetAwaiter().GetResult()
                    if ($received.MessageType -eq [Net.WebSockets.WebSocketMessageType]::Close) { throw 'WebView2 debugger closed' }
                    $message.Write($buffer, 0, $received.Count)
                } until ($received.EndOfMessage)
                $response = [Text.Encoding]::UTF8.GetString($message.ToArray()) | ConvertFrom-Json
            } finally { $message.Dispose() }
        } until ($response.id -eq $id)
        if ($response.error) { throw ($response.error | ConvertTo-Json -Compress) }
        return $response.result
    } finally { $timeout.Dispose() }
}

function Evaluate($Expression) {
    $result = Send-CDP 'Runtime.evaluate' @{expression=$Expression; returnByValue=$true; awaitPromise=$true}
    if ($result.exceptionDetails) { throw ($result.exceptionDetails | ConvertTo-Json -Depth 10 -Compress) }
    return $result.result.value
}

$binary = (Resolve-Path "$Package/reforge-bench-desktop.exe").Path
$sandbox = Join-Path ([IO.Path]::GetTempPath()) ('reforge-windows-' + [Guid]::NewGuid())
New-Item -ItemType Directory -Path $sandbox | Out-Null
$previousData = $env:LOCALAPPDATA
$previousArguments = $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
$listener = [Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback, 0)
$listener.Start()
$port = $listener.LocalEndpoint.Port
$listener.Stop()
$socket = [Net.WebSockets.ClientWebSocket]::new()
$process = $null
try {
    $env:LOCALAPPDATA = $sandbox
    $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=$port"
    $process = Start-Process $binary -ArgumentList '--hardware' -WorkingDirectory $sandbox -PassThru -RedirectStandardOutput "$sandbox/stdout.txt" -RedirectStandardError "$sandbox/stderr.txt"
    $window = Wait-Until 'native window' {
        $process.Refresh()
        if ($process.HasExited) { throw "Desktop exited with code $($process.ExitCode)" }
        if ($process.MainWindowHandle -ne 0) {
            [System.Windows.Automation.AutomationElement]::FromHandle($process.MainWindowHandle)
        }
    }
    $button = Wait-Until 'native Hardware navigation' {
        $window = [System.Windows.Automation.AutomationElement]::FromHandle($process.MainWindowHandle)
        $condition = [System.Windows.Automation.PropertyCondition]::new(
            [System.Windows.Automation.AutomationElement]::AutomationIdProperty, 'nav-hardware')
        $window.FindFirst([System.Windows.Automation.TreeScope]::Descendants, $condition)
    }
    $button.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
    $target = Wait-Until 'embedded hardware page' {
        try {
            @(Invoke-RestMethod "http://127.0.0.1:$port/json/list" -TimeoutSec 2) |
                Where-Object { $_.type -eq 'page' -and $_.url.StartsWith('https://reforge.localhost/hardware/') } |
                Select-Object -First 1
        } catch { $null }
    }
    $timeout = [Threading.CancellationTokenSource]::new(10000)
    try { $socket.ConnectAsync([Uri]$target.webSocketDebuggerUrl, $timeout.Token).GetAwaiter().GetResult() }
    finally { $timeout.Dispose() }
    Wait-Until 'packaged scripts and 2D hardware plan' {
        Evaluate 'document.body.dataset.router==="ready" && document.querySelector(".hardware-tabletop")?.dataset.mode==="2d" && !!document.querySelector(".hardware-tabletop svg")'
    } | Out-Null
    if (!(Evaluate "(async()=>{const r=await fetch('/api/desktop/health');const h=await r.json();return r.ok&&h.ok&&h.transport==='in_process'})()")) {
        throw 'Packaged in-process API failed'
    }
    $output = if ($env:RUNNER_TEMP) { Join-Path $env:RUNNER_TEMP 'reforge-windows-ui' } else { Join-Path $sandbox 'ui' }
    New-Item -ItemType Directory -Path $output -Force | Out-Null
    $image = Send-CDP 'Page.captureScreenshot' @{format='png'}
    [IO.File]::WriteAllBytes("$output/hardware-2d.png", [Convert]::FromBase64String($image.data))
    Evaluate '[...document.querySelectorAll("cds-content-switcher-item")].find(el=>el.value==="3d").click()' | Out-Null
    Wait-Until '3D hardware canvas' {
        Evaluate 'document.querySelector(".hardware-tabletop")?.dataset.mode==="3d" && ["webgpu","webgl2"].includes(document.querySelector(".hardware-tabletop").dataset.graphics) && !!document.querySelector(".hardware-tabletop canvas")'
    } | Out-Null
    $image = Send-CDP 'Page.captureScreenshot' @{format='png'}
    [IO.File]::WriteAllBytes("$output/hardware-3d.png", [Convert]::FromBase64String($image.data))
    if (!(Evaluate "(()=>{try{localStorage.setItem('reforge.windows.smoke','retained');return localStorage.getItem('reforge.windows.smoke')==='retained'}catch{return false}})()")) {
        throw 'WebView2 local storage failed'
    }
    Write-Output 'Packaged Windows app: native command, embedded 2D/3D views, local API and storage passed.'
 } catch {
    if ($window) {
        $all = $window.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)
        for ($i = 0; $i -lt [Math]::Min($all.Count, 100); $i++) {
            $item = $all.Item($i).Current
            Write-Output ("UIA: {0} | {1}" -f $item.AutomationId, $item.Name)
        }
    }
    throw
} finally {
    $socket.Dispose()
    if ($process -and !$process.HasExited) {
        & taskkill /PID $process.Id /T /F | Out-Null
    }
    $env:LOCALAPPDATA = $previousData
    $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = $previousArguments
}
