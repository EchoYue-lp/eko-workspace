const fs = require('node:fs');
const path = require('node:path');
const sharp = require('/Users/ls/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/sharp');
const dir = __dirname;

function symbol(color) {
  return `<path d="M178 155C161 178 130 186 103 176C76 166 61 142 67 118C73 94 97 81 119 87C140 92 153 109 143 123C134 136 110 141 86 135C113 155 170 130 196 80" fill="none" stroke="${color}" stroke-width="26" stroke-linecap="round" stroke-linejoin="round"/>`;
}
function svg(w,h,body,title='EchoLeap') {
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}" viewBox="0 0 ${w} ${h}" role="img" aria-labelledby="title"><title id="title">${title}</title>${body}</svg>\n`;
}
async function main() {
  const themes = [{name:'ink',color:'#292622',bg:'#F9F6F0'},{name:'clay',color:'#B66B52',bg:'#F9F0E8'}];
  for (const t of themes) {
    const mark = svg(256,256,symbol(t.color));
    const icon = svg(1024,1024,`<rect x="12" y="12" width="1000" height="1000" rx="224" fill="${t.bg}"/><g transform="translate(-28 -6) scale(4.2)">${symbol(t.color)}</g>`, 'EchoLeap 柔和曲线图标');
    const logo = svg(940,256,`${symbol(t.color)}<text x="272" y="159" fill="${t.color}" font-family="Avenir Next, Helvetica Neue, Arial, sans-serif" font-weight="500" font-size="116" letter-spacing="-4">EchoLeap</text>`, 'EchoLeap 柔和曲线字标');
    fs.writeFileSync(path.join(dir,`echoleap-soft-${t.name}-symbol.svg`),mark);
    fs.writeFileSync(path.join(dir,`echoleap-soft-${t.name}-icon.svg`),icon);
    fs.writeFileSync(path.join(dir,`echoleap-soft-${t.name}-logo.svg`),logo);
    for (const size of [1024,512,256,128,64,32,16]) {
      await sharp(Buffer.from(icon)).resize(size,size).png().toFile(path.join(dir,`echoleap-soft-${t.name}-${size}.png`));
    }
    await sharp(Buffer.from(logo)).resize(1410,384).png().toFile(path.join(dir,`echoleap-soft-${t.name}-logo.png`));
  }
  console.log('EchoLeap 新曲线方向已生成，保留原方案供对照。');
}
main().catch(e=>{process.stderr.write(String(e)+'\n');process.exitCode=1;});
