#!/usr/bin/env python3
"""重算并核对 2026-11-04 至 11-06 历史状态证据索引。

仅检查原切点，不是当前 main 或新包门禁；旧制品退役后的缺失/变化保留原义，
不为得到零差异恢复旧环境或改写历史条目。

用法：python3 verify_proof_index.py
退出码：0 全部一致；1 存在差异（差异逐条打印，不自动改写索引）。

设计意图：索引里的 SHA-256 是写入瞬间的快照。源码/锁变化后必然失效，
因此交付前应重跑本脚本，把差异当成“索引需要重发”的信号，而不是改索引去迁就现状。
"""
import hashlib
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
CURRENT_INDEX = os.path.join(HERE, "proof-index-2026-11-05.json")
INDEX = CURRENT_INDEX if os.path.isfile(CURRENT_INDEX) else os.path.join(HERE, "proof-index.json")


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def main():
    print(f"index={os.path.basename(INDEX)}")
    index = json.load(open(INDEX))
    missing, changed, ok = [], [], 0
    for entry in index["entries"]:
        path = entry["path"]
        if not os.path.isfile(path):
            missing.append(entry)
            continue
        actual = sha256(path)
        if actual == entry["sha256"] and os.path.getsize(path) == entry["bytes"]:
            ok += 1
        else:
            changed.append((entry, actual))

    print(f"entries={len(index['entries'])} ok={ok} changed={len(changed)} missing={len(missing)}")
    for entry, actual in changed:
        print(f"  CHANGED {entry['key']}: {entry['sha256'][:12]} -> {actual[:12]} ({entry['rel']})")
    for entry in missing:
        print(f"  MISSING {entry['key']}: {entry['rel']}")
    return 0 if not changed and not missing else 1


if __name__ == "__main__":
    sys.exit(main())
