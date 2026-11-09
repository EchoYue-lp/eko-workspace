const fs = require('node:fs');
const path = require('node:path');
const sharp = require('/Users/ls/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/sharp');
async function main() {
  const source = fs.readFileSync(path.join(__dirname, 'echoleap-ragdoll-head-v2.svg'));
  for (const size of [1024, 512, 256, 128, 64, 32]) {
    await sharp(source).resize(size, size).png().toFile(path.join(__dirname, `echoleap-ragdoll-head-v2-${size}.png`));
  }
  console.log('圆脸布偶猫头矢量稿已导出，天蓝底色保持。');
}
main().catch(error=>{process.stderr.write(String(error)+'\n');process.exitCode=1;});
