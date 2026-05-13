"""
Generate crisp small icons from the 512x512 source icon.

- icon-16.png samples the center pixel of each logical LED cell.
- icon-32.png is a nearest-neighbor 2x upscale of that crisp 16x16 icon.
- Windows assets are also replaced with nearest-neighbor versions so
  native icons match the crisp PWA icon.
"""
from PIL import Image

SRC_PATH = "src-tauri/icons/icon.png"
DST_16_PATH = "public/icon-16.png"
DST_32_PATH = "public/icon-32.png"
NATIVE_ICO_PATH = "src-tauri/icons/icon.ico"
NATIVE_ICONS = [
    ("src-tauri/icons/32x32.png", 32),
    ("src-tauri/icons/64x64.png", 64),
    ("src-tauri/icons/StoreLogo.png", 50),
    ("src-tauri/icons/Square30x30Logo.png", 30),
    ("src-tauri/icons/Square44x44Logo.png", 44),
    ("src-tauri/icons/Square71x71Logo.png", 71),
    ("src-tauri/icons/Square89x89Logo.png", 89),
    ("src-tauri/icons/Square107x107Logo.png", 107),
    ("src-tauri/icons/Square142x142Logo.png", 142),
    ("src-tauri/icons/Square150x150Logo.png", 150),
    ("src-tauri/icons/Square284x284Logo.png", 284),
    ("src-tauri/icons/Square310x310Logo.png", 310),
]


src = Image.open(SRC_PATH).convert("RGB")
w, h = src.size
cell_w = w // 16
cell_h = h // 16

icon_16 = Image.new("RGB", (16, 16), (0, 0, 0))
for row in range(16):
    for col in range(16):
        cx = col * cell_w + cell_w // 2
        cy = row * cell_h + cell_h // 2
        r, g, b = src.getpixel((cx, cy))
        color = (r, g, b) if max(r, g, b) > 80 else (0, 0, 0)
        icon_16.putpixel((col, row), color)

icon_32 = icon_16.resize((32, 32), Image.Resampling.NEAREST)

icon_16.save(DST_16_PATH)
icon_32.save(DST_32_PATH)

for path, size in NATIVE_ICONS:
    icon_16.resize((size, size), Image.Resampling.NEAREST).save(path)

icon_16.resize((256, 256), Image.Resampling.NEAREST).save(
    NATIVE_ICO_PATH,
    sizes=[(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)],
)

print(f"Generated crisp small icons from {w}x{h} source")
