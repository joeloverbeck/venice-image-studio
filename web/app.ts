export {};
type Draft = {
  id: string;
  prompt: string;
  safe_mode: boolean;
  model: string;
  parent_id: string | null;
};
type Attempt = {
  id: string;
  created_at: string;
  status: string;
  summary: string;
  prompt: string;
  results: Array<{
    id: string;
    width: number;
    height: number;
    mime: string;
    bytes: number;
    sha256: string;
  }>;
};
const el = <T extends HTMLElement>(id: string) =>
  document.getElementById(id) as T;
const prompt = el<HTMLTextAreaElement>("prompt"),
  safe = el<HTMLInputElement>("safe"),
  generate = el<HTMLButtonElement>("generate");
let state: any,
  current: Draft,
  selected: string | null = null,
  dirty = false,
  saving: Promise<void> = Promise.resolve();
async function api(path: string, body?: unknown, method = "POST") {
  const response = await fetch(
    path,
    body === undefined
      ? {}
      : {
          method,
          headers: {
            "Content-Type": "application/json",
            "X-Studio-Token": state?.token || "",
          },
          body: JSON.stringify(body),
        },
  );
  const value = await response.json();
  if (!response.ok) throw Error(value.error || "Studio request failed.");
  return value;
}
function notice(error: unknown) {
  el("notice").textContent = String(
    error instanceof Error ? error.message : error,
  );
}
function counter() {
  const model = state?.catalog?.models?.find(
    (m: any) => m.id === current?.model,
  );
  const count = Array.from(prompt.value).length;
  el("count").textContent =
    `${count.toLocaleString()} Unicode scalar characters · advertised limit ${model?.model_spec?.constraints?.promptCharacterLimit?.toLocaleString() || "unknown"}` +
    (count > 7500
      ? " · Above the generic 7,500-character schema; provider acceptance is unverified."
      : "");
}
function edited() {
  if (!dirty) {
    current = { ...current, id: crypto.randomUUID() };
    dirty = true;
  }
  current.prompt = prompt.value;
  current.safe_mode = safe.checked;
  counter();
  generate.disabled = true;
  el("draft-status").textContent = "Saving draft…";
  saving = saving
    .catch(() => {})
    .then(async () => {
      const snapshot = { ...current };
      try {
        await api("/api/draft", snapshot, "PUT");
        if (
          snapshot.prompt === current.prompt &&
          snapshot.safe_mode === current.safe_mode
        ) {
          dirty = false;
          el("draft-status").textContent = "Draft saved locally.";
          render();
        }
      } catch (e) {
        notice(e);
        el("draft-status").textContent =
          "Draft not saved. Keep this window open.";
      }
    });
}
prompt.addEventListener("input", edited);
safe.addEventListener("change", edited);
function render() {
  const model = state.catalog?.models?.find((m: any) => m.id === current.model);
  const price = model?.model_spec?.pricing?.generation?.usd;
  el("model").textContent =
    `${model?.model_spec?.name || current.model} (${current.model}) · 1024 × 1024 · one image · PNG`;
  el("estimate").textContent =
    price === undefined
      ? "Estimate unavailable. Refresh before generation."
      : `Estimate: $${price} USD · Venice catalog fetched ${state.catalog.fetched_at}. Estimate is not a charge or spending cap.`;
  el("library-path").textContent = `Library: ${state.library || "unavailable"}`;
  counter();
  generate.disabled =
    Boolean(state.inflight) ||
    dirty ||
    !state.credential_present ||
    Boolean(state.setup_error) ||
    state.attempts.some((a: Attempt) => a.id === current.id);
  const errors = [
    state.setup_error,
    state.catalog_error,
    !state.credential_present
      ? "Set VENICE_API_KEY in the private .env or environment, then restart Studio."
      : null,
  ].filter(Boolean);
  if (errors.length) notice(errors.join(" "));
  const list = el("attempts");
  list.replaceChildren();
  state.attempts.sort((a: Attempt, b: Attempt) =>
    b.created_at.localeCompare(a.created_at),
  );
  for (const a of state.attempts as Attempt[]) {
    const button = document.createElement("button");
    button.className = "attempt";
    button.textContent = `${new Date(a.created_at).toLocaleString()} · ${a.status} · ${a.prompt.slice(0, 70)}`;
    button.onclick = () => {
      selected = a.id;
      show(a);
    };
    list.append(button);
  }
  const shown =
    state.attempts.find((a: Attempt) => a.id === selected) || state.attempts[0];
  if (shown) show(shown);
}
function show(a: Attempt) {
  selected = a.id;
  el("result-status").textContent = `${a.status}: ${a.summary}`;
  el("recipe").textContent = JSON.stringify(a, null, 2);
  const images = el("images");
  if (
    images.dataset.attempt !== a.id ||
    images.childElementCount !== a.results.length
  ) {
    images.dataset.attempt = a.id;
    images.replaceChildren();
    for (const r of a.results) {
      const figure = document.createElement("figure"),
        image = document.createElement("img"),
        caption = document.createElement("figcaption"),
        download = document.createElement("a");
      image.src = `/api/images/${r.id}`;
      image.alt = "Generated original";
      caption.textContent = `${r.width} × ${r.height} · ${r.mime} · ${r.bytes.toLocaleString()} bytes`;
      download.href = image.src;
      download.download = `${r.id}.${r.mime.split("/")[1]}`;
      download.className = "download";
      download.textContent = "Download original";
      figure.append(image, caption, download);
      images.append(figure);
    }
  }
  const actions = el("result-actions");
  actions.replaceChildren();
  const reuse = document.createElement("button");
  reuse.className = "secondary";
  reuse.textContent = "Use recipe as new draft";
  reuse.onclick = async () => {
    try {
      await saving;
      current = await api(`/api/attempts/${a.id}/reuse`, {});
      prompt.value = current.prompt;
      safe.checked = current.safe_mode;
      dirty = false;
      el("draft-status").textContent =
        "New draft saved. Nothing has been generated.";
      render();
      prompt.focus();
    } catch (e) {
      notice(e);
    }
  };
  actions.append(reuse);
  if (a.status === "save_failed") {
    const save = document.createElement("button");
    save.textContent = "Retry local save";
    save.onclick = async () => {
      try {
        await api(`/api/attempts/${a.id}/save`, {});
        await reload();
      } catch (e) {
        notice(e);
      }
    };
    actions.append(save);
  }
}
async function reload() {
  state = await api("/api/state");
  render();
}
generate.onclick = async () => {
  generate.disabled = true;
  try {
    await saving;
    const a = await api("/api/generate", { draft_id: current.id, catalog_fetched_at: state.catalog.fetched_at });
    selected = a.id;
    await reload();
  } catch (e) {
    render();
    notice(e);
  }
};
el("refresh").onclick = async () => {
  try {
    state = await api("/api/catalog/refresh", {});
    el("notice").textContent = "";
    render();
  } catch (e) {
    notice(e);
  }
};
(async () => {
  try {
    state = await api("/api/state");
    current = state.draft;
    prompt.value = current.prompt;
    safe.checked = current.safe_mode;
    render();
    setInterval(() => reload().catch(notice), 1500);
  } catch (e) {
    notice(e);
  }
})();
