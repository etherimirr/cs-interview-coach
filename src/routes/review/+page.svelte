<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { api } from '$lib/api';
  import type { ReviewCard, Grade, KnowledgeCard } from '$lib/types';

  let queue = $state<ReviewCard[]>([]);
  let parentCards = $state<Record<string, KnowledgeCard | null>>({});
  let idx = $state(0);
  let showAnswer = $state(false);
  let loading = $state(true);
  let sessionDone = $state(0);

  const current = $derived(queue[idx] || null);
  const currentParent = $derived(current ? parentCards[current.knowledge_card_id] : null);

  async function load() {
    loading = true;
    try {
      queue = await api.dueReviews();
      idx = 0;
      showAnswer = false;
      // Preload parent KnowledgeCard titles
      const ids = [...new Set(queue.map((r) => r.knowledge_card_id))];
      const map: Record<string, KnowledgeCard | null> = {};
      await Promise.all(ids.map(async (id) => { map[id] = await api.getCard(id); }));
      parentCards = map;
    } finally {
      loading = false;
    }
  }

  onMount(load);

  async function rate(grade: Grade) {
    if (!current) return;
    await api.rateReview(current.id, grade);
    sessionDone += 1;
    if (idx + 1 < queue.length) {
      idx += 1;
      showAnswer = false;
    } else {
      queue = [];
      idx = 0;
    }
  }

  function reveal() { showAnswer = true; }

  function onKey(e: KeyboardEvent) {
    if (!current) return;
    if (!showAnswer) {
      if (e.key === ' ' || e.key === 'Enter') {
        e.preventDefault();
        reveal();
      }
      return;
    }
    if (e.key === '1') rate('again');
    else if (e.key === '2') rate('hard');
    else if (e.key === '3') rate('good');
    else if (e.key === '4') rate('easy');
  }

  onMount(() => {
    window.addEventListener('keydown', onKey);
  });
  onDestroy(() => {
    window.removeEventListener('keydown', onKey);
  });
</script>

<div class="wrap">
  {#if loading}
    <p class="muted">加载到期复习卡…</p>
  {:else if !current}
    <div class="done">
      <h2>🎉 今日复习完成</h2>
      <p class="muted">这次 session 完成了 {sessionDone} 张卡。</p>
      <button class="primary" onclick={load}>重新加载</button>
    </div>
  {:else}
    <div class="header muted">
      第 {idx + 1} / {queue.length} 张
      · 这次 session 已完成 {sessionDone}
      {#if currentParent}
        · 来自卡片 <a href="/card/{currentParent.id}">{currentParent.title}</a>
      {/if}
    </div>

    <div class="card">
      <div class="q">{current.question}</div>

      {#if !showAnswer}
        <button class="primary big" onclick={reveal}>
          显示答案 <kbd>Space</kbd>
        </button>
      {:else}
        <div class="a">{current.answer}</div>

        <div class="grades">
          <button class="grade again" onclick={() => rate('again')}>
            <div>Again</div><kbd>1</kbd>
          </button>
          <button class="grade hard" onclick={() => rate('hard')}>
            <div>Hard</div><kbd>2</kbd>
          </button>
          <button class="grade good" onclick={() => rate('good')}>
            <div>Good</div><kbd>3</kbd>
          </button>
          <button class="grade easy" onclick={() => rate('easy')}>
            <div>Easy</div><kbd>4</kbd>
          </button>
        </div>

        <div class="muted small fsrs">
          稳定度 {current.fsrs.stability.toFixed(1)}d
          · 难度 {current.fsrs.difficulty.toFixed(1)}
          · 复习 {current.fsrs.reps} 次 · 失败 {current.fsrs.lapses} 次
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .wrap { max-width: 720px; margin: 24px auto; }
  .header { margin-bottom: 16px; font-size: 12px; }
  .card {
    background: var(--bg-elev);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 32px 28px;
    min-height: 320px;
    display: flex;
    flex-direction: column;
    gap: 24px;
  }
  .q { font-size: 20px; line-height: 1.5; white-space: pre-wrap; }
  .a {
    font-size: 15px;
    line-height: 1.7;
    color: var(--fg-dim);
    white-space: pre-wrap;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 16px;
  }
  .big { font-size: 14px; padding: 12px 24px; align-self: center; }

  .grades { display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; }
  .grade {
    padding: 14px 8px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    align-items: center;
    font-weight: 500;
  }
  .grade.again { border-color: var(--bad); }
  .grade.again:hover { background: rgba(248, 113, 113, 0.15); }
  .grade.hard  { border-color: var(--warn); }
  .grade.hard:hover  { background: rgba(251, 191, 36, 0.15); }
  .grade.good  { border-color: var(--accent); }
  .grade.good:hover  { background: var(--accent-soft); }
  .grade.easy  { border-color: var(--good); }
  .grade.easy:hover  { background: rgba(74, 222, 128, 0.15); }

  .fsrs { text-align: center; font-size: 11px; }
  .small { font-size: 11px; }
  .done { text-align: center; padding: 60px 20px; }
  .done h2 { font-size: 20px; }
</style>
