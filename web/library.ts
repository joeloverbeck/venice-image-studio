type Entry = {id: string; created_at: string; prompt: string; model: string; model_name: string | null; status: string; outcome: string; result_count: number};

export function libraryView(api: (path: string) => Promise<any>, open: (id: string) => void, report: (error: unknown) => void) {
  const search = document.getElementById('library-search') as HTMLInputElement;
  const order = document.getElementById('library-order') as HTMLSelectElement;
  const list = document.getElementById('attempts')!;
  const count = document.getElementById('library-count')!;
  let revision = 0;
  let selected: string | null = null;
  async function refresh(id = selected) {
    selected = id;
    const pending = ++revision;
    try {
      const result = await api(`/api/library?${new URLSearchParams({search: search.value, order: order.value})}`);
      if (pending !== revision) return;
      count.textContent = result.attempts.length ? `${result.attempts.length} of ${result.total} attempts` : result.total ? 'No attempts match this search.' : 'Your saved results and failed attempts will appear here.';
      const existing = new Map(Array.from(list.children, node => [(node as HTMLElement).dataset.attempt, node as HTMLButtonElement]));
      result.attempts.forEach((a: Entry, index: number) => {
        const button = existing.get(a.id) || document.createElement('button');
        existing.delete(a.id);
        button.className = 'attempt'; button.dataset.attempt = a.id;
        button.setAttribute('aria-pressed', String(a.id === selected));
        const label = `${new Date(a.created_at).toLocaleString()} · ${a.model_name || a.model} (${a.model})\n${a.status} · ${a.outcome} · ${a.result_count} original${a.result_count === 1 ? '' : 's'}\n${a.prompt.slice(0, 180)}`;
        if (button.textContent !== label) button.textContent = label;
        button.onclick = () => { selected = a.id; open(a.id); void refresh(); };
        if (list.children[index] !== button) list.insertBefore(button, list.children[index] || null);
      });
      for (const button of existing.values()) button.remove();
    } catch (error) { if (pending === revision) report(error); }
  }
  search.addEventListener('input', () => void refresh());
  order.addEventListener('change', () => void refresh());
  return refresh;
}
