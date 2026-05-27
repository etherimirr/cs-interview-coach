<script lang="ts">
  import { api } from '$lib/api';
  import type { Suggestion } from '$lib/types';
  import { goto } from '$app/navigation';

  let query = $state('');
  let suggestions = $state<Suggestion[]>([]);
  let selectedIdx = $state(0);
  let inputEl: HTMLInputElement;
  let perfMs = $state<number | null>(null);

  async function onInput() {
    const q = query.trim();
    if (!q) { suggestions = []; perfMs = null; return; }
    const t0 = performance.now();
    suggestions = await api.suggestTitles(q, 12);
    perfMs = +(performance.now() - t0).toFixed(2);
    selectedIdx = 0;
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIdx = Math.min(selectedIdx + 1, suggestions.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIdx = Math.max(selectedIdx - 1, 0);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (suggestions.length > 0) {
        goto(`/card/${suggestions[selectedIdx].card_id}`);
      }
    } else if (e.key === 'Escape') {
      query = '';
      suggestions = [];
    }
  }

  function pick(s: Suggestion) {
    goto(`/card/${s.card_id}`);
  }
</script>

<div class="wrap">
  <div class="search-box">
    <input
      bind:this={inputEl}
      bind:value={query}
      oninput={onInput}
      onkeydown={onKeydown}
      data-global-search
      placeholder="输入概念名跳到知识卡  ·  ⌘K 任何地方唤起"
      autofocus
      spellcheck="false"
    />
    {#if perfMs !== null}
      <div class="perf muted">{perfMs} ms</div>
    {/if}
  </div>

  {#if suggestions.length > 0}
    <ul class="results">
      {#each suggestions as s, i (s.card_id + s.key)}
        <li class:selected={i === selectedIdx}>
          <button class="ghost" onclick={() => pick(s)}>
            <span class="title">{s.key}</span>
            <span class="muted card-id">{s.card_id.slice(0, 8)}</span>
          </button>
        </li>
      {/each}
    </ul>
  {:else if query.trim()}
    <div class="empty">
      <p class="muted">没有匹配的知识卡。</p>
      <p class="muted">M2 接 Tantivy 后会用 BM25 + 语义搜索兜底。</p>
    </div>
  {:else}
    <div class="empty">
      <p class="muted">从 <a href="/map">Map</a> 进任意分类节点，新建你的第一张知识卡。</p>
    </div>
  {/if}
</div>

<style>
  .wrap { max-width: 720px; margin: 40px auto; }
  .search-box {
    position: relative;
    display: flex;
    align-items: center;
  }
  .search-box input {
    width: 100%;
    font-size: 18px;
    padding: 14px 18px;
  }
  .perf {
    position: absolute;
    right: 14px;
    font-size: 11px;
    pointer-events: none;
  }
  .results {
    list-style: none;
    margin: 12px 0 0;
    padding: 0;
    background: var(--bg-elev);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    overflow: hidden;
  }
  .results li { border-bottom: 1px solid var(--border); }
  .results li:last-child { border-bottom: none; }
  .results li.selected { background: var(--accent-soft); }
  .results button {
    width: 100%;
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    padding: 10px 16px;
    border-radius: 0;
    text-align: left;
  }
  .title { font-size: 14px; }
  .card-id {
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 11px;
  }
  .empty {
    margin-top: 24px;
    text-align: center;
    line-height: 1.7;
  }
</style>
