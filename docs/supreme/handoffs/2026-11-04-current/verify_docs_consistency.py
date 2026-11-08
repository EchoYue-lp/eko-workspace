#!/usr/bin/env python3
"""2026-11-04 至 11-06 历史验收文档一致性检查（只读）。

这是旧切点索引的维护工具，不是当前 main 或新包的最终门禁；历史制品退役后
原索引可能不再与磁盘一致，不得为通过检查恢复旧环境或改写历史哈希。

用途：本会话多次出现「同一错误在多份文档各存一份」「数字与实际不符」。
本脚本把这些检查固化成可重复运行的形式，改完文档跑一次即可。

用法：/usr/bin/python3 -B docs/supreme/handoffs/2026-11-04-current/verify_docs_consistency.py
退出码：0 全部通过，1 有失败项。
"""
import hashlib
import json
import os
import re
import sys

# 默认以本脚本所在仓库为根；VERIFY_ROOT 可覆盖，用于负向测试（在副本上故意制造错误，
# 确认脚本真的会报错，而不是永远通过）。
ROOT = os.environ.get("VERIFY_ROOT") or os.path.abspath(
    os.path.join(os.path.dirname(__file__), "..", "..", "..", ".."))
ROOT = os.path.abspath(ROOT)
ACC = os.path.join(ROOT, "docs", "supreme", "acceptance")
CHK = os.path.join(ACC, "eko-eclip-gui-manual-checklist-2026-11-02.md")
LEDGER = os.path.join(ROOT, "docs", "supreme", "handoffs", "2026-11-04-current", "remaining-26-ledger.md")
INDEX = os.path.join(ROOT, "docs", "supreme", "handoffs", "2026-11-04-current", "proof-index.json")
CURRENT_INDEX = os.path.join(ROOT, "docs", "supreme", "handoffs", "2026-11-04-current", "proof-index-2026-11-05.json")
if os.path.isfile(CURRENT_INDEX):
    INDEX = CURRENT_INDEX

fails = []
oks = []


def ok(msg):
    oks.append(msg)


def bad(msg):
    fails.append(msg)


def read(p):
    with open(p, encoding="utf-8") as f:
        return f.read()


# 1. 清单：组数与条目数
s = read(CHK)
groups = re.findall(r"^## ([A-M])．", s, re.M)
if len(groups) == 13:
    ok("清单 13 组")
else:
    bad(f"清单组数 {len(groups)} != 13")

items = re.findall(r"^\s*-\s*\[\s*\]\s*\*\*([A-M]\d{2})\b", s, re.M)
if len(items) == 79:
    ok("清单 79 项")
else:
    bad(f"清单条目 {len(items)} != 79")

# 2. 编号唯一且按组连续、前缀与组一致
if len(items) == len(set(items)):
    ok("清单编号无重复")
else:
    dup = sorted({i for i in items if items.count(i) > 1})
    bad(f"清单编号重复: {dup}")

by = {}
for i in items:
    by.setdefault(i[0], []).append(int(i[1:]))
for g, xs in sorted(by.items()):
    if sorted(xs) != list(range(1, len(xs) + 1)):
        bad(f"{g} 组编号不连续: {sorted(xs)}")
if not fails:
    ok("13 组编号均从 01 连续")

# 3. 台账 26 行、编号 1..26
led = read(LEDGER)
rows = re.findall(r"^\|\s*(\d{1,2})\s*\|", led, re.M)
nums = sorted({int(r) for r in rows})
if nums == list(range(1, 27)):
    ok("台账 26 行且编号 1..26 连续")
else:
    bad(f"台账编号异常: {nums[:5]}...共{len(nums)}个")

# 4. 证据索引 30 条且 sha 与磁盘一致
d = json.load(open(INDEX, encoding="utf-8"))
ents = d.get("entries", [])
if len(ents) == 30:
    ok("证据索引 30 条")
else:
    bad(f"证据索引 {len(ents)} 条 != 30")
for e in ents:
    p = os.path.join(ROOT, e["rel"])
    if not os.path.exists(p):
        bad(f"索引条目缺失: {e['rel']}")
        continue
    h = hashlib.sha256(open(p, "rb").read()).hexdigest()
    if h != e["sha256"]:
        bad(f"索引哈希不符: {e['rel']}")
if not any("索引" in f for f in fails):
    ok("索引 30 条全部存在且哈希一致")

# 5. 入口页与目录互引完整
rd = read(os.path.join(ACC, "README.md"))
refs = set(re.findall(r"`(eko-[A-Za-z0-9-]+\.md)`", rd))
have = {f for f in os.listdir(ACC) if f.endswith(".md")} - {"README.md"}
if refs == have:
    ok(f"入口页覆盖全部 {len(have)} 份验收文档且无悬空引用")
else:
    bad(f"入口页引用不完整: 缺 {sorted(have - refs)} 悬空 {sorted(refs - have)}")

# 6. 已更正的错误不得复现（只在「要求用户去验证」的语句里查）
BAD_PATTERNS = [
    (r"推理深度[、，].{0,12}输出上限", "仍把不存在的『推理深度』当可验证项"),
    (r"三项仍(在|可)", "仍写『三项』"),
]
for pat, why in BAD_PATTERNS:
    for p in [CHK, os.path.join(ACC, "eko-first-four-manual-cases-2026-11-04.md")]:
        for ln, line in enumerate(read(p).split("\n"), 1):
            if re.search(pat, line):
                bad(f"{os.path.basename(p)}:{ln} {why}")

if not [f for f in fails if "推理深度" in f or "三项" in f]:
    ok("已更正的 A06 表述未复现")

# 7. 入口页给出的启动脚本路径必须真实存在
launchers = re.findall(r"^(\.echo-semantic/local/[^\r\n`]+\.command)$", rd, re.M)
if len(launchers) == 1 and os.path.isfile(os.path.join(ROOT, launchers[0])):
    ok("入口页启动脚本路径存在")
else:
    bad(f"入口页启动脚本缺失或不唯一: {launchers}")

# 8. SDK 落地清单里的路径必须存在，且 4 个纯增行数之和与正文写的 154 一致
DELTA = os.path.join(ROOT, "docs/supreme/handoffs/2026-11-04-current/sdk-candidate-landing-delta.md")
if os.path.exists(DELTA):
    dd = read(DELTA)
    for rel in [
        "echo-sdk-protocol/src/scalar.rs",
        "sdks/java/src/test/java/com/echoagent/sdk/A2AValueTest.java",
        "sdks/python/tests/test_jwt_values.py",
        "sdks/typescript/test/jwt-values.test.js",
        "echo-sdk-host/tests/core_profile_e2e.rs",
    ]:
        if not os.path.exists(os.path.join(ROOT, "echo-agent-sdk", rel)):
            bad(f"落地清单引用的文件不存在: {rel}")
    if not [f for f in fails if "落地清单引用" in f]:
        ok("落地清单 5 个文件均存在")
    # 4 个纯增文件的候选独有行数之和（按表格列解析：路径 | HEAD独有 | 候选独有 | 方向）
    nums = []
    for line in dd.split("\n"):
        if not line.startswith("|") or "纯增" not in line:
            continue
        cells = [c.strip() for c in line.split("|")]
        if len(cells) < 5:
            continue
        try:
            nums.append(int(cells[3]))
        except ValueError:
            bad(f"落地清单表格列无法解析: {line[:60]}")
    stated = re.search(r"合计 \*\*(\d+)\*\* 行", dd)
    stated_num = int(stated.group(1)) if stated else None
    if sum(nums) == 154 == stated_num:
        ok(f"纯增行数之和 {sum(nums)} 与正文声明一致")
    else:
        bad(f"纯增行数：表格合计 {sum(nums)}，正文声明 {stated_num}，实测应为 154")
else:
    bad("落地清单文件不存在")

# 9. MASTER-PLAN 里的关键数字必须与实际一致
MP = os.path.join(ROOT, "docs/MASTER-PLAN.md")
mp = read(MP)
for token, why in [
    ("928", "SDK 相同数"),
    ("26", "原义务数"),
]:
    if token not in mp:
        bad(f"MASTER-PLAN 缺关键数字 {token}（{why}）")
if not [f for f in fails if "MASTER-PLAN" in f]:
    ok("MASTER-PLAN 含关键数字")

print("通过:")
for m in oks:
    print(f"  + {m}")
if fails:
    print("\n失败:")
    for m in fails:
        print(f"  - {m}")
    sys.exit(1)
print(f"\nALL OK ({len(oks)} 项)")
