const fs = require('node:fs');
const path = require('node:path');
const sharp = require('/Users/ls/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/sharp');
async function main() {
  for (const version of ['portrait', 'simplified']) {
    const source = fs.readFileSync(path.join(__dirname, `echoleap-ragdoll-${version}.svg`));
    for (const size of [1024, 512, 256, 128, 64]) {
      await sharp(source).resize(size, size).png().toFile(path.join(__dirname, `echoleap-ragdoll-${version}-${size}.png`));
    }
  }
  console.log('参考描摹矢量稿已渲染。');
}
main().catch(error=>{process.stderr.write(String(error)+'\n');process.exitCode=1;});
