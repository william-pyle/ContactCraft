# Developer helper: start ContactSolutions (demo contacts), photograph its window, close it.
# Usage: snap.ps1 <exe> <out.png> [extra args...]
param([string]$Exe, [string]$Out, [Parameter(ValueFromRemainingArguments = $true)][string[]]$Rest)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System; using System.Runtime.InteropServices;
public static class W {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  public delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc f, IntPtr l);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  public static IntPtr Biggest(uint pid) {
    IntPtr best = IntPtr.Zero; long area = 0;
    EnumWindows((h, l) => { uint p; GetWindowThreadProcessId(h, out p); RECT r;
      if (p == pid && IsWindowVisible(h) && GetWindowRect(h, out r)) { long a = (long)(r.R - r.L) * (r.B - r.T); if (a > area) { area = a; best = h; } }
      return true; }, IntPtr.Zero);
    return best;
  }
}
"@
[W]::SetProcessDPIAware() | Out-Null
$args2 = (@('--demo') + $Rest | ForEach-Object { if ($_ -match '\s') { '"' + $_ + '"' } else { $_ } }) -join ' '
$p = Start-Process -FilePath $Exe -ArgumentList $args2 -PassThru
try {
    $h = [IntPtr]::Zero
    for ($i = 0; $i -lt 40 -and $h -eq [IntPtr]::Zero; $i++) { Start-Sleep -Milliseconds 250; $p.Refresh(); $h = $p.MainWindowHandle }
    Start-Sleep -Seconds 4
    $h = [W]::Biggest([uint32]$p.Id)
    [W]::SetForegroundWindow($h) | Out-Null
    Start-Sleep -Milliseconds 800
    $r = New-Object W+RECT
    [W]::GetWindowRect($h, [ref]$r) | Out-Null
    $w = $r.R - $r.L; $hh = $r.B - $r.T
    $bmp = New-Object System.Drawing.Bitmap $w, $hh
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen($r.L, $r.T, 0, 0, $bmp.Size)
    $bmp.Save($Out, [System.Drawing.Imaging.ImageFormat]::Png)
    Write-Host "Saved $Out ($w x $hh)"
} finally {
    Stop-Process -Id $p.Id -ErrorAction SilentlyContinue
}
