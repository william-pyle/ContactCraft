# Draws ContactCraft's icon (the Craft family's blue tile with a white contact card) and writes
# assets\contactcraft.ico (16, 24, 32, 48, 64, 128 and 256 px) and web\favicon.ico.
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
$root = Split-Path $PSScriptRoot -Parent

function Draw([int]$n) {
    $bmp = New-Object System.Drawing.Bitmap $n, $n, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = 'AntiAlias'
    $g.Clear([System.Drawing.Color]::Transparent)
    $k = $n / 64.0
    function RR($x, $y, $w, $h, $r) {
        $p = New-Object System.Drawing.Drawing2D.GraphicsPath
        $d = 2 * $r
        $p.AddArc($x, $y, $d, $d, 180, 90); $p.AddArc($x + $w - $d, $y, $d, $d, 270, 90)
        $p.AddArc($x + $w - $d, $y + $h - $d, $d, $d, 0, 90); $p.AddArc($x, $y + $h - $d, $d, $d, 90, 90)
        $p.CloseFigure(); return $p
    }
    $blue = [System.Drawing.Color]::FromArgb(255, 0x0F, 0x68, 0xF0)
    $g.FillPath((New-Object System.Drawing.SolidBrush $blue), (RR (2*$k) (2*$k) (60*$k) (60*$k) (14*$k)))
    $white = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::White)
    $g.FillPath($white, (RR (11*$k) (16*$k) (42*$k) (32*$k) (6*$k)))
    $b = New-Object System.Drawing.SolidBrush $blue
    $g.FillEllipse($b, 17*$k, 21*$k, 11*$k, 11*$k)
    $g.FillPie($b, 14*$k, 33*$k, 17*$k, 18*$k, 180, 180)
    $g.FillPath($b, (RR (34*$k) (24*$k) (14*$k) (3.5*$k) (1.5*$k)))
    $g.FillPath($b, (RR (34*$k) (31*$k) (10*$k) (3.5*$k) (1.5*$k)))
    $g.FillPath($b, (RR (34*$k) (38*$k) (12*$k) (3.5*$k) (1.5*$k)))
    $g.Dispose()
    $ms = New-Object System.IO.MemoryStream
    $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
    return ,$ms.ToArray()
}

$sizes = 16, 24, 32, 48, 64, 128, 256
$pngs = @(); foreach ($s in $sizes) { $pngs += ,(Draw $s) }
$ms = New-Object System.IO.MemoryStream
$w = New-Object System.IO.BinaryWriter $ms
$w.Write([UInt16]0); $w.Write([UInt16]1); $w.Write([UInt16]$sizes.Count)
$offset = 6 + 16 * $sizes.Count
for ($i = 0; $i -lt $sizes.Count; $i++) {
    $s = $sizes[$i]; $len = $pngs[$i].Length
    $w.Write([byte]($s % 256)); $w.Write([byte]($s % 256)); $w.Write([byte]0); $w.Write([byte]0)
    $w.Write([UInt16]1); $w.Write([UInt16]32); $w.Write([UInt32]$len); $w.Write([UInt32]$offset)
    $offset += $len
}
foreach ($p in $pngs) { $w.Write($p) }
$w.Flush()
New-Item -ItemType Directory -Force (Join-Path $root 'assets'), (Join-Path $root 'web') | Out-Null
[System.IO.File]::WriteAllBytes((Join-Path $root 'assets\contactcraft.ico'), $ms.ToArray())
[System.IO.File]::WriteAllBytes((Join-Path $root 'web\favicon.ico'), $ms.ToArray())
[System.IO.File]::WriteAllBytes((Join-Path $root 'assets\contactcraft-256.png'), $pngs[$sizes.Count - 1])
Write-Host 'Wrote assets\contactcraft.ico'
