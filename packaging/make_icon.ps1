# Draws ContactSolutions's icon (the Craft family's blue tile with a solid white person) and writes
# assets\contactsolutions.ico (16, 24, 32, 48, 64, 128 and 256 px) and web\favicon.ico.
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
    $tile = RR (2*$k) (2*$k) (60*$k) (60*$k) (14*$k)
    $blueBrush = New-Object System.Drawing.SolidBrush $blue
    $g.FillPath($blueBrush, $tile)
    $tile.Dispose()
    $blueBrush.Dispose()
    $white = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::White)
    $g.FillEllipse($white, 20*$k, 10*$k, 24*$k, 24*$k)
    $body = New-Object System.Drawing.Drawing2D.GraphicsPath
    $body.AddBezier((12*$k), (52*$k), (15*$k), (41*$k), (22*$k), (36*$k), (32*$k), (36*$k))
    $body.AddBezier((32*$k), (36*$k), (42*$k), (36*$k), (49*$k), (41*$k), (52*$k), (52*$k))
    $body.AddLine((52*$k), (52*$k), (12*$k), (52*$k))
    $body.CloseFigure()
    $g.FillPath($white, $body)
    $body.Dispose()
    $white.Dispose()
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
[System.IO.File]::WriteAllBytes((Join-Path $root 'assets\contactsolutions.ico'), $ms.ToArray())
[System.IO.File]::WriteAllBytes((Join-Path $root 'web\favicon.ico'), $ms.ToArray())
[System.IO.File]::WriteAllBytes((Join-Path $root 'assets\contactsolutions-256.png'), $pngs[$sizes.Count - 1])
Write-Host 'Wrote assets\contactsolutions.ico'
