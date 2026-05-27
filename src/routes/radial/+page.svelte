<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import * as d3 from 'd3';
  import { api } from '$lib/api';
  import type { Taxonomy, Topic } from '$lib/types';

  type Node = {
    id: string;
    name: string;
    short?: string;
    cards?: number;
    topicId?: string;
    children?: Node[];
    _children?: Node[];
  };

  let container: HTMLDivElement;
  let loading = $state(true);
  let totalCards = $state(0);
  let activeNode = $state<string>('');

  onMount(async () => {
    const [taxonomy, cards] = await Promise.all([
      api.getTaxonomy(),
      api.listCards(),
    ]);
    totalCards = cards.length;

    const counts: Record<string, number> = {};
    for (const c of cards) {
      for (const t of c.topic_ids) counts[t] = (counts[t] || 0) + 1;
    }

    const root = buildTree(taxonomy, counts);
    loading = false;
    requestAnimationFrame(() => render(root));
  });

  function buildTree(tax: Taxonomy, counts: Record<string, number>): Node {
    return {
      id: 'root',
      name: 'CS',
      children: tax.groups.map((g) => {
        const groupTopics = tax.topics.filter((t) =>
          g.topics.map(String).includes(String(t.id))
        );
        return {
          id: 'group-' + g.id,
          name: g.id + '. ' + g.name,
          short: g.id,
          children: groupTopics.map((t: Topic) => ({
            id: 'topic-' + t.id,
            name: String(t.id) + ' ' + t.name,
            short: t.short,
            children: t.children.map((c) => ({
              id: 'sub-' + c.id,
              name: String(c.id) + ' ' + c.name,
              topicId: String(c.id),
              cards: counts[String(c.id)] || 0,
            })),
          })),
        };
      }),
    };
  }

  // ───────────── D3 radial render ─────────────

  function render(rootData: Node) {
    container.innerHTML = '';

    const width = Math.max(900, container.clientWidth);
    const height = Math.max(900, container.clientWidth);
    const radius = Math.min(width, height) / 2 - 120;

    const tree = d3.cluster<Node>()
      .size([2 * Math.PI, radius])
      .separation((a, b) => (a.parent == b.parent ? 1 : 2) / Math.max(1, a.depth));

    const root: any = d3.hierarchy<Node>(rootData);
    root.descendants().forEach((d: any) => {
      d._children = d.children;
      // Default: show root + group + topic. Hide topic.children (3rd level).
      if (d.depth >= 2) d.children = null;
    });

    const svg = d3.select(container)
      .append('svg')
      .attr('viewBox', [-width / 2, -height / 2, width, height] as any)
      .attr('width', width)
      .attr('height', height)
      .style('font', '11px -apple-system, BlinkMacSystemFont, sans-serif')
      .style('user-select', 'none')
      .style('max-width', '100%')
      .style('height', 'auto');

    // Zoom/pan
    const g = svg.append('g');
    svg.call(
      d3.zoom<SVGSVGElement, unknown>()
        .scaleExtent([0.4, 4])
        .on('zoom', (ev) => g.attr('transform', ev.transform.toString())) as any
    );

    const gLink = g.append('g')
      .attr('fill', 'none')
      .attr('stroke-opacity', 0.5)
      .attr('stroke-linecap', 'round');

    const gNode = g.append('g').attr('cursor', 'pointer');

    update(root);

    function update(source: any) {
      tree(root);

      const nodes = root.descendants();
      const links = root.links();

      // ---- links ----
      const link = gLink.selectAll<SVGPathElement, any>('path')
        .data(links, (d: any) => d.target.data.id);

      link.exit().remove();

      const linkEnter = link.enter().append('path')
        .attr('stroke', '#2a313d')
        .attr('stroke-width', 1.4);

      link.merge(linkEnter as any)
        .transition().duration(350)
        .attr('d', d3.linkRadial<any, any>()
          .angle((d: any) => d.x)
          .radius((d: any) => d.y) as any);

      // ---- nodes ----
      const node = gNode.selectAll<SVGGElement, any>('g.node')
        .data(nodes, (d: any) => d.data.id);

      node.exit().remove();

      const nodeEnter = node.enter().append('g')
        .attr('class', 'node')
        .on('click', (ev: any, d: any) => {
          if (d.data.topicId && (!d._children || d._children.length === 0)) {
            goto(`/topic/${d.data.topicId}`);
            return;
          }
          // toggle subtree
          d.children = d.children ? null : d._children;
          activeNode = d.data.id;
          update(d);
        });

      nodeEnter.append('circle')
        .attr('fill', (d: any) => nodeFill(d))
        .attr('stroke', (d: any) => nodeStroke(d))
        .attr('stroke-width', 1.5)
        .attr('r', (d: any) => nodeRadius(d));

      nodeEnter.append('text')
        .attr('transform', (d: any) =>
          `rotate(${(d.x * 180) / Math.PI - 90}) translate(${d.y + 10},0) ` +
          `rotate(${d.x >= Math.PI ? 180 : 0})`)
        .attr('dy', '0.31em')
        .attr('x', (d: any) => (d.x < Math.PI === !d.children ? 6 : -6))
        .attr('text-anchor', (d: any) => (d.x < Math.PI === !d.children ? 'start' : 'end'))
        .attr('fill', '#e6e9ef')
        .text((d: any) => labelFor(d))
        .clone(true).lower()
        .attr('stroke', '#0f1115')
        .attr('stroke-width', 3.5)
        .attr('stroke-linejoin', 'round');

      // Update existing
      node.merge(nodeEnter as any)
        .transition().duration(350)
        .attr('transform', (d: any) =>
          `rotate(${(d.x * 180) / Math.PI - 90}) translate(${d.y},0)`);

      node.merge(nodeEnter as any)
        .select('circle')
        .attr('r', (d: any) => nodeRadius(d))
        .attr('fill', (d: any) => nodeFill(d))
        .attr('stroke', (d: any) => nodeStroke(d));
    }
  }

  function nodeRadius(d: any): number {
    if (d.data.topicId !== undefined) {
      const n = d.data.cards || 0;
      return 3 + Math.min(8, Math.sqrt(n) * 1.8);
    }
    if (d.depth === 0) return 8;       // root
    if (d.depth === 1) return 7;       // group
    return d._children ? 5 : 3;        // topic / sub
  }
  function nodeFill(d: any): string {
    if (d.data.topicId !== undefined) {
      const n = d.data.cards || 0;
      if (n === 0) return '#0f1115';
      if (n < 3) return '#4f8eff';
      if (n < 6) return '#22c55e';
      return '#facc15';
    }
    if (d.depth === 0) return '#9aa3b2';
    if (d.depth === 1) return '#4f8eff';
    return d._children ? (d.children ? '#22c55e' : '#4f8eff') : '#1e242f';
  }
  function nodeStroke(d: any): string {
    if (d.data.topicId !== undefined && (d.data.cards || 0) === 0) return '#3a414d';
    return '#4f8eff';
  }
  function labelFor(d: any): string {
    if (d.data.topicId !== undefined) {
      const n = d.data.cards || 0;
      return n > 0 ? `${d.data.name} · ${n}` : d.data.name;
    }
    if (d.depth === 0) return 'CS 面试';
    return d.data.name;
  }
</script>

<div class="wrap">
  <header>
    <div>
      <h1>径向知识树</h1>
      <p class="muted">
        {totalCards} 张卡 · 点节点展开/收起 · 点叶子进主题 · 拖拽平移 · 滚轮缩放
      </p>
    </div>
    <div class="legend">
      <span><span class="dot" style="background:#9aa3b2"></span>根</span>
      <span><span class="dot" style="background:#4f8eff"></span>分组/有内容</span>
      <span><span class="dot" style="background:#22c55e"></span>已展开</span>
      <span><span class="dot" style="background:#facc15"></span>卡 ≥ 6</span>
      <span><span class="dot" style="background:#0f1115;border:1px solid #3a414d"></span>空叶</span>
    </div>
  </header>

  {#if loading}
    <p class="muted">加载…</p>
  {/if}
  <div bind:this={container} class="radial"></div>
</div>

<style>
  .wrap { max-width: 1400px; margin: 0 auto; }
  header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    margin-bottom: 12px;
    flex-wrap: wrap;
    gap: 12px;
  }
  h1 { margin: 0 0 4px; font-size: 22px; }
  .legend {
    display: flex;
    gap: 12px;
    font-size: 11px;
    color: var(--fg-dim);
    align-items: center;
    flex-wrap: wrap;
  }
  .dot {
    display: inline-block;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    margin-right: 4px;
    vertical-align: middle;
  }
  .radial {
    background: var(--bg-elev);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 8px;
    overflow: hidden;
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 700px;
  }
</style>
