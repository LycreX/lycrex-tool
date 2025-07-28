Add-Type -AssemblyName System.Windows.Forms

$code=@'
using System.Runtime.InteropServices;
public static class WinApi{
    [DllImport("user32.dll")]
    public static extern bool SetWindowPos(uint hWnd,uint hAfter,uint x,uint y,uint cx,uint cy,uint flags);
    [DllImport("kernel32.dll")]
    public static extern uint GetConsoleWindow();
    [DllImport("user32.dll")]
    public static extern int GetSystemMetrics(int nIndex);
    [DllImport("user32.dll")]
    public static extern int GetWindowLong(uint hWnd, int nIndex);
    [DllImport("user32.dll")]
    public static extern int SetWindowLong(uint hWnd, int nIndex, int dwNewLong);
}
'@

Add-Type -TypeDefinition $code

$hwnd = [WinApi]::GetConsoleWindow()
Write-Host "Window Handle: $hwnd"

$GWL_STYLE = -16
$WS_POPUP = 0x80000000
$newStyle = $WS_POPUP
[WinApi]::SetWindowLong($hwnd, $GWL_STYLE, $newStyle)

$screenWidth = [WinApi]::GetSystemMetrics(0)
$screenHeight = [WinApi]::GetSystemMetrics(1)
Write-Host "Screen Resolution: ${screenWidth}x${screenHeight}"

if ($screenWidth -eq 0 -or $screenHeight -eq 0) {
    $screenWidth = 1920
    $screenHeight = 1080
    Write-Host "Using default resolution: ${screenWidth}x${screenHeight}"
}

$windowWidth = 400
$windowHeight = 200

$x = [math]::Max(0, [math]::Floor(($screenWidth - $windowWidth) / 2))
$y = [math]::Max(0, [math]::Floor(($screenHeight - $windowHeight) / 2))

Write-Host "Calculated Position: ($x, $y)"
Write-Host "Window Size: ${windowWidth}x${windowHeight}"

$SWP_NOMOVE = 0x0002
$SWP_NOSIZE = 0x0001
$SWP_FRAMECHANGED = 0x0020
$SWP_SHOWWINDOW = 0x0040

$result = [WinApi]::SetWindowPos($hwnd, 0, $x, $y, $windowWidth, $windowHeight, $SWP_FRAMECHANGED -bor $SWP_SHOWWINDOW)

$host.UI.RawUI.WindowSize = $host.UI.RawUI.WindowSize
$host.UI.RawUI.BufferSize = $host.UI.RawUI.WindowSize

if ($result) {
    Write-Host "Window centered successfully!"
} else {
    Write-Host "Failed to center window!"
    $errorCode = [System.Runtime.InteropServices.Marshal]::GetLastWin32Error()
    Write-Host "Error code: $errorCode"
}

$host.UI.RawUI.BufferSize = $host.UI.RawUI.WindowSize
$host.UI.RawUI.BackgroundColor = 'Black'
$host.UI.RawUI.ForegroundColor = 'White'
$host.UI.RawUI.CursorPosition = @{X=0;Y=0}
Clear-Host

"Lycrex Tool"
Start-Sleep -Seconds 2
exit
Read-Host "Press Enter to exit"