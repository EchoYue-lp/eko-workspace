from pathlib import Path
import sys
import re
import xml.etree.ElementTree as ET
from PIL import Image, ImageFilter

root = Path(__file__).resolve().parent
sys.path.insert(0, str(root / '.tools'))
import vtracer

original = Image.open(root / 'reference-head-cutout.png').convert('RGBA')
for name, colors in [('portrait', 48), ('simplified', 24)]:
    alpha = original.getchannel('A')
    smooth = original.convert('RGB').filter(ImageFilter.MedianFilter(3)).filter(ImageFilter.GaussianBlur(.5))
    reduced = smooth.quantize(colors=colors, method=Image.Quantize.MEDIANCUT).convert('RGBA')
    reduced.putalpha(alpha.point(lambda a: 255 if a >= 128 else 0))
    input_file = root / f'trace-input-{name}.png'
    traced_file = root / f'traced-{name}.svg'
    reduced.save(input_file)
    vtracer.convert_image_to_svg_py(
        str(input_file), str(traced_file), colormode='color', hierarchical='stacked',
        mode='spline', filter_speckle=8, color_precision=6, layer_difference=8,
        corner_threshold=90, length_threshold=4, max_iterations=10,
        splice_threshold=45, path_precision=2,
    )
    tree = ET.parse(traced_file)
    svg = tree.getroot()
    body = ''.join(ET.tostring(child, encoding='unicode') for child in svg)
    body = re.sub(r'ns\d+:', '', body)
    body = re.sub(r' xmlns:ns\d+="[^"]*"', '', body)
    if '<image' in body or 'base64' in body:
        raise RuntimeError('最终矢量稿不得嵌入位图')
    result = f'''<svg xmlns="http://www.w3.org/2000/svg" width="1024" height="1024" viewBox="0 0 1024 1024" role="img" aria-labelledby="title desc">
<title id="title">EchoLeap 布偶猫头像 · 参考照片矢量描摹</title>
<desc id="desc">依据用户提供的布偶猫照片描摹猫头，保留略向右的姿态、深棕色重点毛、蓝眼睛、白鼻梁和蓬松脸颊；天蓝底色。猫头由矢量曲线路径组成，无嵌入照片。</desc>
<rect width="1024" height="1024" rx="224" fill="#8CD0F2"/>
<g transform="translate(92 73) scale(1.7872340426)">{body}</g>
</svg>\n'''
    output = root / f'echoleap-ragdoll-{name}.svg'
    output.write_text(result, encoding='utf-8')
    print(f'{name}: {len(svg)} 个矢量图层；{output.stat().st_size} 字节；无嵌入位图。')
