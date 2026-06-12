<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import type { Job } from '$lib/types';

  let jobs = $state<Job[]>([]);
  let loading = $state(true);

  onMount(async () => {
    jobs = await api.listJobs();
    loading = false;
  });
</script>

<div class="wrap">
  <header>
    <h1>岗位查询</h1>
    <p class="muted">
      点一个岗位切进去, 看所有跟这个岗位匹配的知识卡 + 你的项目锚点.
    </p>
  </header>

  {#if loading}
    <p class="muted">加载岗位库…</p>
  {:else if jobs.length === 0}
    <p class="muted">还没有岗位. 编辑 <code>seed/jobs.yaml</code> 添加.</p>
  {:else}
    <ul class="job-list">
      {#each jobs as job (job.id)}
        <li>
          <a href="/jobs/{job.id}">
            <div class="title-row">
              <span class="title">{job.title}</span>
              <span class="muted track">{job.track}</span>
            </div>
            <div class="meta muted">
              {job.company}
              {#if job.level} · {job.level}{/if}
            </div>
            {#if job.jd}
              <p class="jd muted">{job.jd.split('\n')[0]}</p>
            {/if}
            <div class="counts muted">
              {job.relevant_topic_ids.length} 主题 ·
              {job.cherry_picked_cards.length} 重点卡 ·
              {job.my_anchors.length} 项目锚点
            </div>
          </a>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .wrap { max-width: 900px; margin: 0 auto; }
  header { margin-bottom: 24px; }
  h1 { margin: 0 0 4px; font-size: 22px; }
  .job-list { list-style: none; margin: 0; padding: 0; }
  .job-list li {
    background: var(--bg-elev);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    margin-bottom: 10px;
  }
  .job-list a {
    display: block;
    padding: 14px 18px;
    color: var(--fg);
  }
  .job-list a:hover { background: var(--bg-elev-2); text-decoration: none; }
  .title-row {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
  }
  .title { font-weight: 500; font-size: 15px; }
  .track {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  .meta { font-size: 12px; margin-top: 4px; }
  .jd { font-size: 13px; margin: 8px 0; line-height: 1.5; }
  .counts { font-size: 11px; margin-top: 4px; }
  code {
    background: var(--bg-elev-2);
    padding: 1px 6px;
    border-radius: 4px;
    font-size: 12px;
  }
</style>
