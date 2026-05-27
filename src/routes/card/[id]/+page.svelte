<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { api } from '$lib/api';
  import { EMPTY_SLOTS, SLOT_LABELS } from '$lib/types';
  import type { KnowledgeCard, ReviewCard, Slots, Fact } from '$lib/types';

  let cardId = $derived(page.params.id!);
  let card = $state<KnowledgeCard | null>(null);
  let reviews = $state<ReviewCard[]>([]);
  let aliasesText = $state('');
  let saving = $state(false);
  let savedMsg = $state('');

  // Prev / Next siblings (cards in the same primary topic, alphabetically)
  let siblings = $state<KnowledgeCard[]>([]);
  let curIdx = $state(-1);
  let prevCard = $derived(curIdx > 0 ? siblings[curIdx - 1] : null);
  let nextCard = $derived(curIdx >= 0 && curIdx < siblings.length - 1 ? siblings[curIdx + 1] : null);
  let primaryTopicId = $derived(card?.topic_ids?.[0] ?? null);

  let newReviewQ = $state('');
  let newReviewA = $state('');

  async function load() {
    card = await api.getCard(cardId);
    if (card) {
      aliasesText = card.aliases.join(', ');
      reviews = await api.listReviewsForCard(cardId);
      await loadSiblings(card);
    }
  }

  async function loadSiblings(c: KnowledgeCard) {
    // Build prev/next list from the primary topic. If no topic, fall back
    // to all cards alphabetically.
    let list: KnowledgeCard[];
    if (c.topic_ids.length > 0) {
      list = await api.listCardsByTopic(c.topic_ids[0]);
    } else {
      list = await api.listCards();
    }
    list.sort((a, b) => a.title.localeCompare(b.title, 'zh-CN'));
    siblings = list;
    curIdx = list.findIndex((x) => x.id === c.id);
  }

  function go(c: KnowledgeCard | null) {
    if (!c) return;
    goto(`/card/${c.id}`);
  }

  // Reload when route changes (prev/next without remount)
  $effect(() => {
    if (cardId) load();
  });

  function onKey(e: KeyboardEvent) {
    const tag = (e.target as HTMLElement)?.tagName;
    const inField = tag === 'INPUT' || tag === 'TEXTAREA' || (e.target as HTMLElement)?.isContentEditable;
    if (inField) return;
    if (e.metaKey || e.ctrlKey || e.altKey) return; // let ⌘← / ⌘→ be layout-level back/forward
    if (e.key === 'ArrowLeft' && prevCard) { e.preventDefault(); go(prevCard); }
    else if (e.key === 'ArrowRight' && nextCard) { e.preventDefault(); go(nextCard); }
  }

  onMount(() => {
    window.addEventListener('keydown', onKey);
  });
  onDestroy(() => {
    window.removeEventListener('keydown', onKey);
  });

  function addFact(key: keyof Slots) {
    if (!card) return;
    const slots = { ...card.slots };
    slots[key] = [...slots[key], { text: '', cites: [] }];
    card = { ...card, slots };
  }

  function removeFact(key: keyof Slots, idx: number) {
    if (!card) return;
    const slots = { ...card.slots };
    slots[key] = slots[key].filter((_: Fact, i: number) => i !== idx);
    card = { ...card, slots };
  }

  async function save() {
    if (!card) return;
    saving = true;
    try {
      const aliases = aliasesText.split(',').map((s) => s.trim()).filter(Boolean);
      const updated = await api.updateCardSlots(card.id, card.slots, aliases);
      card = updated;
      aliasesText = updated.aliases.join(', ');
      savedMsg = '已保存 ✓';
      setTimeout(() => savedMsg = '', 1500);
    } finally {
      saving = false;
    }
  }

  async function addReview() {
    if (!card) return;
    const q = newReviewQ.trim();
    const a = newReviewA.trim();
    if (!q || !a) return;
    await api.createReview(card.id, q, a);
    newReviewQ = '';
    newReviewA = '';
    reviews = await api.listReviewsForCard(card.id);
  }

  async function del() {
    if (!card) return;
    if (!confirm(`删除知识卡 "${card.title}" 吗？`)) return;
    await api.deleteCard(card.id);
    goto('/map');
  }

  const slotKeys: (keyof Slots)[] = [
    'definition', 'mechanism', 'complexity', 'comparison',
    'use_cases', 'interview_points', 'pitfalls', 'code',
  ];
</script>

{#if !card}
  <p class="muted">加载中…</p>
{:else}
  <div class="layout">
    <div class="main">
      <!-- Prev / Next nav strip -->
      <div class="navrow">
        <button class="ghost prev" onclick={() => go(prevCard)} disabled={!prevCard}
                title={prevCard ? `← ${prevCard.title}` : '没有上一张'}>
          ← {prevCard ? prevCard.title : '上一张'}
        </button>
        <div class="navmeta muted">
          {#if primaryTopicId}
            <a href="/topic/{primaryTopicId}">主题 {primaryTopicId}</a> ·
          {/if}
          {curIdx + 1} / {siblings.length}
        </div>
        <button class="ghost next" onclick={() => go(nextCard)} disabled={!nextCard}
                title={nextCard ? `${nextCard.title} →` : '没有下一张'}>
          {nextCard ? nextCard.title : '下一张'} →
        </button>
      </div>

      <header>
        <div class="row">
          <h1>{card.title}</h1>
          <span class="spacer"></span>
          {#if savedMsg}<span class="muted">{savedMsg}</span>{/if}
          <button class="primary" onclick={save} disabled={saving}>
            {saving ? '保存中…' : '保存'}
          </button>
          <button onclick={del}>删除</button>
        </div>
        <div class="meta muted">
          挂在主题: {card.topic_ids.join(', ')}
          · 创建 {new Date(card.created_at).toLocaleDateString()}
          · <kbd>←</kbd>/<kbd>→</kbd> 切卡 · <kbd>⌘←</kbd> 返回
        </div>
        <div class="aliases-row">
          <label for="aliases-input">别名（逗号分隔，进 FST 索引）</label>
          <input id="aliases-input" bind:value={aliasesText} placeholder="B+ tree, B-plus tree" />
        </div>
      </header>

      {#each slotKeys as key (key)}
        <section class="slot">
          <div class="slot-head">
            <h3>{SLOT_LABELS[key]}</h3>
            <button class="ghost" onclick={() => addFact(key)}>＋ 加一条</button>
          </div>
          {#if card.slots[key].length === 0}
            <p class="empty muted">（空）</p>
          {:else}
            {#each card.slots[key] as fact, i (i)}
              <div class="fact">
                <textarea
                  bind:value={fact.text}
                  rows={key === 'code' ? 6 : 2}
                  placeholder={SLOT_LABELS[key] + '…'}
                  class:code={key === 'code'}
                ></textarea>
                <button class="ghost del" onclick={() => removeFact(key, i)}>✕</button>
              </div>
            {/each}
          {/if}
        </section>
      {/each}
    </div>

    <aside class="side">
      <h3>面试官追问 <span class="muted small">(M3 接 LLM)</span></h3>
      <p class="muted small">
        M1 只能手动加题；M3 会基于卡内容自动生成问题树。
      </p>

      <div class="review-add">
        <h4>手动添加复习卡</h4>
        <input bind:value={newReviewQ} placeholder="问：例如 为什么数据库用 B+ 不用 B 树？" />
        <textarea bind:value={newReviewA} placeholder="答（要点）: 1) 范围查询… 2) IO… 3) ..." rows="4"></textarea>
        <button class="primary" onclick={addReview} disabled={!newReviewQ.trim() || !newReviewA.trim()}>
          + 加入复习
        </button>
      </div>

      <h4>已有复习卡 ({reviews.length})</h4>
      {#if reviews.length === 0}
        <p class="muted small">还没有。加几个面试官可能问的题吧。</p>
      {:else}
        <ul class="reviews">
          {#each reviews as r (r.id)}
            <li>
              <div class="q">{r.question}</div>
              <div class="muted small">
                下次复习 {new Date(r.fsrs.next_review).toLocaleDateString()}
                · 稳定度 {r.fsrs.stability.toFixed(1)}d
                · 复习 {r.fsrs.reps} 次
              </div>
            </li>
          {/each}
        </ul>
      {/if}
    </aside>
  </div>
{/if}

<style>
  .layout {
    display: grid;
    grid-template-columns: 1fr 360px;
    gap: 24px;
    max-width: 1200px;
    margin: 0 auto;
  }
  .main { min-width: 0; }

  .navrow {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    gap: 12px;
    align-items: center;
    margin-bottom: 16px;
    padding: 8px;
    background: var(--bg-elev);
    border: 1px solid var(--border);
    border-radius: var(--radius);
  }
  .navrow button {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 100%;
  }
  .navrow .prev { text-align: left; justify-self: start; }
  .navrow .next { text-align: right; justify-self: end; }
  .navrow .navmeta { font-size: 12px; white-space: nowrap; }
  .navrow button:disabled { opacity: 0.3; }

  header { margin-bottom: 24px; }
  h1 { margin: 0; font-size: 22px; }
  .meta { font-size: 12px; margin-top: 6px; }

  .aliases-row {
    margin-top: 12px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .aliases-row label { font-size: 11px; color: var(--fg-dim); text-transform: uppercase; letter-spacing: 0.5px; }

  .slot {
    background: var(--bg-elev);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 14px 16px;
    margin-bottom: 12px;
  }
  .slot-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }
  .slot-head h3 { margin: 0; font-size: 13px; }
  .empty { margin: 4px 0; font-size: 12px; }
  .fact { display: flex; gap: 6px; margin-bottom: 6px; align-items: flex-start; }
  .fact textarea { flex: 1; resize: vertical; }
  .fact textarea.code {
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 12px;
  }
  .del { color: var(--bad); padding: 4px 8px; }

  .side {
    background: var(--bg-elev);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 16px;
    height: fit-content;
    position: sticky;
    top: 16px;
  }
  .side h3 { margin: 0 0 8px; font-size: 13px; }
  .side h4 {
    margin: 16px 0 6px;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--fg-dim);
  }
  .small { font-size: 11px; }
  .review-add { display: flex; flex-direction: column; gap: 6px; margin-top: 8px; }

  .reviews { list-style: none; margin: 0; padding: 0; }
  .reviews li {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 8px 10px;
    margin-bottom: 6px;
  }
  .reviews .q { font-size: 13px; margin-bottom: 4px; }
</style>
