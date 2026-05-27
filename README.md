# CS Interview Coach

本地优先的 CS 面试知识 App。详见 [`docs/DESIGN.md`](docs/DESIGN.md)。

## 当前状态：M1 ✅

可用：
- 三视图导航（Search / Map / Review）
- 20 顶层种子分类树（A/B/C/D 四组，[`seed/taxonomy.yaml`](seed/taxonomy.yaml)）
- 手动新建知识卡，8 slot 编辑（定义/原理/复杂度/对比/场景/考点/陷阱/代码）
- 手动加复习卡，FSRS-like 调度 + 四档评分
- FST 标题/别名前缀补全（亚毫秒）
- ⌘K 全局唤起搜索

未做（M2 以后）：
- 不能 ingest PDF/GPT/笔记（手动建卡）
- 没有 Tantivy 全文 + HNSW 语义搜索
- 没有 LLM 接面试官追问
- 没有 KnowledgeCard 自动合成 / 冲突仲裁

---

## 开发

```bash
# 装依赖（第一次）
npm install

# 跑（窗口会弹出）
npm run tauri dev

# 类型检查
npm run check

# 打包
npm run tauri build
```

**数据位置**：`~/Library/Application Support/com.jyj.cs-interview-coach/coach.redb`
删了就重置所有卡片和复习状态。

---

## M1 Smoke Test（手动跑一遍验证）

打开 App 后按顺序：

1. **Map**：左上角点 Map → 看到 4 组 / 20 顶层
2. **展开**：点 `1. 数据结构与算法` → 看到 1.1-1.7 子节点
3. **进 Topic**：点 `1.1 线性结构` → 进入 topic 详情页
4. **新建卡**：输入"数组"，点创建 → 跳到卡详情页
5. **编辑 slots**：定义里加"连续内存…"，复杂度里加"随机访问 O(1)"，保存
6. **加别名**：填 `array, list` → 保存
7. **加复习卡**：右边面板
   - Q："数组随机访问的时间复杂度？"
   - A："O(1)，因为内存连续可以直接算偏移"
   - 加入复习
8. **Search 跳卡**：⌘K → 输 `数` → 看到"数组"候选 → ↵ 回车跳过去
9. **Search 别名**：⌘K → 输 `arr` → 应该也能匹配到（别名进了 FST）
10. **Review**：点 Review → 看到刚才那道题 → Space 显示答案 → 按 3 (Good)
11. **复习卡进调度**：回卡片详情看，复习卡下次复习日期已经推到 ~2-3 天后

通了就是 M1 OK。

---

## 已知问题

- `state.rs:13` dead_code warning 是为 M2 留的 `data_dir` 字段，不影响运行
- macOS 上 autofocus a11y warning 不影响功能
- 首次 `tauri dev` 编译 ~10 分钟下所有依赖；增量 < 5s

---

## 下一步（M2）

按 [`docs/DESIGN.md`](docs/DESIGN.md) §8 路线图：
- PDF / GPT 导出 / MD ingestion
- BGE-M3 本地向量化
- HNSW + Tantivy 混合检索
- LLM 自动 Topic 分类 + KnowledgeCard 合成
- Dedup / Merge / Conflict 流程 + 冲突仲裁 UI
- D3 Mind Map
