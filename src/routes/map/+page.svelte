<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import type { Taxonomy, Topic } from '$lib/types';

  let taxonomy = $state<Taxonomy | null>(null);
  let expanded = $state<Set<string>>(new Set());
  let cardCounts = $state<Record<string, number>>({});

  onMount(async () => {
    taxonomy = await api.getTaxonomy();
    const cards = await api.listCards();
    const counts: Record<string, number> = {};
    for (const c of cards) {
      for (const t of c.topic_ids) counts[t] = (counts[t] || 0) + 1;
    }
    cardCounts = counts;
  });

  function toggleTopic(id: string) {
    const s = new Set(expanded);
    if (s.has(id)) s.delete(id); else s.add(id);
    expanded = s;
  }

  function topicsOfGroup(g_topic_ids: number[]): Topic[] {
    if (!taxonomy) return [];
    const wanted = new Set(g_topic_ids.map(String));
    return taxonomy.topics.filter((t) => wanted.has(String(t.id)));
  }
</script>

<div class="wrap">
  {#if !taxonomy}
    <p class="muted">加载分类树…</p>
  {:else}
    <header>
      <h1>知识地图</h1>
      <p class="muted">
        v{taxonomy.version} · {taxonomy.topics.length} 顶层主题 · 锁死前 {taxonomy.locked_levels} 级
      </p>
    </header>

    {#each taxonomy.groups as g (g.id)}
      <section class="group">
        <h2><span class="grp-id">{g.id}.</span> {g.name}</h2>
        <div class="topics">
          {#each topicsOfGroup(g.topics) as t (t.id)}
            <div class="topic">
              <button class="ghost topic-head" onclick={() => toggleTopic(String(t.id))}>
                <span class="caret">{expanded.has(String(t.id)) ? '▾' : '▸'}</span>
                <span class="topic-id">{t.id}</span>
                <span class="topic-name">{t.name}</span>
                {#if t.short}<span class="muted topic-short">{t.short}</span>{/if}
              </button>
              {#if expanded.has(String(t.id))}
                <ul class="children">
                  {#each t.children as c (c.id)}
                    {@const ckey = String(c.id)}
                    {@const cnt = cardCounts[ckey] || 0}
                    <li>
                      <a href="/topic/{ckey}">
                        <span class="topic-id">{c.id}</span>
                        <span>{c.name}</span>
                        {#if c.hint}<span class="muted hint">{c.hint}</span>{/if}
                        {#if cnt > 0}<span class="badge">{cnt}</span>{/if}
                      </a>
                    </li>
                  {/each}
                </ul>
              {/if}
            </div>
          {/each}
        </div>
      </section>
    {/each}
  {/if}
</div>

<style>
  .wrap { max-width: 1000px; margin: 0 auto; }
  header { margin-bottom: 28px; }
  h1 { margin: 0 0 4px; font-size: 24px; }
  .group { margin-bottom: 32px; }
  h2 {
    font-size: 14px;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: var(--fg-dim);
    border-bottom: 1px solid var(--border);
    padding-bottom: 6px;
    margin-bottom: 14px;
  }
  .grp-id { color: var(--accent); margin-right: 6px; }

  .topics { display: grid; grid-template-columns: repeat(2, 1fr); gap: 8px; }

  .topic {
    background: var(--bg-elev);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    overflow: hidden;
  }
  .topic-head {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 14px;
    border-radius: 0;
    text-align: left;
    border: none;
  }
  .caret { width: 12px; color: var(--fg-dim); }
  .topic-id {
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 12px;
    color: var(--fg-dim);
    min-width: 28px;
  }
  .topic-name { font-weight: 500; }
  .topic-short {
    margin-left: auto;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .children {
    list-style: none;
    margin: 0;
    padding: 4px 0 8px 32px;
    background: var(--bg);
  }
  .children li a {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    color: var(--fg);
    border-radius: var(--radius);
    font-size: 13px;
  }
  .children li a:hover { background: var(--bg-elev); text-decoration: none; }
  .hint {
    margin-left: auto;
    font-size: 11px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 50%;
  }
  .badge {
    background: var(--accent);
    color: white;
    border-radius: 999px;
    font-size: 10px;
    padding: 1px 7px;
    min-width: 18px;
    text-align: center;
  }
</style>
