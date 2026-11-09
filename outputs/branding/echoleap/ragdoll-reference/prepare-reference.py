from pathlib import Path
from PIL import Image, ImageFilter, ImageEnhance

root = Path(__file__).resolve().parent
photo = Image.open(root / 'reference-head-crop.png').convert('RGB')
mask = Image.open(root / 'head-mask.png').convert('RGBA').getchannel('A')
mask = mask.filter(ImageFilter.GaussianBlur(.65))
head = photo.convert('RGBA')
head.putalpha(mask)
head = head.crop((40, 20, 510, 505))
head.save(root / 'reference-head-cutout.png')
canvas = Image.new('RGBA', (1024, 1024), '#8CD0F2')
head = head.resize((840, 867), Image.Resampling.LANCZOS)
canvas.alpha_composite(head, (92, 73))
canvas.save(root / 'reference-head-on-sky.png')
print('照片猫头轮廓已提取；保留实际五官和毛色，未替换为几何五官。')
