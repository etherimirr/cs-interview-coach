<script lang="ts">
  import '../app.css';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';

  let { children } = $props();

  const tabs = [
    { href: '/',       label: 'Search', key: 'search' },
    { href: '/map',    label: 'Map',    key: 'map'    },
    { href: '/tree',   label: 'Tree',   key: 'tree'   },
    { href: '/radial', label: 'Radial', key: 'radial' },
    { href: '/jobs',   label: 'Jobs',   key: 'jobs'   },
    { href: '/review', label: 'Review', key: 'review' },
  ];

  function isActive(href: string): boolean {
    if (href === '/') return page.url.pathname === '/';
    return page.url.pathname.startsWith(href);
  }

  function back() { history.back(); }
  function forward() { history.forward(); }

  // Track whether back/forward are available (approximate — browser doesn't
  // expose the actual stack, so we rely on a session counter)
  let canBack = $state(false);
  let canForward = $state(false);

  function refreshNav() {
    // window.history.length includes initial entry; > 1 means we've navigated
    canBack = window.history.length > 1;
    // No reliable forward signal in browsers; show button but it just no-ops
    // when nothing to go forward to.
    canForward = true;
  }

  onMount(() => {
    refreshNav();

    function onKey(e: KeyboardEvent) {
      // Ignore when typing in inputs/textareas
      const tag = (e.target as HTMLElement)?.tagName;
      const inField = tag === 'INPUT' || tag === 'TEXTAREA' || (e.target as HTMLElement)?.isContentEditable;

      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        goto('/');
        setTimeout(() => {
          const el = document.querySelector<HTMLInputElement>('input[data-global-search]');
          el?.focus();
          el?.select();
        }, 0);
        return;
      }

      // ⌘← / ⌘→ for back/forward
      if ((e.metaKey || e.altKey) && e.key === 'ArrowLeft' && !inField) {
        e.preventDefault();
        back();
        return;
      }
      if ((e.metaKey || e.altKey) && e.key === 'ArrowRight' && !inField) {
        e.preventDefault();
        forward();
        return;
      }
    }
    window.addEventListener('keydown', onKey);
    window.addEventListener('popstate', refreshNav);
    return () => {
      window.removeEventListener('keydown', onKey);
      window.removeEventListener('popstate', refreshNav);
    };
  });
</script>

<div class="app">
  <nav class="topbar">
    <div class="navbtns">
      <button class="ghost navbtn" onclick={back} title="返回 (⌘←)" disabled={!canBack}>←</button>
      <button class="ghost navbtn" onclick={forward} title="前进 (⌘→)">→</button>
    </div>
    <div class="brand">📚 CS Interview Coach</div>
    <div class="tabs">
      {#each tabs as t (t.key)}
        <a href={t.href} class:active={isActive(t.href)}>{t.label}</a>
      {/each}
    </div>
    <div class="spacer"></div>
    <div class="hint muted">
      <kbd>⌘K</kbd> 搜索 · <kbd>⌘←</kbd> 返回
    </div>
  </nav>
  <main>
    {@render children?.()}
  </main>
</div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }
  .topbar {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 10px 20px;
    background: var(--bg-elev);
    border-bottom: 1px solid var(--border);
  }
  .navbtns { display: flex; gap: 2px; }
  .navbtn {
    width: 28px;
    height: 28px;
    padding: 0;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-size: 14px;
    color: var(--fg-dim);
  }
  .navbtn:hover:not(:disabled) {
    color: var(--fg);
    background: var(--bg-elev-2);
  }
  .navbtn:disabled { opacity: 0.3; cursor: not-allowed; }
  .brand {
    font-weight: 600;
    font-size: 13px;
    letter-spacing: 0.5px;
  }
  .tabs { display: flex; gap: 4px; }
  .tabs a {
    color: var(--fg-dim);
    padding: 6px 12px;
    border-radius: var(--radius);
    font-size: 13px;
  }
  .tabs a:hover { background: var(--bg-elev-2); color: var(--fg); text-decoration: none; }
  .tabs a.active { color: var(--fg); background: var(--accent-soft); }
  .hint { font-size: 12px; }
  main {
    flex: 1;
    overflow: auto;
    padding: 24px;
  }
</style>
