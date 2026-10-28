const fs = require('node:fs');
const path = require('node:path');
const sharp = require('/Users/ls/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/sharp');

const destination = __dirname;
const colors = { ink: '#142C30', mint: '#25C7A4', dark: '#10272B', paper: '#F5F8F6' };

// 单一几何母版：回声的开放曲线和跃出的上行笔画共用固定坐标。
function mark(ink = colors.ink, accent = colors.mint) {
  return `<g fill="none" stroke-linecap="round" stroke-linejoin="round">
    <path d="M173 164C161 181 142 190 122 190C87 190 59 162 59 127C59 92 87 64 122 64C141 64 157 71 169 85" stroke="${ink}" stroke-width="25"/>
    <path d="M62 127H139" stroke="${ink}" stroke-width="25"/>
    <path d="M130 127L194 63M164 63H194V93" stroke="${accent}" stroke-width="25"/>
  </g>`;
}

function svg(width, height, content, title = 'EchoLeap') {
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 ${width} ${height}" role="img" aria-labelledby="title"><title id="title">${title}</title>${content}</svg>\n`;
}

function appIcon(theme) {
  const dark = theme === 'dark';
  const base = dark ? colors.dark : '#F4F8F5';
  const line = dark ? '#244044' : '#DFEAE3';
  return svg(1024, 1024,
    `<rect x="12" y="12" width="1000" height="1000" rx="224" fill="${base}"/>
    <rect x="13" y="13" width="998" height="998" rx="223" fill="none" stroke="${line}" stroke-width="2"/>
    <g transform="translate(-14 -2) scale(4.12)">${mark(dark ? '#F1FAF5' : colors.ink, dark ? '#65E9C0' : '#0AAE85')}</g>`, `EchoLeap ${dark ? '深色' : '浅色'}桌面图标`);
}

function lockup(theme) {
  const ink = theme === 'dark' ? '#F1FAF5' : colors.ink;
  const accent = theme === 'dark' ? '#65E9C0' : '#0AAE85';
  return svg(1110, 256, `${mark(ink, accent)}<text x="285" y="161" fill="${ink}" font-family="Avenir Next, Helvetica Neue, Arial, sans-serif" font-weight="600" font-size="127" letter-spacing="-5">EchoLeap</text>`, 'EchoLeap 横版标志');
}

async function main() {
  const files = {
    'echoleap-symbol.svg': svg(256, 256, mark()),
    'echoleap-symbol-mono.svg': svg(256, 256, mark('currentColor', 'currentColor')),
    'echoleap-logo-light.svg': lockup('light'),
    'echoleap-logo-dark.svg': lockup('dark'),
    'echoleap-icon-dark.svg': appIcon('dark'),
    'echoleap-icon-light.svg': appIcon('light'),
  };
  for (const [name, source] of Object.entries(files)) fs.writeFileSync(path.join(destination, name), source);
  for (const theme of ['dark', 'light']) {
    const source = Buffer.from(files[`echoleap-icon-${theme}.svg`]);
    for (const size of [1024, 512, 256, 128, 64, 32, 16]) {
      await sharp(source).resize(size, size).png().toFile(path.join(destination, `echoleap-icon-${theme}-${size}.png`));
    }
  }
  await sharp(Buffer.from(files['echoleap-logo-light.svg'])).resize(1665, 384).png().toFile(path.join(destination, 'echoleap-logo.png'));
  await sharp(Buffer.from(files['echoleap-symbol.svg'])).resize(1024, 1024).png().toFile(path.join(destination, 'echoleap-symbol.png'));
  console.log(JSON.stringify({ result: 'created', destination, masterSvgCount: Object.keys(files).length, iconSizes: [1024, 512, 256, 128, 64, 32, 16] }));
}
main().catch(error => { process.stderr.write(String(error) + '\n'); process.exitCode = 1; });
