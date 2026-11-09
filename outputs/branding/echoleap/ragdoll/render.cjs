const fs = require('node:fs');
const path = require('node:path');
const sharp = require('/Users/ls/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/sharp');
async function main() {
  const source = fs.readFileSync(path.join(__dirname, 'echoleap-ragdoll.svg'));
  await sharp(source).resize(1080, 1200).png().toFile(path.join(__dirname, 'echoleap-ragdoll-transparent.png'));
  await sharp(source).resize(1080, 1200).flatten({background:'#F3F0EB'}).png().toFile(path.join(__dirname, 'echoleap-ragdoll-preview.png'));
  console.log('布偶猫 SVG、透明 PNG 与浅底预览已生成。');
}
main().catch(e=>{process.stderr.write(String(e)+'\n');process.exitCode=1;});
