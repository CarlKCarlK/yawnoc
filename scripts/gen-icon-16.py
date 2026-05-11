"""
Generate a crisp 16x16 icon by sampling the center pixel of each cell
from the 512x512 source icon (which is a 16x16 LED grid, 32px per cell).
Each cell either gets its exact center color or pure black — no blending.
"""
from PIL import Image
import sys

src_path = "src-tauri/icons/icon.png"
dst_path = "public/icon-16.png"

src = Image.open(src_path).convert("RGB")
w, h = src.size
cell_w = w // 16
cell_h = h // 16

out = Image.new("RGB", (16, 16), (0, 0, 0))
for row in range(16):
    for col in range(16):
        cx = col * cell_w + cell_w // 2
        cy = row * cell_h + cell_h // 2
        r, g, b = src.getpixel((cx, cy))
        out.putpixel((col, row), (r, g, b) if max(r, g, b) > 80 else (0, 0, 0))

out.save(dst_path)
print(f"Generated {dst_path} from {w}x{h} source ({cell_w}x{cell_h}px cells)")
