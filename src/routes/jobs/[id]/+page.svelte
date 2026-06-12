<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { api } from '$lib/api';
  import type { Job, KnowledgeCard } from '$lib/types';

  let jobId = $derived(page.params.id!);
  let job = $state<Job | null>(null);
  let cards = $state<KnowledgeCard[]>([]);
  let loading = $state(true);
  let activeTab = $state<'cherry' | 'anchor' | 'topic' | 'all' | 'notes'>('cherry');

  // Normalized sets for quick categorization
  let cherrySet = $derived(new Set(
    (job?.cherry_picked_cards ?? []).map((s) => s.trim().toLowerCase())
  ));
  let anchorSet = $derived(new Set(
    (job?.my_anchors ?? []).map((s) => s.trim().toLowerCase())
  ));
  let topicSet = $derived(new Set(job?.relevant_topic_ids ?? []));

  function tagOf(c: KnowledgeCard): 'cherry' | 'anchor' | 'topic' {
    const title = c.title.trim().toLowerCase();
    const aliases = c.aliases.map((a) => a.trim().toLowerCase());
    if (cherrySet.has(title) || aliases.some((a) => cherrySet.has(a))) return 'cherry';
    if (anchorSet.has(title) || aliases.some((a) => anchorSet.has(a))) return 'anchor';
    return 'topic';
  }

  let cherryCards = $derived(cards.filter((c) => tagOf(c) === 'cherry'));
  let anchorCards = $derived(cards.filter((c) => tagOf(c) === 'anchor'));
  let topicCards = $derived(cards.filter((c) => tagOf(c) === 'topic'));

  onMount(async () => {
    [job, cards] = await Promise.all([api.getJob(jobId), api.listCardsForJob(jobId)]);
    loading = false;
  });

  // Reload when route changes
  $effect(() => {
    if (jobId && !loading) {
      loading = true;
      (async () => {
        [job, cards] = await Promise.all([api.getJob(jobId), api.listCardsForJob(jobId)]);
        loading = false;
      })();
    }
  });
</script>

{#if loading}
  <p class="muted">加载岗位…</p>
{:else if !job}
  <p class="muted">岗位不存在.</p>
{:else}
  <div class="wrap">
    <a href="/jobs" class="back muted">← 所有岗位</a>

    <header>
      <h1>{job.title}</h1>
      <div class="meta muted">
        {job.company}
        {#if job.level} · {job.level}{/if}
        {#if job.track} · <span class="track">{job.track}</span>{/if}
      </div>
    </header>

    {#if job.jd}
      <section class="jd">
        <h3>岗位描述</h3>
        <pre>{job.jd}</pre>
      </section>
    {/if}

    {#if job.hard_requirements.length > 0}
      <section>
        <h3>硬性门槛</h3>
        <ul class="reqs">
          {#each job.hard_requirements as r}
            <li>{r}</li>
          {/each}
        </ul>
      </section>
    {/if}

    <section class="cards">
      <h3>匹配的知识 ({cards.length})</h3>

      <div class="tabs">
        <button class:active={activeTab === 'cherry'} onclick={() => activeTab = 'cherry'}>
          🎯 重点必看 ({cherryCards.length})
        </button>
        <button class:active={activeTab === 'anchor'} onclick={() => activeTab = 'anchor'}>
          ⚓ 我的项目锚点 ({anchorCards.length})
        </button>
        <button class:active={activeTab === 'topic'} onclick={() => activeTab = 'topic'}>
          📚 相关主题 ({topicCards.length})
        </button>
        <button class:active={activeTab === 'all'} onclick={() => activeTab = 'all'}>
          全部 ({cards.length})
        </button>
        {#if job.notes}
          <button class:active={activeTab === 'notes'} onclick={() => activeTab = 'notes'}>
            📝 准备策略
          </button>
        {/if}
      </div>

      {#if activeTab === 'notes'}
        <div class="notes">
          <pre>{job.notes}</pre>
        </div>
      {:else}
        {@const shown =
          activeTab === 'cherry' ? cherryCards :
          activeTab === 'anchor' ? anchorCards :
          activeTab === 'topic'  ? topicCards :
          cards}
        {#if shown.length === 0}
          <p class="muted">这一类没有卡.</p>
        {:else}
          <ul class="card-list">
            {#each shown as c (c.id)}
              {@const tag = tagOf(c)}
              <li class="card-row" data-tag={tag}>
                <a href="/card/{c.id}">
                  <span class="tag-dot" title={
                    tag === 'cherry' ? '重点必看 (cherry-picked)' :
                    tag === 'anchor' ? '你的项目锚点 (my_anchors)' :
                    '通过主题匹配 (relevant_topic_ids)'
                  }></span>
                  <span class="title">{c.title}</span>
                  {#if c.aliases.length > 0}
                    <span class="muted aliases">({c.aliases.slice(0, 3).join(' / ')})</span>
                  {/if}
                  <span class="muted topics">
                    {c.topic_ids.join(', ')}
                  </span>
                </a>
              </li>
            {/each}
          </ul>
        {/if}
      {/if}
    </section>
  </div>
{/if}

<style>
  .wrap { max-width: 1000px; margin: 0 auto; }
  .back { font-size: 12px; }
  header { margin: 12px 0 24px; }
  h1 { margin: 0 0 4px; font-size: 22px; }
  .meta { font-size: 13px; }
  .track {
    display: inline-block;
    padding: 1px 8px;
    background: var(--accent-soft);
    color: var(--accent);
    border-radius: 999px;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  section { margin-bottom: 24px; }
  h3 {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: var(--fg-dim);
    border-bottom: 1px solid var(--border);
    padding-bottom: 6px;
    margin-bottom: 12px;
  }

  .jd pre,
  .notes pre {
    background: var(--bg-elev);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 14px 18px;
    white-space: pre-wrap;
    font-family: inherit;
    line-height: 1.6;
    font-size: 13px;
    margin: 0;
  }
  .reqs { margin: 0; padding-left: 20px; }
  .reqs li { font-size: 13px; margin-bottom: 4px; }

  .tabs {
    display: flex;
    gap: 6px;
    margin-bottom: 16px;
    flex-wrap: wrap;
  }
  .tabs button {
    padding: 6px 12px;
    font-size: 12px;
    border-radius: var(--radius);
  }
  .tabs button.active {
    background: var(--accent-soft);
    color: var(--fg);
    border-color: var(--accent);
  }

  .card-list { list-style: none; margin: 0; padding: 0; }
  .card-row {
    background: var(--bg-elev);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    margin-bottom: 6px;
  }
  .card-row a {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 14px;
    color: var(--fg);
  }
  .card-row a:hover { background: var(--bg-elev-2); text-decoration: none; }
  .tag-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .card-row[data-tag="cherry"] .tag-dot { background: var(--accent); }
  .card-row[data-tag="anchor"] .tag-dot { background: var(--good); }
  .card-row[data-tag="topic"]  .tag-dot { background: var(--fg-dim); }
  .title { font-size: 13px; }
  .aliases { font-size: 11px; }
  .topics {
    margin-left: auto;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 11px;
  }
</style>
