const fs = require('node:fs');
const path = require('node:path');
const sharp = require('/Users/ls/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/sharp');
async function main() {
  const source = fs.readFileSync(path.join(__dirname, 'echoleap-ragdoll-head.svg'));
  for (const size of [1024, 512, 256, 128, 64, 32]) {
    await sharp(source).resize(size, size).png().toFile(path.join(__dirname, `echoleap-ragdoll-head-${size}.png`));
  }
  console.log('已导出天蓝底布偶猫头 SVG 和六档 PNG。');
}
main().catch(e=>{process.stderr.write(String(e)+'\n');process.exitCode=1;});
