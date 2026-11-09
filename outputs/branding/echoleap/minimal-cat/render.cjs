const fs = require('node:fs');
const path = require('node:path');
const sharp = require('/Users/ls/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/sharp');
async function main() {
  const svg = fs.readFileSync(path.join(__dirname, 'echoleap-cat.svg'));
  for (const size of [1024, 512, 256, 128, 64, 32]) {
    await sharp(svg).resize(size, size).png().toFile(path.join(__dirname, `echoleap-cat-${size}.png`));
  }
  console.log('简笔猫头已导出；仅保存独立品牌资产，未修改应用或其它工作区。');
}
main().catch(error => { process.stderr.write(String(error) + '\n'); process.exitCode = 1; });
