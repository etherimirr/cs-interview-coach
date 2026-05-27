<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { api } from '$lib/api';
  import type { KnowledgeCard, Taxonomy, SubTopic } from '$lib/types';

  let topicId = $derived(page.params.topicId!);
  let cards = $state<KnowledgeCard[]>([]);
  let topicInfo = $state<{ parent: string; sub: SubTopic } | null>(null);
  let newTitle = $state('');
  let creating = $state(false);

  async function load() {
    const [tax, list] = await Promise.all([
      api.getTaxonomy(),
      api.listCardsByTopic(topicId),
    ]);
    cards = list;
    topicInfo = findTopic(tax, topicId);
  }

  function findTopic(tax: Taxonomy, id: string) {
    for (const t of tax.topics) {
      for (const c of t.children) {
        if (String(c.id) === id) return { parent: t.name, sub: c };
      }
    }
    return null;
  }

  onMount(load);

  async function createCard() {
    const title = newTitle.trim();
    if (!title) return;
    creating = true;
    try {
      const card = await api.createCard(title, [topicId]);
      goto(`/card/${card.id}`);
    } finally {
      creating = false;
    }
  }
</script>

<div class="wrap">
  <a href="/map" class="back muted">← 返回 Map</a>

  {#if topicInfo}
    <header>
      <div class="muted">{topicInfo.parent}</div>
      <h1><span class="muted">{topicId}</span> {topicInfo.sub.name}</h1>
      {#if topicInfo.sub.hint}
        <p class="hint muted">{topicInfo.sub.hint}</p>
      {/if}
    </header>
  {/if}

  <section class="new">
    <h3>新建知识卡</h3>
    <form
      class="row"
      onsubmit={(e) => { e.preventDefault(); createCard(); }}
    >
      <input
        bind:value={newTitle}
        placeholder="例如：B+ 树 / Wide&Deep / TCP 三次握手"
        disabled={creating}
        style="flex: 1"
      />
      <button class="primary" type="submit" disabled={creating || !newTitle.trim()}>
        {creating ? '创建中…' : '创建'}
      </button>
    </form>
  </section>

  <section class="list">
    <h3>已有知识卡 <span class="muted">({cards.length})</span></h3>
    {#if cards.length === 0}
      <p class="muted">还没有卡。从上面输入第一个概念吧。</p>
    {:else}
      <ul>
        {#each cards as c (c.id)}
          <li>
            <a href="/card/{c.id}">
              <span class="title">{c.title}</span>
              {#if c.aliases.length > 0}
                <span class="muted aliases">({c.aliases.join(' / ')})</span>
              {/if}
              <span class="muted ts">{new Date(c.updated_at).toLocaleDateString()}</span>
            </a>
          </li>
        {/each}
      </ul>
    {/if}
  </section>
</div>

<style>
  .wrap { max-width: 800px; margin: 0 auto; }
  .back { font-size: 12px; }
  header { margin: 16px 0 28px; }
  h1 { margin: 4px 0; font-size: 22px; }
  h1 .muted {
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 16px;
    margin-right: 8px;
  }
  .hint { font-size: 13px; margin: 4px 0 0; }
  section { margin-bottom: 28px; }
  h3 { font-size: 12px; text-transform: uppercase; letter-spacing: 1px; color: var(--fg-dim); }

  ul { list-style: none; margin: 0; padding: 0; }
  li {
    border: 1px solid var(--border);
    background: var(--bg-elev);
    border-radius: var(--radius);
    margin-bottom: 6px;
  }
  li a {
    display: flex;
    gap: 8px;
    align-items: baseline;
    padding: 10px 14px;
    color: var(--fg);
  }
  li a:hover { background: var(--bg-elev-2); text-decoration: none; }
  .title { font-weight: 500; }
  .aliases { font-size: 12px; }
  .ts { margin-left: auto; font-size: 11px; }
</style>
