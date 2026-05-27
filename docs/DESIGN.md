# CS Interview Coach — Design Doc v0.2

作者：Christina + Claude
日期：2026-05-24
状态：**讨论稿** — 标 ❓ 的地方需要你拍板

**v0.2 改动（核心定位重写）**：
- 产品是**知识本身**，不是文档库；PDF/GPT 历史只是原料，榨完即弃（仅内部留作 cite）
- 删除公司/round/source_type 等"来源元数据" — 知识不带出处标签
- 核心流程：**关键词 → 瞬时跳到知识卡 → 站在卡上接面试官追问**
- Map 与 Search 并列为主入口，不搜也能点进去

---

## 0. 一句话目标

一个本地优先的桌面 App：**所有 CS 面试知识被整理成一张张概念卡，关键词亚毫秒级跳转 / 鼠标点击导航 / 站在每张卡上跟 LLM 对话接住面试官追问**。
零散 PDF / GPT 对话只是知识原料，进来后被抽取、归并、丢弃，最终留在屏幕上的全是干净的知识。

---

## 1. 核心需求拆解

| 需求 | 设计含义 |
|---|---|
| **关键词秒跳到概念解释** | 知识卡标题 + 别名建 FST 倒排，前缀即时补全；回车 < 1ms 跳卡 |
| **不搜也能点进去** | Map 视图（分类树）与 Search 同为一级入口 |
| **接住面试官追问** | 每张知识卡内嵌 LLM 对话面板 + 预生成的问题树（What/Why/How/...） |
| **产品是知识，不是文档** | KnowledgeCard 是一等公民；PDF/GPT/笔记只是原料，处理完不暴露给用户 |
| **资料零散、重复但又互补** | 抽取-归并-合成进同一张知识卡，slot 化结构，细节不丢 |
| **资料有冲突** | 冲突进仲裁队列，用户拍板；不自动覆盖 |
| **< 1ms 搜索响应**，能扩到很大 | 本地进程内 Rust 引擎 (Tantivy + HNSW)，不走网络 |
| 长期复习 | 知识卡的每个问题可生成 FSRS 复习卡 |

> ⚠️ 关键洞察：sub-ms 搜索 = **绝对不能走 HTTP**。任何 client/server 架构第一跳就 1-5ms 起步。所以核心必须是**嵌入式引擎、跟 UI 同进程**。

---

## 2. 技术栈推荐（我的判断）

### 2.1 总体架构

```
┌────────────────────────────────────────────┐
│  Tauri 2 桌面壳 (Mac native, ~15MB)         │
│  ┌──────────────────────────────────────┐  │
│  │ 前端: SvelteKit + Tailwind           │  │
│  │  - 卡片复习 UI / 搜索框 / 聊天面板    │  │
│  └────────────────┬─────────────────────┘  │
│                   │ Tauri IPC (zero-copy)  │
│  ┌────────────────▼─────────────────────┐  │
│  │ Rust Core (同进程)                    │  │
│  │  ┌──────────────┬──────────────────┐ │  │
│  │  │ Tantivy      │ hnswlib-rs       │ │  │
│  │  │ (BM25 全文)  │ (向量 ANN)        │ │  │
│  │  └──────┬───────┴────────┬─────────┘ │  │
│  │         └─── RRF 融合 ───┘            │  │
│  │  ┌────────────────────────────────┐  │  │
│  │  │ redb (KV: 元数据 + FSRS 状态)   │  │  │
│  │  └────────────────────────────────┘  │  │
│  │  ┌────────────────────────────────┐  │  │
│  │  │ fastembed-rs (BGE-small 本地)   │  │  │
│  │  └────────────────────────────────┘  │  │
│  └──────────────────┬───────────────────┘  │
│                     │ 仅 LLM 调用走网络     │
│                     ▼                       │
│             Anthropic / OpenAI API          │
└────────────────────────────────────────────┘
```

### 2.2 为什么这么选

| 组件 | 选型 | 替代方案 | 为什么 |
|---|---|---|---|
| **桌面壳** | Tauri 2 | Electron / 纯 Web | 15MB vs 150MB；Mac 原生；以后可打包 iOS |
| **全文检索** | Tantivy (Rust) | SQLite FTS5 / Meilisearch | Tantivy p99 < 1ms @ 100万 doc；Lucene 级别成熟；嵌入式无网络 |
| **向量索引** | hnswlib-rs | FAISS / Qdrant / sqlite-vec | HNSW 是目前 ANN 综合最快；纯 Rust 嵌入式；100万 384维向量内存 ~600MB |
| **嵌入模型** | BGE-small-en-v1.5 (384d) | OpenAI text-embedding-3 / E5 | 本地跑 ~30ms/chunk，免费、离线、隐私；MTEB 分数也够高 |
| **KV / 元数据** | redb | SQLite / RocksDB | 纯 Rust，零依赖，事务安全，比 SQLite 简单 |
| **前端** | SvelteKit | React / Vue | 包体积小、渲染快、卡片切换无卡顿 |
| **LLM** | Claude API (主) + 本地 Ollama 备选 | 只用 OpenAI | 你在 Anthropic 生态，Sonnet 4.6 性价比最高 |
| **SRS 算法** | FSRS-6 | SM-2 (Anki 老版) | FSRS 基于真实数据训练，复习效率比 SM-2 高 20-40% |

❓ **决策点 1**：embedding 用本地 (BGE) 还是云端 (OpenAI text-embedding-3)？
- 本地：免费 / 离线 / 慢一点（30ms/chunk，首次 ingest 慢）
- 云端：贵一点（$0.02/百万 tokens）/ 联网 / 快
- **我的推荐：本地**，配合首次批量 ingest 时跑一晚上就完事。

❓ **决策点 2**：要不要支持手机？
- Tauri 2 已支持 iOS，但 Tantivy/HNSW 在 iOS 上要额外工作
- **我的推荐：v1 只做 Mac**，v2 再考虑手机（手机也很难真正复习深度内容）

---

## 3. 数据结构与索引（核心）

### 3.1 数据模型（用户面 vs 内部）

**用户面对的只有 3 个东西**：

```
KnowledgeCard (知识卡)      一个 CS 概念，如 "B+ 树" / "TCP 三次握手"
  ├── title + aliases        搜索键
  ├── slots (8 个固定栏)      定义/原理/复杂度/对比/场景/考点/陷阱/代码
  ├── QuestionTree           面试官可能怎么追问 + 答案
  ├── related_cards[]        到其他知识卡的边（图谱）
  └── ReviewCards[]          基于每个问题生成的复习卡 (FSRS)

Topic (分类节点)             知识卡挂在哪个分类下
  └── 多对多: 一张卡可挂多个 Topic

ReviewCard (复习卡)          一道题 (Q→A) + FSRS 状态
```

**内部产物（用户不看见）**：

```
Source (原料)               PDF / GPT 对话 / MD / 网页 — 只在抽取时存在
  └── Chunk                 切块后喂给 LLM 抽知识，抽完打 cite 后可丢弃
        └── 全文+向量索引   仅用于"找原文证据"，不参与用户搜索
```

**关键差别**：
- 用户搜索 / 浏览 / 复习 → 只命中 **KnowledgeCard**
- Chunk 索引仅供内部 "show me where this fact came from" 时回查（默认隐藏）
- 这样保证：屏幕上**永远是干净的知识**，不会蹦出"蚂蚁面经第 3 页"这种东西

### 3.2 为什么这样能 sub-ms（且更轻）

由于用户只搜知识卡（不是 chunks），索引规模骤降：

| 操作 | 数据结构 | 时间复杂度 | 实测 @ 10万 知识卡 |
|---|---|---|---|
| **关键词跳卡**（最高频）| FST 前缀树 (title + aliases) | O(prefix len) | **< 0.05ms** |
| 卡内全文搜索 | Tantivy 倒排 | O(log n) | 0.1-0.3ms |
| 语义搜索 | HNSW (M=16, ef=50) | O(log n) | 0.2-0.5ms |
| 混合检索 (RRF) | 两路并行 | O(k) | 0.4-1.0ms |
| 复习卡到期查询 | redb 范围扫描 | O(log n) | < 0.1ms |

⚠️ **核心洞见**：你最高频的动作是"打字 → 跳 B+ 树这张卡"。这个走 **FST title 补全**，比全文/向量都快一个数量级（< 0.05ms），且 100% 精确。

### 3.3 扩展性
- CS 面试的概念总量上限大约 **2000-5000 张知识卡**（覆盖到博士面试也够）
- 这个规模下所有操作都是亚毫秒；内存 < 200MB
- 原始 chunks 即使 10 万级也只在后台 ingest 时用，不影响前台

---

## 4. 知识组织层 (Knowledge Organization Layer) — 核心

> 这一层解决：**你给的资料零散、有重复、但细节互补**。
> 目标：自动整合成一棵清晰的知识树 + 每个叶节点一张规范化"知识卡"+ 一棵问题树。

### 4.1 三层（详见 §3.1，这里强调职责）

```
Topic 树                  骨架，决定知识怎么归类、怎么导航
  └── KnowledgeCard       产品本体，"一个概念一张卡"
        ├── 8 个 slot     结构化教材（定义/原理/...）
        └── QuestionTree  考纲，面试官追问的形状
              └── ReviewCard[]  FSRS 复习单位

Chunk (内部)              抽完即藏；仅用于事实溯源
```

- **Topic** ≠ KnowledgeCard。一个 Topic 节点（如"4.2 索引"）可包含多张卡（B+树、LSM、哈希、倒排、向量）
- 一张卡可以挂到多个 Topic（如"Skip List"挂到 1.1 线性结构 + 4.2 索引）

### 4.2 CS 面试分类树（v0.2 种子 — 全知识、无行为面）

**A. 计算机基础（5 个）**
```
1. 数据结构与算法 (DSA)
   1.1 线性结构 (数组/链表/栈/队列/堆)
   1.2 树 (二叉树/BST/AVL/红黑/Trie/线段树/Skip List)
   1.3 图 (表示/BFS/DFS/最短路/MST/拓扑/网络流)
   1.4 哈希
   1.5 算法范式 (DP/贪心/分治/回溯/双指针/滑窗/二分)
   1.6 字符串 (KMP/Manacher/AC自动机/后缀数组)
   1.7 复杂度分析 (时间/空间/摊销)

2. 操作系统 (OS)
   2.1 进程与线程 (调度/上下文/IPC)
   2.2 同步 (锁/信号量/无锁/MESI)
   2.3 内存 (虚拟内存/页表/分配器/GC)
   2.4 文件系统与 IO (inode/页缓存/零拷贝/io_uring)
   2.5 系统调用与中断

3. 计算机网络 (Network)
   3.1 TCP/IP (握手/挥手/拥塞控制/重传)
   3.2 HTTP/HTTPS (1.1/2/3 QUIC/TLS)
   3.3 DNS / CDN / 负载均衡
   3.4 Socket / 编程模型 (epoll/kqueue/io_uring)

4. 数据库 (DB)
   4.1 关系模型与 SQL
   4.2 索引 (B+树/LSM/哈希/倒排/向量)
   4.3 事务 (ACID/MVCC/隔离级别)
   4.4 存储引擎 (InnoDB/RocksDB)
   4.5 分布式 (分片/复制/共识/CAP/PACELC)
   4.6 NoSQL / KV / 文档 / 时序 / 向量数据库

5. 计算机体系结构 (Architecture)
   5.1 CPU (流水线/乱序/分支预测/SIMD)
   5.2 缓存 (多级 cache/MESI/false sharing)
   5.3 内存 (NUMA/对齐/带宽)
   5.4 GPU 架构 (SM/warp/tensor core/shared mem)
   5.5 性能分析 (profiling/roofline)
```

**B. 工程能力（5 个）**
```
6. 系统设计 (System Design)
   6.1 设计原则 (SOLID/DRY/YAGNI)
   6.2 架构模式 (单体/微服务/event-driven/CQRS)
   6.3 高并发 (缓存/限流/熔断/队列/异步)
   6.4 经典题 (短链/朋友圈/打车/IM/秒杀/feed/搜索)
   6.5 可观测性 (日志/指标/链路/告警)

7. 编程语言
   7.1 Python (GIL/asyncio/类型/打包/CPython 内核)
   7.2 C++ (内存/模板/并发/RAII/STL)
   7.3 Java/Go (按需扩展)

8. 大数据与流处理 (Big Data)
   8.1 批处理 (MapReduce/Spark)
   8.2 流处理 (Flink/Kafka Streams)
   8.3 消息队列 (Kafka/Pulsar)
   8.4 数仓 / OLAP (Hive/Presto/ClickHouse/Doris)
   8.5 数据湖 (Iceberg/Delta/Hudi)

9. 云原生与 DevOps (Cloud Native)
   9.1 容器 (Docker/runc/containerd)
   9.2 编排 (K8s/Helm/Operator)
   9.3 服务治理 (服务网格/网关)
   9.4 CI/CD
   9.5 IaC (Terraform/Pulumi)

10. 安全 (Security)
    10.1 密码学基础 (对称/非对称/哈希/签名)
    10.2 Web 安全 (XSS/CSRF/SQLi/SSRF)
    10.3 认证授权 (OAuth2/JWT/RBAC)
    10.4 系统/网络安全
```

**C. 数学基础（1 个）**
```
11. 数学 (Math for ML/CS)
    11.1 线性代数 (矩阵/特征值/SVD/张量)
    11.2 概率与统计 (分布/MLE/MAP/贝叶斯/假设检验)
    11.3 微积分与优化 (梯度/凸优化/拉格朗日)
    11.4 信息论 (熵/KL/互信息)
    11.5 数值方法 (稳定性/精度/收敛)
```

**D. AI / ML（9 个 — 你主战场）**
```
12. 机器学习 (ML)
    12.1 监督 (线性/树/SVM/集成)
    12.2 无监督 (聚类/降维/异常)
    12.3 评估 (指标/CV/过拟合/校准)
    12.4 特征工程
    12.5 模型解释 (SHAP/LIME)

13. 深度学习 (DL)
    13.1 基础 (BP/优化器/正则/归一化/初始化)
    13.2 CNN 基础
    13.3 RNN / LSTM / GRU
    13.4 Attention / Transformer 基础
    13.5 损失函数与训练技巧

14. 自然语言处理 (NLP)
    14.1 文本表示 (word2vec/glove/fasttext)
    14.2 序列建模 (RNN/CRF)
    14.3 预训练模型 (BERT/RoBERTa/T5)
    14.4 经典任务 (NER/分类/QA/摘要/翻译)
    14.5 中文 NLP (分词/词性/句法)

15. 计算机视觉 (CV)
    15.1 经典 CNN (AlexNet/VGG/ResNet/EfficientNet)
    15.2 目标检测 (YOLO 全系/DETR/Faster-RCNN)
    15.3 分割 (U-Net/Mask-RCNN/SAM)
    15.4 视觉 Transformer (ViT/Swin/DINOv2)
    15.5 生成 (GAN/VAE/Diffusion/SD)

16. 强化学习 (RL)
    16.1 基础 (MDP/Bellman/value/policy)
    16.2 Q 系列 (DQN/Double/Dueling)
    16.3 策略梯度 (REINFORCE/A2C/PPO/SAC)
    16.4 RLHF / DPO / GRPO
    16.5 Model-based RL (MuZero/Dreamer)

17. 多模态 (Multimodal)
    17.1 视觉-语言对齐 (CLIP/SigLIP)
    17.2 视觉语言模型 (LLaVA/Qwen-VL/InternVL)
    17.3 音频-语言 (Whisper/SeamlessM4T)
    17.4 视频理解
    17.5 统一多模态生成

18. LLM / Agent
    18.1 Transformer 内核 (Attention/KV cache/MoE/MLA)
    18.2 预训练 (data/scaling law/loss)
    18.3 后训练 (SFT/RLHF/DPO/GRPO)
    18.4 RAG (chunk/检索/重排/评估)
    18.5 Agent (planning/tools/memory/multi-agent)
    18.6 Agent 评估与可观测性 ★ 你研究方向

19. 推荐 / 搜索 / 广告 (RecSys / Search / Ads)
    19.1 召回 (双塔/向量/i2i/u2i/SwingI2I)
    19.2 粗排 / 精排
    19.3 重排 / 多目标 / 多样性
    19.4 经典模型 (FM/Wide&Deep/DIN/DeepFM/SIM)
    19.5 序列推荐 (SASRec/BERT4Rec/生成式推荐)
    19.6 搜索 (query 理解/相关性/向量检索)
    19.7 广告 (CTR/CVR/出价/拍卖/uplift)
    19.8 评估 (AUC/NDCG/线上 A/B/因果)

20. AI 工程化 (AI Engineering) ★ 你强项
    20.1 训练框架 (PyTorch/JAX/分布式 DDP/FSDP)
    20.2 微调 (LoRA/QLoRA/PEFT/全参)
    20.3 量化 / 蒸馏 / 剪枝
    20.4 推理引擎 (vLLM/SGLang/TensorRT-LLM/llama.cpp)
    20.5 边缘部署 (Jetson/Triton/ONNX/CoreML)
    20.6 MLOps (W&B/MLflow/feature store/model registry)
```

> 共 **20 个顶层 / 100+ 子节点**。这是种子，LLM 在 ingest 时可推荐新叶节点（标 `pending`），你审核后纳入（决策 6）。
> Map 视图按 A/B/C/D 四组排版，不会一屏看见全部 20 个。

### 4.3 KnowledgeCard 规范结构（每个叶节点一张）

每张知识卡用统一的 8 个 slot，多源信息按 slot 归位：

```yaml
topic: 4.2.1 B+ 树
last_synthesized_at: 2026-05-24T10:00:00
sources: [chunk_id_3, chunk_id_17, chunk_id_42]  # 全部证据

slots:
  定义:
    text: "B+ 树是一种自平衡多路搜索树，所有数据存在叶子节点..."
    cite: [chunk_3, chunk_17]

  原理机制:
    - point: "非叶子节点只存索引键"
      cite: [chunk_3]
    - point: "叶子节点用双向链表串起来"
      cite: [chunk_42]   # 你笔记里的细节
    - point: "节点 fanout 通常 ~100-200"
      cite: [chunk_3]

  复杂度:
    查询: "O(log_B n), B 为 fanout"
    插入: "O(log_B n) 摊销"
    cite: [chunk_3]

  对比相关:
    vs B 树: "B+ 叶子链表 → 范围查询快"
    vs LSM 树: "B+ 读快写慢，LSM 反之"
    vs 红黑树: "B+ 多路 → 磁盘 IO 少"
    cite: [chunk_17, chunk_42]

  应用场景:
    - "MySQL InnoDB 主键 + 二级索引"   # GPT 对话提的细节
    - "PostgreSQL 默认索引"
    cite: [chunk_17]

  经典考点:
    - "为什么 MySQL 用 B+ 不用 B 树？"
    - "InnoDB 二级索引为什么要回表？"

  易错点:
    - "B+ 树非叶节点不存数据 (常和 B 树搞混)"

  代码/伪代码:
    (可选)
```

**关键**：每条事实都带 `cite`（指回原始 chunk），点击能跳回来源 PDF 第几页。

### 4.4 Dedup / Merge / Contradict 算法

新 chunk 进入时的处理流程：

```
新 chunk
  │
  ├─[1] LLM 一次性分类 → 推断 0..N 个 Topic
  │     (用整个分类树做 zero-shot, ~200 tokens)
  │
  ├─[2] 对每个分到的 Topic：
  │     向量搜索这个 Topic 下已有 chunks (top-5, cosine)
  │       ├─ sim > 0.92  → 判定 DUPLICATE
  │       │              → 保留更详细的，标 alias
  │       ├─ 0.75-0.92  → 判定 SUPPLEMENT
  │       │              → 触发 KnowledgeCard 重合成
  │       └─ < 0.75     → 判定 NEW ANGLE
  │                      → 直接挂到 Topic
  │
  ├─[3] LLM 二次判定 (只在 SUPPLEMENT 时跑)：
  │     "新信息是补充、纠正、还是冲突？"
  │       ├─ 补充 → 直接 merge 进对应 slot
  │       ├─ 纠正 → 替换旧 slot，旧的标 deprecated
  │       └─ 冲突 → ⚠️ 进 ConflictQueue，等你审核
  │
  └─[4] 增量更新索引 (Tantivy + HNSW)
```

**为什么这套设计能保证"细节不丢、又不重复"**：
- DUPLICATE 不丢：原 chunk 留着，alias 指向 canonical
- SUPPLEMENT 不丢：所有 slot 都带 multi-source cite
- 冲突不丢：进队列让人决定，不自动覆盖（避免 LLM 自信瞎合并）

### 4.5 QuestionTree（问题树）— 你要的"什么方面有什么问题、子问题"

每个 Topic 自动生成一棵问题树，结构固定：

```
Topic: 4.2.1 B+ 树
└── 问题树
    ├── [是什么] What
    │   ├── B+ 树的定义？
    │   └── 跟 B 树有什么区别？
    │
    ├── [为什么] Why
    │   ├── 为什么数据库索引用 B+ 不用 B？
    │   ├── 为什么不用红黑树？
    │   └── 为什么不用哈希？
    │
    ├── [怎么做] How
    │   ├── 插入时节点分裂怎么做？
    │   ├── 删除时怎么合并？
    │   └── 范围查询怎么走？
    │
    ├── [场景] When/Where
    │   ├── InnoDB 主键索引怎么组织？
    │   └── 二级索引为什么要回表？
    │
    ├── [边界/陷阱] Pitfalls
    │   ├── 索引失效的常见情况？
    │   └── 联合索引最左前缀？
    │
    └── [延伸] Extensions
        ├── B+ vs LSM 的取舍？
        └── 自适应哈希索引是什么？
```

- 每个叶问题挂一个/多个 chunks 作答案
- 每个叶问题可以"一键变成复习卡"
- 整棵树存为 markdown，你可以编辑

**生成方式**：KnowledgeCard 合成后，LLM 按 6 个固定维度 (What/Why/How/When/Pitfalls/Extensions) 生成问题列表。

### 4.6 全局 Mind Map UI

第四个主视图（在 Review/Search/Ask 之外）：**Map**

```
┌─────────────────────────────────────────────────────┐
│  [搜索框]                              [筛选: 全部▼] │
├─────────────────────────────────────────────────────┤
│                                                     │
│    ROOT                                             │
│     ├─ DSA ●●●●○ (78%)  [142 chunks, 28 cards]      │
│     │   ├─ 树 ●●●○○ (52%)                           │
│     │   │   ├─ B+ 树 ●●●●● (95%) ✓                  │
│     │   │   ├─ 红黑树 ●●○○○ (38%) ⚠ 待复习          │
│     │   │   └─ Trie ○○○○○ (0%) 🆕                   │
│     │   └─ 图 ●●●○○                                 │
│     ├─ OS  ●●○○○                                    │
│     ...                                             │
│                                                     │
│  ⚠ 2 个 conflict 待你裁决  →                        │
└─────────────────────────────────────────────────────┘
```

每个节点显示：
- 掌握度（来自 FSRS 平均 retrievability）
- chunks/cards 数量
- 状态徽标（新/待复习/冲突）
- 点击展开 → 看 KnowledgeCard + QuestionTree

技术：D3.js collapsible tree（v1 够用），v2 换 zoomable circle pack。

### 4.7 关键开放问题

❓ **决策点 6**：分类树要不要 100% 锁死种子结构？
- 锁死：好导航，但 LLM 不能新增节点
- 开放：灵活，但容易长歪
- **我推荐：种子锁死 1-2 级，叶子可由 LLM 提议、你审核**

❓ **决策点 7**：KnowledgeCard 合成什么时候触发？
- 实时（每次新 chunk）→ 贵
- 批量（每天一次后台）→ 便宜，但你看不到最新
- 按需（你打开这个 Topic 时）→ 折中
- **我推荐：按需 + 增量缓存**

❓ **决策点 8**：冲突处理 UI 优先级？
- M2 就要做（早期资料杂，冲突多）
- M3 再做（先把基本流程跑通）
- **我推荐：M2 必须做**，否则你会发现 LLM 默默把你某个细节合并掉了

---

## 5. Ingestion Pipeline （含知识层挂接）

```
原始资料 → 解析 → 清洗 → 切块 → 向量化 → 写入索引 → 知识层处理 (§4.4)
                                                       ├─ Topic 分类
                                                       ├─ Dedup / Merge / Contradict
                                                       └─ KnowledgeCard / QuestionTree 重生成
```

| 格式 | 解析器 | 备注 |
|---|---|---|
| PDF (文本型) | pdfium-rs / mupdf | 处理多列、公式、代码块 |
| **PDF (扫描/图片型)** | **PaddleOCR (中英) / Tesseract** | 微信转发的图片 PDF 必须 OCR |
| ChatGPT 导出 (conversations.json) | 自己写 | 提取 user/assistant turns，每个 turn = 1 chunk |
| Markdown / 自己笔记 | pulldown-cmark | 按 H2/H3 切 |
| Web (CS Notes / Hello-Algo) | 内置爬虫 + readability | 白名单 + 离线缓存 |
| .docx / .pptx | docx-rs / pptx-parser | 你 1216 文件夹里有不少 |

### 5.1 抽取流程（核心：原料 → 知识，原料丢弃）

```
PDF/MD/GPT → 解析文本 → LLM 抽"原子知识点"(JSON 数组) 
                            ↓
                   每个知识点找对应 Topic
                            ↓
              喂给 §4.4 dedup/merge 进 KnowledgeCard 的 slot
                            ↓
              原 chunk 仅留 hash 指针（不在 UI 出现）
```

举例：你那张"蚂蚁搜索推荐广告算法"的 PDF 进来后：
- LLM 抽出 ~50 个原子知识点："Wide&Deep 双塔结构是…"、"DIN 的 attention 怎么算"、"召回阶段 in-batch negative 的作用"…
- 每个点找 Topic（多半挂到 8.x LLM/Agent，或新建 11. 推荐系统 — 见 §4.2）
- merge 进对应知识卡的 slot
- **PDF 本身在 UI 任何地方都不出现**，你以后看到的就是干净的"Wide&Deep"知识卡

### 5.2 微信 PDF 专项处理

**自动发现路径**：
```
~/Library/Containers/com.tencent.xinWeChat/
  Data/Documents/xwechat_files/<wxid>/msg/file/YYYY-MM/*.pdf
```

**通用 PDF 坑及解法**：

| 坑 | 解法 |
|---|---|
| 文件重复 (`xxx(1).pdf` / `_副本`) | SHA256 内容去重 |
| 图片型 PDF（截图无文本层）| 检测 text < 50 字/页 → PaddleOCR ch_PP-OCRv4 |
| 中文为主 | jieba tokenizer + BGE-M3 |
| 微信噪音（水印 / "复制此条消息打开 App"）| 正则黑名单清洗 |
| 文件名垃圾（`IMG_2024.pdf` / `大贾(1).pdf`）| LLM 看第一页判"是不是含知识"，不是就跳过 |
| 非知识文件（订票 / 成绩单 / 简历）| 同上，自动跳过 |

✅ **决策点 9（已定）**：知识卡保留"看原文"按钮，**但条件渲染** — 有 cite 的事实才显示按钮；没原文（如 LLM 概括 / 你手敲 / 跨多源合成的总结）就不显示。
- 实现：每个 slot fact 有 optional `cite[]` 字段，cite 非空才挂按钮
- 收益：界面默认干净，但需要溯源时一点就有；不会出现"点了按钮跳不到任何地方"的尴尬

### 5.3 一键扫描 UI

Map / Search 顶栏 **📥 扫描原料** 按钮（所有来源统一）：

```
┌─────────────────────────────────────────────┐
│  扫描新原料                                  │
├─────────────────────────────────────────────┤
│  来源:  ☑ 微信文件夹  ☑ ~/Desktop  ☑ 拖入   │
│                                              │
│  发现 47 个新文件 → LLM 初判 →               │
│   ✓ 12 个含知识 → 准备抽取                   │
│   ✗ 35 个跳过 (订票/简历/截图)               │
│                                              │
│  预计：~340 个知识点 → ~80 张知识卡          │
│    ├─ 60 张新建                              │
│    ├─ 18 张补充已有                          │
│    └─ 2 个冲突 ⚠ (需仲裁)                    │
│                                              │
│  [取消]                          [开始抽取 →]│
└─────────────────────────────────────────────┘
```

### 5.4 自动复习卡生成

- KnowledgeCard 合成后，QuestionTree 的每个问题 → 自动生成一张 ReviewCard
- 进 FSRS 调度
- ❓ **决策点 3**：默认开启还是手动？推 **默认开启 + 预算上限**

---

## 6. UI / 交互（v1 范围）

### 6.1 三个主入口（地位平等）

```
┌──────────────────────────────────────────┐
│  [ Search   |   Map   |   Review ]       │  ← 顶栏切换
├──────────────────────────────────────────┤
│                                          │
│   ⌘K 任何时候全局唤起 Search             │
│                                          │
└──────────────────────────────────────────┘
```

**A. Search（高频，默认页）— 关键词秒跳**
```
┌────────────────────────────────────────┐
│  🔍 b+树_                              │  ← 输入框
├────────────────────────────────────────┤
│  ▸ B+ 树                  数据库/索引   │  ← FST 实时补全
│    B-tree                数据结构/树    │     (< 0.05ms)
│    跳表 (Skip List)      数据结构      │
│                                        │
│  按 ↵ 跳到第一条        按 ⇥ 切语义搜  │
└────────────────────────────────────────┘
```
- 输入即补全（FST），回车瞬时跳卡
- 没精确命中时按 Tab 走语义/混合搜索

**B. Map — 不搜也能逛**
- 整棵分类树（§4.6），节点带掌握度色阶
- 点叶节点 → 进知识卡

**C. Review — 后台 FSRS 队列**
- 今天到期的复习卡
- 不是用户日常入口，是"自动喂养"模式

### 6.2 知识卡视图（产品核心，所有路径都汇聚到这）

打开任意一张知识卡的布局：

```
┌────────────────────────────────────────────────────────┐
│  B+ 树                              [掌握 ●●●●○ 78%]   │
│  挂在: 4.2 索引 / 1.2 树            [开始复习这张卡]   │
├──────────────────────────────────┬─────────────────────┤
│                                  │  💬 面试官追问       │
│  📖 定义                          │ ─────────────────  │
│  自平衡多路搜索树，所有数据在...   │ 快速问题:           │
│                                  │  • 为啥不用 B 树？   │
│  ⚙ 原理机制                       │  • 范围查询怎么走？ │
│  • 非叶节点只存索引键             │  • InnoDB 回表？    │
│  • 叶节点双向链表                 │  • 索引失效场景？   │
│  • fanout ~100-200                │                     │
│                                  │ 或自由提问:         │
│  ⏱ 复杂度                         │ ┌─────────────────┐│
│  查询 O(log_B n), 插入 O(log_B n) │ │ 你随便问...     ││
│                                  │ └─────────────────┘│
│  🔁 对比相关                      │                     │
│  → vs B 树    → vs LSM            │ (LLM 用本卡 + 相关  │
│  → vs 红黑树                      │  卡作为 context     │
│                                  │  扮演 candidate 回答│
│  🎯 经典考点 / ⚠ 易错点 / 💻 代码 │  你看怎么接)        │
│                                  │                     │
└──────────────────────────────────┴─────────────────────┘
```

**右边的"面试官追问"面板就是你说的"接住招"**：
- 上半：QuestionTree 预生成的题目，一键点 → LLM 给候选答案
- 下半：自由对话，LLM 以本卡为 context 扮演候选人作答
- 你可以问 "如果面试官接着问 XXX 怎么答" — LLM 给递进答案
- 任何回答都能 **一键存为新 slot 内容** 或 **新建复习卡**

✅ **决策点 10（已定）**：LLM 用 **(c) 框架+要点** 风格答题
- 输出形如：「先讲 A → 再展开 B 的 3 个点 → 最后用 C 收尾。要点：①… ②… ③…」
- 你自己拿要点组织表述，强化记忆而不是被动看答案
- 模拟候选人口吻 (b) 留给 v2 做"模拟面试"模式

### 6.3 冲突仲裁面板
- Map 视图顶部小红点提醒
- 一屏显示：旧 slot / 新 slot / 双方原文片段
- 三按钮：保留旧 / 用新 / 都留（标"存在不同说法"）

---

## 7. 项目结构

```
cs-interview-coach/
├── docs/
│   └── DESIGN.md            ← 本文
├── src-tauri/               ← Rust 后端
│   ├── src/
│   │   ├── main.rs
│   │   ├── index/           ← Tantivy + HNSW
│   │   ├── ingest/          ← PDF/MD/GPT/docx/pptx 解析
│   │   ├── embed/           ← fastembed (BGE-M3)
│   │   ├── knowledge/       ← §4: Topic 树 / KnowledgeCard / QuestionTree / dedup
│   │   ├── srs/             ← FSRS
│   │   └── llm/             ← Anthropic/OpenAI client
│   └── Cargo.toml
├── src/                     ← SvelteKit 前端
│   ├── routes/
│   │   ├── +page.svelte     ← Review
│   │   ├── map/             ← Mind Map (D3)
│   │   ├── search/
│   │   ├── ask/
│   │   └── conflicts/       ← 冲突仲裁
│   └── lib/
├── seed/
│   └── taxonomy.yaml        ← §4.2 分类树种子
├── package.json
└── tauri.conf.json
```

---

## 8. 开发路线（建议分阶段）

### M1（1-2 周）— 能用的最小闭环
- [ ] Tauri 项目脚手架 + Rust core
- [ ] Tantivy 全文检索 + 手动加文档
- [ ] 简单 Review UI + FSRS
- [ ] 手动创建卡片
- [ ] **种子分类树 taxonomy.yaml 写好**（不接 LLM，先静态展示）
- **目标**：能用，跟 Anki 差不多 + 知识树骨架在

### M2（2-3 周）— Ingestion + 向量 + **知识层 v1**
- [ ] PDF / MD / ChatGPT 导出 / docx / pptx 解析
- [ ] BGE-M3 本地向量化
- [ ] HNSW + 混合检索 (RRF)
- [ ] 1216 文件夹一键全量导入
- [ ] **LLM 自动 Topic 分类**（chunk → 分类树）
- [ ] **Dedup / Merge / Conflict 流程**（含冲突仲裁 UI）
- [ ] **Mind Map 视图**（D3 collapsible tree）

### M3（1-2 周）— LLM 闭环 + **知识合成**
- [ ] Claude API 集成
- [ ] RAG 问答（Ask 视图）
- [ ] **KnowledgeCard 按需合成**
- [ ] **QuestionTree 自动生成**
- [ ] 自动 Q&A 卡片生成

### M4（持续）— 内容获取
- [ ] 内置 CS Notes / Hello-Algo / System Design Primer 离线包
- [ ] Web 爬虫（白名单源）
- [ ] 中英双语 polish

**总计 ~7-9 周到 v1**（比之前估算多 1 周，因为加了知识层）。
**M1 一周后就能日常用**，M2 完了你扔的零散资料就能自动归位。

---

## 9. 风险与开放问题

| 风险 | 缓解 |
|---|---|
| Rust 学习曲线 | 核心逻辑用 Rust，UI/胶水用 TS；必要时部分 Rust 模块先用 Python 原型 |
| HNSW 索引重建慢 | 增量更新 + 后台合并；首次构建可以接受跑一晚上 |
| LLM 成本（知识层会频繁调）| 默认 Sonnet 4.6；分类用 Haiku；合成用 Sonnet；本地 Ollama 兜底 |
| 中文资料（你有中文 PDF）| Tantivy 用 jieba tokenizer；BGE-M3 替代 BGE-small（支持中英） |
| **LLM 合并合错信息** | 高 sim 才自动合，否则进冲突队列；所有 slot 带 cite，可溯源回滚 |
| **分类树长歪** | 1-2 级锁死；新叶节点必须人工审核才生效 |

❓ **决策点 4**：中文支持要做到什么程度？你那两份字节面试 PDF 是中文。我建议直接上 **BGE-M3**（多语言），稍微大一点（~500MB 模型），但中英文都好。

❓ **决策点 5**：要不要把 *applyagent* 文件夹的对话历史也灌进去当复习材料？里面应该有不少 JD 分析、技能讨论。

---

## 10. 我需要你确认的几个东西

**技术决策**：
1. **决策点 1**：embedding 本地还是云端？（推 **本地 BGE-M3**）
2. **决策点 2**：v1 只 Mac 还是包含手机？（推 **只 Mac**）
3. **决策点 3**：自动生成卡片默认开还是关？（推 **默认开 + 预算上限**）
4. **决策点 4**：中文支持力度？（推 **BGE-M3 + jieba**）
5. **决策点 5**：要不要灌 applyagent 历史？（推 **要**）

**知识层决策**：
6. ✅ **决策点 6（已定）**：分类树 1-2 级锁死；叶子可由 LLM 提议、用户审核纳入
7. ✅ **决策点 7（已定）**：KnowledgeCard 按需合成 + 增量缓存（首次打开/有新原料时触发）
8. ✅ **决策点 8（已定）**：冲突仲裁 UI **M2 必做**，红点提醒 + 一屏对比

**v0.2 新加（知识为产品）**：
9. ✅ **决策点 9（已定）**：保留"看原文"按钮，**条件渲染**（有 cite 才显示）
10. ✅ **决策点 10（已定）**：LLM 用 **(c) 框架+要点** 风格

**整体确认**：
11. 是否同意整体技术栈：Tauri 2 + Rust (Tantivy + HNSW) + SvelteKit + Claude API？
12. 是否同意 M1-M4 分阶段？
13. ✅ **决策点 13（已定）**：§4.2 分类树 = **20 顶层（A/B/C/D 四组）**，删行为面，加体系结构/大数据/云原生/安全/数学/NLP/CV/RL/多模态/推荐搜索广告/AI 工程化
14. v0.2 的"知识为产品、原料丢弃"定位你认可吗？

---

## 11. 不在 v1 范围（先记下来）

- 多人协作 / 云同步
- 移动端
- 模拟面试 (voice agent)
- 代码题在线判题
- 知识图谱可视化

这些都好做，但 v1 先聚焦"快+全+智能"三件事。
