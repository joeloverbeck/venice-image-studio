type Original = {id: string; width: number; height: number; mime: string; bytes: number};

export function originalInspector(report: (error: unknown) => void) {
  const dialog = document.getElementById('original-inspector') as HTMLDialogElement;
  const viewport = document.getElementById('inspector-viewport')!;
  const image = document.getElementById('inspector-image') as HTMLImageElement;
  const zoom = document.getElementById('inspector-zoom')!;
  const fullscreen = document.getElementById('inspector-fullscreen')!;
  const surface = document.getElementById('inspector-surface')!;
  let original: Original | null = null;
  let scale = 1, x = 0, y = 0, fitting = true;
  let drag: {pointer: number; x: number; y: number} | null = null;
  function draw() {
    if (!original || !dialog.open) return;
    if (fitting) { scale = Math.min(1, viewport.clientWidth / original.width, viewport.clientHeight / original.height); x = y = 0; }
    const limitX = Math.max(0, (original.width * scale - viewport.clientWidth) / 2);
    const limitY = Math.max(0, (original.height * scale - viewport.clientHeight) / 2);
    x = Math.max(-limitX, Math.min(limitX, x)); y = Math.max(-limitY, Math.min(limitY, y));
    image.style.width = `${original.width * scale}px`; image.style.height = `${original.height * scale}px`;
    image.style.transform = `translate(calc(-50% + ${x}px), calc(-50% + ${y}px))`;
    zoom.textContent = `${Math.round(scale * 100)}%${fitting ? ' · Fit' : ''}`;
  }
  function setScale(next: number) { fitting = false; scale = Math.max(.01, Math.min(16, next)); draw(); }
  function fit() { fitting = true; draw(); }
  async function toggleFullscreen() {
    try { if (document.fullscreenElement) await document.exitFullscreen(); else await surface.requestFullscreen(); }
    catch { report('Fullscreen is unavailable in this browser. The inspector remains usable in this window.'); }
  }
  document.getElementById('inspector-fit')!.onclick = fit;
  document.getElementById('inspector-actual')!.onclick = () => { x = y = 0; setScale(1); };
  document.getElementById('inspector-in')!.onclick = () => setScale(scale * 1.25);
  document.getElementById('inspector-out')!.onclick = () => setScale(scale / 1.25);
  fullscreen.onclick = toggleFullscreen;
  document.addEventListener('fullscreenchange', () => { fullscreen.textContent = document.fullscreenElement ? 'Exit fullscreen' : 'Fullscreen'; draw(); });
  document.getElementById('close-inspector')!.onclick = () => dialog.close();
  dialog.addEventListener('close', () => { if (document.fullscreenElement === surface) void document.exitFullscreen(); image.removeAttribute('src'); original = null; });
  image.onerror = () => report('Original unavailable or integrity check failed. Its saved recipe remains available.');
  viewport.addEventListener('wheel', event => {
    event.preventDefault(); setScale(scale * Math.exp(-event.deltaY * .002));
  }, {passive: false});
  viewport.addEventListener('pointerdown', event => {
    if (event.button !== 0) return;
    event.preventDefault(); viewport.focus(); viewport.setPointerCapture(event.pointerId);
    drag = {pointer: event.pointerId, x: event.clientX, y: event.clientY};
  });
  viewport.addEventListener('pointermove', event => {
    if (!drag || drag.pointer !== event.pointerId) return;
    fitting = false; x += event.clientX - drag.x; y += event.clientY - drag.y;
    drag.x = event.clientX; drag.y = event.clientY; draw();
  });
  viewport.addEventListener('lostpointercapture', () => { drag = null; });
  viewport.addEventListener('pointerup', event => { if (viewport.hasPointerCapture(event.pointerId)) viewport.releasePointerCapture(event.pointerId); drag = null; });
  viewport.addEventListener('keydown', event => {
    if (event.key === '+' || event.key === '=') setScale(scale * 1.25);
    else if (event.key === '-') setScale(scale / 1.25);
    else if (event.key === '0') fit();
    else if (event.key === '1') setScale(1);
    else if (event.key === 'ArrowLeft') { fitting = false; x += 80; draw(); }
    else if (event.key === 'ArrowRight') { fitting = false; x -= 80; draw(); }
    else if (event.key === 'ArrowUp') { fitting = false; y += 80; draw(); }
    else if (event.key === 'ArrowDown') { fitting = false; y -= 80; draw(); }
    else return;
    event.preventDefault();
  });
  new ResizeObserver(draw).observe(viewport);
  return (result: Original, prompt: string, request: unknown) => {
    original = result; fitting = true; x = y = 0;
    image.alt = 'Saved original'; image.src = `/api/images/${result.id}`;
    document.getElementById('inspector-metadata')!.textContent = `${result.width} × ${result.height} actual pixels · ${result.mime} · ${result.bytes.toLocaleString()} original bytes`;
    document.getElementById('inspector-recipe')!.textContent = `${prompt}\n\nSubmitted settings\n${JSON.stringify(request, null, 2)}`;
    const download = document.getElementById('inspector-download') as HTMLAnchorElement;
    download.href = image.src; download.download = `${result.id}.${result.mime.split('/')[1]}`;
    dialog.showModal(); draw(); viewport.focus();
  };
}
