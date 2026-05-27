# Imports

Drop JSON files here and bulk-import via:

```bash
# 知识卡 (KnowledgeCard) — title 重名自动 merge slots
./src-tauri/target/debug/cs-interview-coach import imports/your_concepts.json

# 复习卡 (ReviewCard) — Q&A 绑到现有知识卡 (按 title 或 alias 查找)
./src-tauri/target/debug/cs-interview-coach import-reviews imports/your_reviews.json
```

仓库里已有 10 个 JSON 文件（v1-v4 批次共 139 张知识卡 + 143 张复习卡），来源：
- `01_pdd_suji.json` / `02_bytedance_finance.json` — 用豆包整理的算法岗笔试/面试要点
- `03-10_*.json` — 从公开 GitHub repo (LLM Interview / RecSys / DL Awesome / ML Sysdesign 等) 提炼

直接 `import` 即可重建知识库。

## 格式

### 知识卡 JSON

```json
[
  {
    "title": "B+ 树",
    "aliases": ["B+ tree", "B-plus tree"],
    "topic_ids": ["4.2"],
    "slots": {
      "definition": [
        { "text": "...", "cites": [{"source_id": "src-id", "locator": "p.5"}] }
      ],
      "mechanism": [...],
      "complexity": [...],
      "comparison": [...],
      "use_cases": [...],
      "interview_points": [...],
      "pitfalls": [...],
      "code": [...]
    }
  }
]
```

所有 8 个 slot 都可选。重复 title (或 alias) 会自动 merge 进现有卡。

### 复习卡 JSON

```json
[
  {
    "knowledge_card_title": "B+ 树",
    "question": "为什么数据库索引用 B+ 不用 B 树？",
    "answer": "1. 范围查询友好（叶节点双向链表）\n2. 单个节点能存更多 key（非叶节点不存 data）\n3. 查询时间稳定（必到叶子）"
  }
]
```

`knowledge_card_title` 必须能找到现有 KnowledgeCard（按 title 或 alias normalize 匹配），否则该条 review 被跳过并 report。

## Topic IDs

参见 `seed/taxonomy.yaml`，20 顶层 / 100+ 子节点。叶子 id 形如 `4.2`、`18.5` 等。
