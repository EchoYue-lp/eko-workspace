const fs = require('node:fs');
const path = require('node:path');
const sharp = require('/Users/ls/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/sharp');
async function main() {
  const source = fs.readFileSync(path.join(__dirname, 'echoleap-ragdoll-cartoon.svg'));
  if (source.includes('<image') || source.includes('base64')) throw new Error('卡通矢量稿不得嵌入照片');
  for (const size of [1024, 512, 256, 128, 64]) {
    await sharp(source).resize(size,size).png().toFile(path.join(__dirname,`echoleap-ragdoll-cartoon-${size}.png`));
  }
  console.log('参考照片的矢量卡通猫头已导出；无嵌入照片，天蓝底。');
}
main().catch(error=>{process.stderr.write(String(error)+'\n');process.exitCode=1;});
