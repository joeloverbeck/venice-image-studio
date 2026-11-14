export { };
type Draft = {
  id: string;
  prompt: string;
  safe_mode: boolean;
  model: string;
  parent_id: string | null;
  settings: Record<string, any>;
};
type Attempt = {
  id: string;
  created_at: string;
  dispatched_at: string | null;
  status: string;
  summary: string;
  prompt: string;
  request: Record<string, any>;
  returned_seed: number | null;
  can_retry_local_save: boolean;
  results: Array<{
    id: string;
    width: number;
    height: number;
    mime: string;
    bytes: number;
    sha256: string;
    available: boolean;
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
  renderSettings();
  generate.disabled = true;
  el("draft-status").textContent = "Saving draft…";
  saving = saving
    .catch(() => { })
    .then(async () => {
      const snapshot = structuredClone(current);
      try {
        await api("/api/draft", snapshot, "PUT");
        if (
          JSON.stringify(snapshot) === JSON.stringify(current)
        ) {
          state = await api("/api/state");
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
  const model = chosen();
  const estimate = sameDraft() ? state.estimate : { unavailable_reason: "Waiting for validation of your current choices." };
  const settings = current.settings;
  el("model").textContent = `${model?.name || current.model} (${current.model}) · ${settings.sizing === "pixels" ? `${settings.width} × ${settings.height} pixels` : `${settings.aspect_ratio || "ratio unselected"} · ${settings.resolution || "model-native resolution"}`} · ${settings.quality || "model-native quality"} · ${settings.variants} image${settings.variants === 1 ? "" : "s"} · ${settings.format?.toUpperCase() || "format unselected"} · safe mode ${current.safe_mode ? "ON" : "off"}`;
  generate.textContent = settings.variants === 1 ? "Generate one image" : `Generate ${settings.variants} images`;
  el("model-warning").textContent = `${model?.availability || "unknown"}. ${model?.guidance?.fit || "No reviewed guidance available."}`;
  renderSettings();
  const conflicts = el("preflight"); conflicts.replaceChildren();
  if (sameDraft()) for (const message of state.preflight.errors) conflicts.append(text("p", message));
  el("estimate").textContent =
    estimate.usd === undefined
      ? `Estimate unavailable. ${estimate.base_usd === undefined ? "" : `Base $${estimate.base_usd} USD; selected extras remain uncertain. `}${estimate.unavailable_reason}`
      : `Estimate: $${estimate.usd} USD · Venice catalog fetched ${estimate.fetched_at}. ${(estimate.extras || []).map((e: any) => `${e.name}: ${e.pricing}`).join(" ")} Estimate is not a charge or spending cap.`;
  el("library-path").textContent = `Library: ${state.library || "unavailable"}`;
  counter();
  generate.disabled =
    Boolean(state.inflight) ||
    dirty ||
    !sameDraft() ||
    Boolean(state.preflight.errors.length) ||
    estimate.usd === undefined ||
    !state.credential_present ||
    Boolean(state.setup_error) ||
    state.attempts.some((a: Attempt) => a.id === current.id);
  const errors = [
    state.setup_error,
    state.catalog_error,
    state.styles_error,
    !state.credential_present
      ? "Set VENICE_API_KEY in the private .env or environment, then restart Studio."
      : null,
  ].filter(Boolean);
  if (errors.length) notice(errors.join(" "));
  const list = el("attempts");
  const buttons = new Map(
    Array.from(list.children, (node) => {
      const button = node as HTMLButtonElement;
      return [button.dataset.attempt, button];
    }),
  );
  state.attempts.sort((a: Attempt, b: Attempt) =>
    b.created_at.localeCompare(a.created_at),
  );
  (state.attempts as Attempt[]).forEach((a, index) => {
    const button = buttons.get(a.id) || document.createElement("button");
    buttons.delete(a.id);
    button.dataset.attempt = a.id;
    button.className = "attempt";
    const label = `${new Date(a.created_at).toLocaleString()} · ${a.status} · ${a.prompt.slice(0, 70)}`;
    if (button.textContent !== label) button.textContent = label;
    button.onclick = () => {
      selected = a.id;
      show(a);
    };
    if (list.children[index] !== button)
      list.insertBefore(button, list.children[index] || null);
  });
  for (const button of buttons.values()) button.remove();
  const shown =
    state.attempts.find((a: Attempt) => a.id === selected) || state.attempts[0];
  if (shown) show(shown);
}
function show(a: Attempt) {
  selected = a.id;
  el("requested").textContent = `Requested: ${a.request.model} · ${a.request.width ? `${a.request.width} × ${a.request.height} pixels` : `${a.request.aspect_ratio} · ${a.request.resolution || "model-native resolution"}`} · ${a.request.quality || "model-native quality"} · ${a.request.format} · ${a.request.variants} images. Actual result dimensions appear below.`;
  el("requested").textContent += ` Seed requested: ${a.request.seed === undefined ? "omitted/random (schema default is uncertain)" : a.request.seed}; provider-returned seed: ${a.returned_seed ?? "unknown"}.`;
  el("result-status").textContent = `${a.status}: ${a.summary}`;
  if (a.status === "waiting" && a.dispatched_at) {
    const seconds = Math.max(0, Math.floor((Date.now() - Date.parse(a.dispatched_at)) / 1000));
    el("result-status").textContent += ` Elapsed ${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, "0")}. No completion time is known.`;
  }
  el("recipe").textContent = JSON.stringify(a, null, 2);
  const images = el("images");
  if (
    images.dataset.attempt !== a.id ||
    images.childElementCount !== a.results.length
  ) {
    images.dataset.attempt = a.id;
    images.replaceChildren();
    for (const r of a.results) {
      if (r.available === false) {
        const missing = document.createElement("figure");
        missing.append(text("p", `Original unavailable after interruption (${r.width} × ${r.height}). Missing or damaged bytes cannot be recovered by generating again.`));
        images.append(missing);
        continue;
      }
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
  let reuse = actions.querySelector<HTMLButtonElement>(".reuse");
  if (!reuse) {
    reuse = document.createElement("button");
    reuse.className = "secondary reuse";
    reuse.textContent = "Use recipe as new draft";
    actions.append(reuse);
  }
  reuse.onclick = async () => {
    try {
      await saving;
      current = await api(`/api/attempts/${a.id}/reuse`, {});
      prompt.value = current.prompt;
      safe.checked = current.safe_mode;
      state = await api("/api/state");
      controlsKey = "";
      dirty = false;
      el("draft-status").textContent =
        "New draft saved. Nothing has been generated.";
      render();
      prompt.focus();
    } catch (e) {
      notice(e);
    }
  };
  let spending = actions.querySelector<HTMLParagraphElement>(".spending-note");
  if (!spending) {
    spending = document.createElement("p"); spending.className = "spending-note"; actions.append(spending);
  }
  spending.textContent = "Using a recipe makes a new draft only. Generate submits a new paid attempt; a previous failed or interrupted attempt may already have been charged. Provider suggestions and retry advice never change or resubmit your draft.";
  let save = actions.querySelector<HTMLButtonElement>(".retry-save");
  if (a.status === "save_failed" && a.can_retry_local_save !== false) {
    if (!save) {
      save = document.createElement("button");
      save.className = "retry-save";
      save.textContent = "Retry local save";
      actions.append(save);
    }
    save.onclick = async () => {
      try {
        await api(`/api/attempts/${a.id}/save`, {});
        await reload();
      } catch (e) {
        notice(e);
      }
    };
  } else save?.remove();
}
async function reload() {
  state = await api("/api/state");
  render();
}
generate.onclick = async () => {
  generate.disabled = true;
  try {
    await saving;
    const a = await api("/api/generate", { draft_id: current.id, catalog_fetched_at: state.catalog.fetched_at, draft_revision: state.preflight.draft_revision });
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
let inspecting: string | null = null;
let controlsKey = "";
const browser = el<HTMLDialogElement>("model-browser");
function cacheAge(seconds: number | null) { return seconds == null ? "unknown" : seconds < 60 ? `${seconds} seconds` : seconds < 3600 ? `${Math.floor(seconds / 60)} minutes` : `${Math.floor(seconds / 3600)} hours`; }
function models(): any[] { return state.models || []; }
function chosen(): any { return models().find(m => m.id === current.model); }
function sameDraft() { return JSON.stringify(state.draft) === JSON.stringify(current); }
function text(tag: string, value: string, className = "") {
  const node = document.createElement(tag); node.textContent = value; node.className = className; return node;
}
function sourceLink(title: string, value: unknown) {
  const a = document.createElement("a"); a.textContent = title;
  try { const url = new URL(String(value)); if (url.protocol === "https:") { a.href = url.href; a.target = "_blank"; a.rel = "noreferrer"; } } catch { }
  return a;
}
const claimNames: Record<string, string> = { uncensored: "Uncensored positioning", privacy: "Venice privacy", upstream_retention: "Upstream retention", additional_protection: "Additional protection" };
const claimLabels: Record<string, string> = { asserted: "Venice assertion", not_asserted: "Explicit negative flag", unknown: "Unknown", private: "Private claim", anonymized: "Anonymized claim", zero_retention_claim: "Venice zero-retention assertion", conflicting: "Conflicting dated claims" };
function renderBrowser() {
  const search = el<HTMLInputElement>("model-search").value.toLocaleLowerCase();
  const matching = models().filter(m => {
    if (![m.name, m.id, m.guidance.fit, m.guidance.advice].join(" ").toLocaleLowerCase().includes(search)) return false;
    return Object.keys(claimNames).every(key => { const filter = el<HTMLSelectElement>(`filter-${key}`).value; return !filter || m.claims[key].state === filter; });
  });
  el("model-results-count").textContent = `${matching.length} of ${models().length} current and historical models`;
  el("catalog-age").textContent = `Catalog fetched ${state.catalog?.fetched_at || "never"}; age ${cacheAge(state.catalog_age_seconds)}. ${state.catalog_error || ""}`;
  const list = el("model-list"); list.replaceChildren();
  for (const m of matching) {
    const row = document.createElement("button"); row.className = "model-row secondary";
    row.append(text("strong", m.name), text("span", m.id), text("span", `${m.availability} · ${m.prompt_limit ?? "unknown"} characters · ${m.contract.sizing} · ${m.price_summary || "price in detail"}`), text("span", `${claimLabels[m.claims.privacy.state]} · uncensored: ${claimLabels[m.claims.uncensored.state]}`));
    row.onclick = () => { inspecting = m.id; renderModelDetail(); }; list.append(row);
  }
  if (!inspecting) inspecting = current.model;
  renderModelDetail();
}
function renderModelDetail() {
  const m = models().find(m => m.id === inspecting); const detail = el("model-detail"); detail.replaceChildren();
  if (!m) { detail.append(text("p", "Choose a model to inspect its guidance.")); return; }
  detail.append(text("h3", m.name), text("p", m.id), text("p", `${m.availability} · advertised prompt limit ${m.prompt_limit ?? "unknown"} · ${m.contract.sizing} sizing`));
  const use = document.createElement("button"); use.textContent = m.id === current.model ? "Selected model" : "Use this model"; use.disabled = m.id === current.model;
  use.onclick = () => { current.model = m.id; controlsKey = ""; edited(); browser.close(); }; detail.append(use);
  detail.append(text("h4", "Practical fit and tradeoffs"), text("p", m.guidance.fit), text("p", `Guidance reviewed ${m.guidance.reviewed_at || "not yet"}. Practical suggestions are editorial, not measured comparative results.`), text("h4", "Prompting advice"), text("p", m.guidance.advice), text("p", m.guidance.advice_scope || "No exact-version guidance established."));
  if (m.vendor_description) detail.append(text("h4", "Current vendor description"), text("p", String(m.vendor_description)));
  if (m.vendor_traits?.length) detail.append(text("p", `Venice labels (not app observations): ${m.vendor_traits.join(", ")}`));
  detail.append(text("h4", "Separate claims"));
  if (m.guidance.dated_privacy && m.guidance.dated_privacy !== m.claims.privacy.raw) detail.append(text("p", `Dated guidance recorded privacy “${m.guidance.dated_privacy}”; current metadata is “${String(m.claims.privacy.raw)}”. The current value remains separate.`));
  if (m.guidance.dated_uncensored === true && m.claims.uncensored.state !== "asserted") detail.append(text("p", "The dated guide recorded a positive uncensored flag. Current metadata does not repeat it; this is not a refusal-rate conclusion."));
  for (const key of Object.keys(claimNames)) {
    const claim = m.claims[key]; detail.append(text("p", `${claimNames[key]}: ${claimLabels[claim.state] || claim.state}. ${claim.meaning}`), sourceLink("Claim source", claim.source), text("small", ` ${claim.fetched_at || claim.reviewed_at || "date unknown"}; raw value: ${JSON.stringify(claim.raw ?? null)}`));
    if (claim.policy_source) detail.append(sourceLink("Meaning of Venice privacy claims", claim.policy_source));
  }
  detail.append(text("h4", "Sizing, controls and prices"));
  const c = m.contract;
  detail.append(text("p", c.sizing === "pixels" ? `Pixel sizing: positive width and height up to ${c.pixel_max}, in multiples of ${c.divisor ?? "unknown"}. Requested and actual dimensions may differ.` : c.sizing === "ratio" ? `Aspect ratios: ${c.aspect_ratios.join(", ")}. ${c.resolutions.length ? `Resolution tiers: ${c.resolutions.join(", ")}. Tiers do not guarantee exact pixel dimensions.` : "Model-native resolution; no selectable tier is advertised."}` : "A required sizing contract is missing; generation is unavailable."));
  detail.append(text("p", `Formats: ${c.formats.join(", ")}. ${c.qualities.length ? `Qualities: ${c.qualities.join(", ")}.` : "No selectable quality is advertised."} Transparency: ${c.transparent_background ? "advertised for PNG/WebP" : "not established"}.`));
  detail.append(text("p", `Per-image base estimates from catalog fetched ${m.price_fetched_at || "unknown"}. Selected extras are shown in preflight. Actual charges and refunds remain unknown.`));
  const table = document.createElement("table"); const header = document.createElement("tr"); header.append(text("th", "Choice"), text("th", "Estimate USD")); table.append(header);
  for (const row of m.price_rows || []) { const tr = document.createElement("tr"); tr.append(text("td", row.choice), text("td", row.usd == null ? "Unknown" : `$${row.usd}`)); table.append(tr); } detail.append(table);
  detail.append(sourceLink("Current catalog source", "https://api.venice.ai/api/v1/models?type=image"));
  const diagnostics = document.createElement("details"); diagnostics.append(text("summary", "Technical metadata and control evidence"), text("pre", JSON.stringify({ constraints: m.metadata.model_spec?.constraints, contract: m.contract }, null, 2))); detail.append(diagnostics);
  detail.append(text("h4", "Guidance sources"));
  for (const source of m.guidance.sources || []) { const p = document.createElement("p"); p.append(sourceLink(source.title, source.url)); detail.append(p); }
}
function renderSettings(force = false) {
  const m = chosen(); const c = m?.contract; const key = JSON.stringify([current.model, c, state.styles?.data]);
  const stylesStatus = `Styles fetched ${state.styles?.fetched_at || "never"}; age ${cacheAge(state.styles_age_seconds)}. ${state.styles_error || ""}`;
  // Age and refresh status change without changing controls; retain their focus.
  const status = document.getElementById("styles-status");
  if (status) status.textContent = stylesStatus;
  if (!force && controlsKey === key) return;
  controlsKey = key;
  const container = el("settings"); const advancedOpen = container.querySelector("details")?.open || false; container.replaceChildren();
  if (!c) return;
  const settings = current.settings;
  const basic = document.createElement("div"); basic.className = "settings-grid";
  const advanced = document.createElement("details"); advanced.open = advancedOpen; advanced.append(text("summary", "Advanced controls"));
  function control(key: string, title: string, choices?: string[], options: { advanced?: boolean, type?: string, supported?: boolean, min?: number, max?: number } = {}) {
    const supported = options.supported !== false;
    const label = document.createElement("label"); label.textContent = title;
    const input = choices ? document.createElement("select") : document.createElement("input"); input.id = `setting-${key}`; input.setAttribute("aria-label", title);
    const value = settings[key];
    if (input instanceof HTMLSelectElement) {
      input.append(new Option("Unselected", ""));
      for (const choice of choices || []) input.append(new Option(choice, choice));
      if (value != null && !choices?.includes(String(value))) input.append(new Option(`${value} (incompatible)`, String(value)));
      input.value = String(value ?? "");
    } else {
      input.type = options.type || "number"; input.value = String(value ?? "");
      if (input.type === "checkbox") input.checked = Boolean(value);
      if (options.min !== undefined) input.min = String(options.min); if (options.max !== undefined) input.max = String(options.max);
      if (key === "cfg_scale") input.step = "any";
    }
    input.disabled = !supported;
    input.onchange = () => {
      settings[key] = input instanceof HTMLInputElement && input.type === "checkbox" ? input.checked : input.value === "" ? null : input instanceof HTMLInputElement && input.type === "number" ? Number(input.value) : input.value;
      edited();
    };
    label.append(input);
    if (!supported) {
      label.append(text("small", `Unavailable: ${c[key] || "support not established"}. ${c.control_notes?.[key] || ""}`));
      if (value != null && value !== false) { const clear = document.createElement("button"); clear.type = "button"; clear.textContent = "Clear incompatible choice"; clear.className = "secondary"; clear.onclick = () => { settings[key] = typeof value === "boolean" ? false : null; edited(); renderSettings(true) }; label.append(clear); }
    }
    (options.advanced ? advanced : basic).append(label);
  }
  if (c.sizing === "pixels") {
    control("width", "Width in pixels", undefined, { min: 1, max: c.pixel_max }); control("height", "Height in pixels", undefined, { min: 1, max: c.pixel_max });
    basic.append(text("p", `Use multiples of ${c.divisor || "unknown"}; maximum ${c.pixel_max} per side. Actual returned pixels may differ.`));
    const presets = document.createElement("div");
    for (const preset of c.pixel_presets || []) { const button = document.createElement("button"); button.className = "secondary"; button.textContent = preset.label; button.onclick = () => { settings.width = preset.width; settings.height = preset.height; edited(); renderSettings(true) }; presets.append(button) }
    basic.append(presets);
  } else if (c.sizing === "ratio") {
    control("aspect_ratio", "Aspect ratio", c.aspect_ratios);
  }
  if (c.resolutions.length || settings.resolution) control("resolution", "Resolution tier", c.resolutions);
  if (c.qualities.length || settings.quality) control("quality", "Quality", c.qualities);
  control("format", "Original format", c.formats);
  control("variants", "Images per attempt", undefined, { advanced: true, min: c.variants_min, max: c.variants_max });
  control("seed", "Explicit seed (blank = omit/random)", undefined, { advanced: true, supported: c.seed === "documented", min: -999999999, max: 999999999 });
  control("negative_prompt", "Negative prompt", undefined, { advanced: true, type: "text", supported: c.negative_prompt === "documented" });
  control("steps", "Inference steps", undefined, { advanced: true, supported: c.steps === "documented", min: 1, max: c.steps_max });
  control("cfg_scale", "CFG scale", undefined, { advanced: true, supported: c.cfg_scale === "documented", min: 0, max: 20 });
  control("style_preset", "Style preset", state.styles?.data || [], { advanced: true, supported: c.style_preset === "documented" });
  control("transparent_background", "Transparent background", undefined, { advanced: true, type: "checkbox", supported: c.transparent_background });
  control("web_search", "Web search", undefined, { advanced: true, type: "checkbox", supported: c.web_search });
  control("optimize_prompt", "Optional optimization / thinking", undefined, { advanced: true, type: "checkbox", supported: c.optimize_prompt });
  for (const limitation of c.limitations || []) advanced.append(text("p", limitation));
  const styleStatus = text("p", stylesStatus); styleStatus.id = "styles-status"; advanced.append(styleStatus);
  container.append(basic, advanced);
}
el("browse-models").onclick = () => { renderBrowser(); browser.showModal(); };
el("close-models").onclick = () => browser.close();
el("model-search").addEventListener("input", renderBrowser);
for (const key of Object.keys(claimNames)) el(`filter-${key}`).addEventListener("change", renderBrowser);
el("model-defaults").onclick = () => {
  const defaults = chosen()?.contract?.defaults; if (!defaults) return;
  current.settings = structuredClone(defaults); edited(); renderSettings(true);
};

(async () => {
  try {
    state = await api("/api/state");
    current = structuredClone(state.draft);
    prompt.value = current.prompt;
    safe.checked = current.safe_mode;
    render();
    prompt.disabled = false; safe.disabled = false;
    for (const id of ["browse-models", "model-defaults", "refresh"]) el<HTMLButtonElement>(id).disabled = false;
    el("draft-status").textContent = "Saved draft loaded.";
    setInterval(() => reload().catch(notice), 1500);
  } catch (e) {
    notice(e);
  }
})();
