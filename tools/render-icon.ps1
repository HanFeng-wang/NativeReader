$ErrorActionPreference = 'Stop'

Add-Type -AssemblyName System.Drawing

$root = Split-Path -Parent $PSScriptRoot
$assets = Join-Path $root 'assets'
$icoPath = Join-Path $assets 'native-reader.ico'
$previewPath = Join-Path $assets 'native-reader-256.png'
New-Item -ItemType Directory -Path $assets -Force | Out-Null

function New-IconBitmap([int]$size) {
    $bitmap = New-Object System.Drawing.Bitmap($size, $size, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $graphics.Clear([System.Drawing.Color]::Transparent)

    $scale = $size / 256.0
    $graphics.ScaleTransform($scale, $scale)

    $page = New-Object System.Drawing.Drawing2D.GraphicsPath
    $page.AddPolygon([System.Drawing.PointF[]]@(
        [System.Drawing.PointF]::new(64, 30),
        [System.Drawing.PointF]::new(154, 30),
        [System.Drawing.PointF]::new(202, 78),
        [System.Drawing.PointF]::new(202, 216),
        [System.Drawing.PointF]::new(202, 228),
        [System.Drawing.PointF]::new(190, 228),
        [System.Drawing.PointF]::new(64, 228),
        [System.Drawing.PointF]::new(52, 228),
        [System.Drawing.PointF]::new(52, 216),
        [System.Drawing.PointF]::new(52, 42),
        [System.Drawing.PointF]::new(52, 30),
        [System.Drawing.PointF]::new(64, 30)
    ))
    $graphics.FillPath((New-Object System.Drawing.SolidBrush([System.Drawing.ColorTranslator]::FromHtml('#FFFDF8'))), $page)
    $outline = New-Object System.Drawing.Pen([System.Drawing.ColorTranslator]::FromHtml('#24313A'), 12)
    $outline.LineJoin = [System.Drawing.Drawing2D.LineJoin]::Round
    $graphics.DrawPath($outline, $page)

    $fold = New-Object System.Drawing.Drawing2D.GraphicsPath
    $fold.AddPolygon([System.Drawing.PointF[]]@(
        [System.Drawing.PointF]::new(154, 30),
        [System.Drawing.PointF]::new(154, 66),
        [System.Drawing.PointF]::new(165, 78),
        [System.Drawing.PointF]::new(202, 78)
    ))
    $graphics.FillPath((New-Object System.Drawing.SolidBrush([System.Drawing.ColorTranslator]::FromHtml('#D9F3F0'))), $fold)
    $graphics.DrawPath($outline, $fold)

    $accent = New-Object System.Drawing.Pen([System.Drawing.ColorTranslator]::FromHtml('#0F766E'), 12)
    $accent.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $accent.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
    $accent.LineJoin = [System.Drawing.Drawing2D.LineJoin]::Round
    $graphics.DrawLines($accent, [System.Drawing.PointF[]]@(
        [System.Drawing.PointF]::new(94, 111),
        [System.Drawing.PointF]::new(75, 132),
        [System.Drawing.PointF]::new(94, 153)
    ))
    $graphics.DrawLines($accent, [System.Drawing.PointF[]]@(
        [System.Drawing.PointF]::new(162, 111),
        [System.Drawing.PointF]::new(181, 132),
        [System.Drawing.PointF]::new(162, 153)
    ))
    $graphics.DrawLine($accent, 143, 99, 114, 165)

    $rule = New-Object System.Drawing.Pen([System.Drawing.ColorTranslator]::FromHtml('#24313A'), 10)
    $rule.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $rule.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
    $graphics.DrawLine($rule, 92, 191, 164, 191)

    $rule.Dispose()
    $accent.Dispose()
    $outline.Dispose()
    $fold.Dispose()
    $page.Dispose()
    $graphics.Dispose()
    $bitmap
}

$sizes = @(16, 24, 32, 48, 64, 128, 256)
$images = @()
foreach ($size in $sizes) {
    $bitmap = New-IconBitmap $size
    $stream = New-Object System.IO.MemoryStream
    $bitmap.Save($stream, [System.Drawing.Imaging.ImageFormat]::Png)
    $images += ,@($size, $stream.ToArray())
    if ($size -eq 256) {
        $bitmap.Save($previewPath, [System.Drawing.Imaging.ImageFormat]::Png)
    }
    $stream.Dispose()
    $bitmap.Dispose()
}

$file = [System.IO.File]::Create($icoPath)
$writer = New-Object System.IO.BinaryWriter($file)
$writer.Write([UInt16]0)
$writer.Write([UInt16]1)
$writer.Write([UInt16]$images.Count)
$offset = 6 + 16 * $images.Count
foreach ($image in $images) {
    $size = [int]$image[0]
    $bytes = [byte[]]$image[1]
    $writer.Write([byte]$(if ($size -ge 256) { 0 } else { $size }))
    $writer.Write([byte]$(if ($size -ge 256) { 0 } else { $size }))
    $writer.Write([byte]0)
    $writer.Write([byte]0)
    $writer.Write([UInt16]1)
    $writer.Write([UInt16]32)
    $writer.Write([UInt32]$bytes.Length)
    $writer.Write([UInt32]$offset)
    $offset += $bytes.Length
}
foreach ($image in $images) {
    $writer.Write([byte[]]$image[1])
}
$writer.Dispose()
$file.Dispose()

Write-Host "Wrote $icoPath"
Write-Host "Wrote $previewPath"
