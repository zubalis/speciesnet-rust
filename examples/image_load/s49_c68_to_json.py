#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.9"
# dependencies = [
#     "pillow",
# ]
# ///

from PIL import Image, ImageFile
from json import dump

ImageFile.LOAD_TRUNCATED_IMAGES = True

input_file = 'S49-C68_IMG_0128.JPG'
image = Image.open(input_file)
    
# Convert to RGB if necessary
if image.mode == 'RGBA' or image.mode == 'L':
    image = image.convert(mode='RGB')

# Print the pixel values
data = []
for y in range(image.height):
    for x in range(image.width):
        r, g, b = image.getpixel((x, y))
        data.append({'x': x, 'y': y, 'r': r, 'g': g, 'b': b})


with open(input_file + '.json', 'w') as f:
    dump(data, f)
