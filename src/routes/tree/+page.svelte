<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import * as d3 from 'd3';
  import { api } from '$lib/api';
  import type { Taxonomy, Topic } from '$lib/types';

  type Node = {
    id: string;
    name: string;
    cards?: number;       // # of knowledge cards
    topicId?: string;     // leaf only
    children?: Node[];
    _children?: Node[];   // hidden (collapsed) children
    depth?: number;
  };

  let container: HTMLDivElement;
  let loading = $state(true);
  let totalCards = $state(0);
  let totalTopics = $state(0);

  onMount(async () => {
    const [taxonomy, cards] = await Promise.all([
      api.getTaxonomy(),
      api.listCards(),
    ]);
    totalCards = cards.length;

    // Count cards per leaf topicId
    const counts: Record<string, number> = {};
    for (const c of cards) {
      for (const t of c.topic_ids) counts[t] = (counts[t] || 0) + 1;
    }

    // Build hierarchical Node tree from taxonomy groups → topics → children
    const root = buildTree(taxonomy, counts);
    totalTopics = countLeaves(root);
    loading = false;
    requestAnimationFrame(() => render(root));
  });

  function buildTree(tax: Taxonomy, counts: Record<string, number>): Node {
    return {
      id: 'root',
      name: 'CS 面试知识',
      children: tax.groups.map((g) => {
        const groupTopics = tax.topics.filter((t) =>
          g.topics.map(String).includes(String(t.id))
        );
        return {
          id: 'group-' + g.id,
          name: g.id + '. ' + g.name,
          children: groupTopics.map((t: Topic) => ({
            id: 'topic-' + t.id,
            name: t.id + ' ' + t.name,
            children: t.children.map((c) => ({
              id: 'sub-' + c.id,
              name: c.id + ' ' + c.name,
              topicId: String(c.id),
              cards: counts[String(c.id)] || 0,
            })),
          })),
        };
      }),
    };
  }

  function countLeaves(n: Node): number {
    if (!n.children && !n._children) return 1;
    return (n.children || n._children || []).reduce((s, c) => s + countLeaves(c), 0);
  }

  // ───────────────────────────── D3 render ─────────────────────────────

  function render(rootData: Node) {
    const margin = { top: 20, right: 240, bottom: 20, left: 100 };
    const dx = 22;
    const dy = 180;

    container.innerHTML = '';
    const width = container.clientWidth;

    const tree = d3.tree<Node>().nodeSize([dx, dy]);
    const diagonal = d3
      .linkHorizontal<any, any>()
      .x((d: any) => d.y)
      .y((d: any) => d.x);

    const root: any = d3.hierarchy<Node>(rootData);
    root.x0 = 0;
    root.y0 = 0;
    root.descendants().forEach((d: any, i: number) => {
      d.id = d.data.id || ('node-' + i);
      d._children = d.children;
      // Collapse below depth 1 by default for a clean initial view
      if (d.depth > 1) d.children = null;
    });

    const svg = d3
      .select(container)
      .append('svg')
      .attr('width', width)
      .attr('height', dx * 30)
      .attr('viewBox', [-margin.left, -margin.top, width, dx * 30] as any)
      .style('font', '12px -apple-system, BlinkMacSystemFont, sans-serif')
      .style('user-select', 'none');

    const gLink = svg
      .append('g')
      .attr('fill', 'none')
      .attr('stroke', '#2a313d')
      .attr('stroke-width', 1.5);

    const gNode = svg
      .append('g')
      .attr('cursor', 'pointer')
      .attr('pointer-events', 'all');

    update(root, root);

    function update(event: any, source: any) {
      const duration = event?.altKey ? 2500 : 250;
      const nodes = root.descendants().reverse();
      const links = root.links();

      tree(root);

      // Vertical extent
      let left: any = root, right: any = root;
      root.eachBefore((node: any) => {
        if (node.x < left.x) left = node;
        if (node.x > right.x) right = node;
      });
      const height = right.x - left.x + margin.top + margin.bottom + 40;

      svg.transition().duration(duration)
        .attr('height', height)
        .attr('viewBox', [-margin.left, left.x - margin.top, width, height] as any);

      // Nodes
      const node = gNode.selectAll<SVGGElement, any>('g').data(nodes, (d: any) => d.id);

      const nodeEnter = node
        .enter().append('g')
        .attr('transform', () => `translate(${source.y0},${source.x0})`)
        .attr('fill-opacity', 0)
        .attr('stroke-opacity', 0)
        .on('click', (ev: any, d: any) => {
          // Leaf with topicId → navigate
          if (d.data.topicId && !d._children) {
            goto(`/topic/${d.data.topicId}`);
            return;
          }
          // Otherwise toggle
          d.children = d.children ? null : d._children;
          update(ev, d);
        });

      nodeEnter.append('circle')
        .attr('r', (d: any) => nodeRadius(d))
        .attr('fill', (d: any) => nodeFill(d))
        .attr('stroke', (d: any) => nodeStroke(d))
        .attr('stroke-width', 1.5);

      nodeEnter.append('text')
        .attr('dy', '0.32em')
        .attr('x', (d: any) => (d._children ? -10 : 10))
        .attr('text-anchor', (d: any) => (d._children ? 'end' : 'start'))
        .text((d: any) => labelFor(d))
        .attr('fill', '#e6e9ef')
        .clone(true).lower()
        .attr('stroke-linejoin', 'round')
        .attr('stroke-width', 3)
        .attr('stroke', '#0f1115');

      node.merge(nodeEnter as any)
        .transition().duration(duration)
        .attr('transform', (d: any) => `translate(${d.y},${d.x})`)
        .attr('fill-opacity', 1)
        .attr('stroke-opacity', 1);

      node.exit()
        .transition().duration(duration).remove()
        .attr('transform', () => `translate(${source.y},${source.x})`)
        .attr('fill-opacity', 0)
        .attr('stroke-opacity', 0);

      // Links
      const link = gLink.selectAll<SVGPathElement, any>('path').data(links, (d: any) => d.target.id);

      const linkEnter = link.enter().append('path')
        .attr('d', () => {
          const o = { x: source.x0, y: source.y0 };
          return diagonal({ source: o, target: o });
        });

      link.merge(linkEnter as any).transition().duration(duration)
        .attr('d', diagonal as any);

      link.exit().transition().duration(duration).remove()
        .attr('d', () => {
          const o = { x: source.x, y: source.y };
          return diagonal({ source: o, target: o });
        });

      root.eachBefore((d: any) => { d.x0 = d.x; d.y0 = d.y; });
    }
  }

  function nodeRadius(d: any): number {
    if (d.data.topicId !== undefined) {
      const n = d.data.cards || 0;
      return 4 + Math.min(10, Math.sqrt(n) * 2);
    }
    return d._children ? 6 : 4;
  }
  function nodeFill(d: any): string {
    if (d.data.topicId !== undefined) {
      const n = d.data.cards || 0;
      if (n === 0) return '#0f1115';
      if (n < 3) return '#4f8eff';
      if (n < 6) return '#22c55e';
      return '#facc15';
    }
    return d._children ? '#4f8eff' : '#1e242f';
  }
  function nodeStroke(d: any): string {
    if (d.data.topicId !== undefined && (d.data.cards || 0) === 0) return '#3a414d';
    return d._children ? '#4f8eff' : '#9aa3b2';
  }
  function labelFor(d: any): string {
    if (d.data.topicId !== undefined) {
      const n = d.data.cards || 0;
      return n > 0 ? `${d.data.name}  ·  ${n}` : d.data.name;
    }
    return d.data.name;
  }
</script>

<div class="wrap">
  <header>
    <div>
      <h1>知识树</h1>
      <p class="muted">
        {totalTopics} 个叶节点 · {totalCards} 张知识卡 · 点击节点展开/收起，点叶子进入主题
      </p>
    </div>
    <div class="legend">
      <span><span class="dot" style="background:#0f1115;border:1px solid #3a414d"></span>空</span>
      <span><span class="dot" style="background:#4f8eff"></span>1-2 张</span>
      <span><span class="dot" style="background:#22c55e"></span>3-5 张</span>
      <span><span class="dot" style="background:#facc15"></span>6+ 张</span>
    </div>
  </header>

  {#if loading}
    <p class="muted">加载分类树…</p>
  {/if}
  <div bind:this={container} class="tree"></div>
</div>

<style>
  .wrap { max-width: 1400px; margin: 0 auto; }
  header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    margin-bottom: 16px;
  }
  h1 { margin: 0 0 4px; font-size: 22px; }
  .legend {
    display: flex;
    gap: 12px;
    font-size: 11px;
    color: var(--fg-dim);
    align-items: center;
  }
  .dot {
    display: inline-block;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    margin-right: 4px;
    vertical-align: middle;
  }
  .tree {
    background: var(--bg-elev);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 16px 8px;
    overflow: auto;
    min-height: 600px;
  }
  :global(.tree svg text) { pointer-events: none; }
</style>
