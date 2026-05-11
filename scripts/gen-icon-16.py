"""
Generate crisp small icons from the 512x512 source icon.

- icon-16.png samples the center pixel of each logical LED cell.
- icon-32.png is a nearest-neighbor 2x upscale of that crisp 16x16 icon.
"""
from PIL import Image

SRC_PATH = "src-tauri/icons/icon.png"
DST_16_PATH = "public/icon-16.png"
DST_32_PATH = "public/icon-32.png"


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
print(f"Generated {DST_16_PATH} and {DST_32_PATH} from {w}x{h} source")
