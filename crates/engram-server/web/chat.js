'use strict';

// ── Config ───────────────────────────────────────────────────────────────────
const ENGRAM_PORT  = 3456;
const NEMO_PORT    = 11500;
const ENGRAM_BASE  = `http://127.0.0.1:${ENGRAM_PORT}`;
const NEMO_BASE    = `http://127.0.0.1:${NEMO_PORT}`;

// ── State ────────────────────────────────────────────────────────────────────
let activeModel   = 'system1';
let chatHistory   = [];
let pendingImage  = null;   // base64 for vision models
let tasks         = JSON.parse(localStorage.getItem('eg_tasks') || '[]');

// Polling state
let _lastThoughtIdx = 0;
let _mintStates     = {};  // concept → last observed status
let _responseId     = 0;

// ── Init ─────────────────────────────────────────────────────────────────────
document.addEventListener('DOMContentLoaded', () => {
  renderTasks();
  pollStatus();
  pollRecent();
  pollMints();
  setInterval(pollStatus, 8000);
  setInterval(pollRecent, 6000);
  setInterval(pollMints,  10000);
});

// ── Model selector ────────────────────────────────────────────────────────────
function selectModel(m) {
  activeModel = m;
  document.getElementById('btn-sys1').classList.toggle('active', m === 'system1');
  document.getElementById('btn-sys2').classList.toggle('active', m === 'system2');
}

// ── Status polling ────────────────────────────────────────────────────────────
async function pollStatus() {
  // Engram memory status
  try {
    const r = await fetch(`${ENGRAM_BASE}/api/list`, { cache: 'no-store' });
    const list = await r.json();
    setStatus('memory', true, `${list.length} concepts`);
    document.getElementById('concept-count').textContent = `${list.length} concepts`;
  } catch {
    setStatus('memory', false, 'OFFLINE');
  }

  // nemo_agency status
  try {
    const r = await fetch(`${NEMO_BASE}/nemo/status`, { cache: 'no-store' });
    if (!r.ok) throw 0;
    const d = await r.json();
    setStatus('agent', d.manifold_ready || d.system1_ready, d.manifold_ready ? 'READY' : 'DEGRADED');
    const crsEl = document.getElementById('pill-crs');
    if (crsEl) { crsEl.textContent = `≥ ${(d.crs_threshold || 0.74).toFixed(2)}`; }
    document.getElementById('live-badge').style.display = 'flex';
  } catch {
    setStatus('agent', false, 'OFFLINE');
    document.getElementById('live-badge').style.display = 'none';
  }
}

function setStatus(id, online, label) {
  const dot  = document.getElementById(`dot-${id}`);
  const pill = document.getElementById(`pill-${id}`);
  if (dot)  { dot.className  = 'sdot' + (online ? ' live' : ''); }
  if (pill) { pill.className = 'spill' + (online ? ' ok' : ' off'); pill.textContent = label; }
}

// ── Recent memory polling ─────────────────────────────────────────────────────
async function pollRecent() {
  try {
    const r = await fetch(`${ENGRAM_BASE}/api/recent?n=20`, { cache: 'no-store' });
    if (!r.ok) return;
    const entries = await r.json();
    renderRecent(entries);
  } catch {}
}

function renderRecent(entries) {
  const list = document.getElementById('recent-list');
  if (!list) return;
  const ts = document.getElementById('recent-ts');
  if (ts) ts.textContent = 'updated just now';
  list.innerHTML = '';
  entries.forEach(e => {
    const { stalk, name } = parseConcept(e.concept);
    const div = document.createElement('div');
    div.className = 'recent-item';
    div.title = e.concept;
    div.onclick = () => recallConcept(e.concept);
    div.innerHTML = `
      <span class="recent-concept">${name}</span>
      <span class="recent-stalk ${stalkClass(stalk)}">${stalk || '—'}</span>
      <span class="recent-ago">${e.ago || ''}</span>`;
    list.appendChild(div);
  });
}

// Click a recent concept → auto-recall it into context panel
async function recallConcept(concept) {
  try {
    const name = concept.split('::').pop();
    const r = await fetch(`${ENGRAM_BASE}/api/recall`, {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ query: name, k: 5 })
    });
    const results = await r.json();
    renderContext(results, name);
  } catch {}
}

// ── JIT Mint polling ──────────────────────────────────────────────────────────
async function pollMints() {
  try {
    const r = await fetch(`${NEMO_BASE}/nemo/pending_mints`, { cache: 'no-store' });
    if (!r.ok) return;
    const mints = await r.json();
    if (!Array.isArray(mints) || mints.length === 0) {
      document.getElementById('mint-section').style.display = 'none';
      return;
    }
    document.getElementById('mint-section').style.display = 'block';
    document.getElementById('mint-count').textContent = `(${mints.length})`;
    const list = document.getElementById('mint-list');
    list.innerHTML = '';

    mints.forEach(m => {
      const prev = _mintStates[m.concept];
      if (prev === 'pending' && m.status === 'minted') {
        // New mint complete — toast it
        showToast('🪙 Minted!', `"${m.concept}" is now in the VSA manifold (CRS ${m.crs ? m.crs.toFixed(3) : '—'})`);
        pollRecent(); // refresh recent list
      }
      _mintStates[m.concept] = m.status;

      const div = document.createElement('div');
      div.className = 'mint-item';
      div.innerHTML = `
        <span class="mint-concept" title="${m.concept}">${m.concept}</span>
        <span class="mint-status ${m.status}">${m.status === 'minted' ? `✓ ${m.crs ? m.crs.toFixed(3) : ''}` : '⏳'}</span>`;
      list.appendChild(div);
    });
  } catch {}
}

// ── Thought stream polling (during active reply) ──────────────────────────────
async function pollThoughts(msgId) {
  try {
    const r = await fetch(`${NEMO_BASE}/nemo/thoughts`, { cache: 'no-store' });
    if (!r.ok) return;
    const raw = await r.json();
    const data = Array.isArray(raw) ? { thoughts: raw, final_response: null, response_id: 0 } : raw;
    const thoughts = data.thoughts || [];
    if (thoughts.length > _lastThoughtIdx) {
      const strip    = document.getElementById('thought-strip');
      const textEl   = document.getElementById('thought-text');
      const newLines = thoughts.slice(_lastThoughtIdx);
      strip.style.display = 'flex';
      textEl.textContent = newLines[newLines.length - 1];
      _lastThoughtIdx = thoughts.length;
    }
    return data;
  } catch {}
}

// ── Send message ──────────────────────────────────────────────────────────────
async function send() {
  const input = document.getElementById('chat-input');
  const text  = input.value.trim();
  if (!text && !pendingImage) return;

  input.value = '';
  input.style.height = '';
  autoResize(input);

  const userContent = text || (pendingImage ? '[image]' : '');
  appendMsg('user', userContent, new Date().toLocaleTimeString());
  chatHistory.push({ role: 'user', content: text });

  clearImage();
  _lastThoughtIdx = 0;
  document.getElementById('thought-strip').style.display = 'none';

  const btnSend = document.getElementById('btn-send');
  btnSend.disabled = true;
  const agoTs = new Date().toLocaleTimeString();

  // Recall context from Engram first (parallel with LLM start)
  recallForQuery(text);

  // Start SSE stream from nemo_agency
  const msgDiv = appendMsg('assistant', '', agoTs, true);
  const bubble = msgDiv.querySelector('.bubble-inner');

  try {
    const body = {
      model:    activeModel,
      messages: chatHistory,
      stream:   true,
    };

    const resp = await fetch(`${NEMO_BASE}/v1/chat/completions`, {
      method:  'POST',
      headers: { 'Content-Type': 'application/json' },
      body:    JSON.stringify(body),
    });

    if (!resp.ok) throw new Error(`Agent returned ${resp.status}`);

    const reader = resp.body.getReader();
    const dec    = new TextDecoder();
    let full     = '';
    let thoughtTimer = setInterval(() => pollThoughts(), 1200);

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      const chunk = dec.decode(value, { stream: true });
      for (const line of chunk.split('\n')) {
        if (!line.startsWith('data: ')) continue;
        const data = line.slice(6).trim();
        if (data === '[DONE]') break;
        try {
          const parsed = JSON.parse(data);
          const delta  = parsed.choices?.[0]?.delta?.content || '';
          full += delta;
          bubble.textContent = full;
          bubble.scrollIntoView({ block: 'end', behavior: 'smooth' });
        } catch {}
      }
    }

    clearInterval(thoughtTimer);
    document.getElementById('thought-strip').style.display = 'none';
    chatHistory.push({ role: 'assistant', content: full });

    // Add save-to-memory button
    const meta = msgDiv.querySelector('.bubble-meta');
    if (meta) {
      const saveBtn = document.createElement('button');
      saveBtn.className = 'bubble-save';
      saveBtn.textContent = '+ Save';
      saveBtn.title = 'Save this response to Engram memory';
      saveBtn.onclick = () => saveResponse(full, text, saveBtn);
      meta.appendChild(saveBtn);
    }

  } catch (err) {
    bubble.textContent = `[Error: ${err.message}]`;
  }

  btnSend.disabled = false;
}

// ── Context panel: auto-recall after message ──────────────────────────────────
async function recallForQuery(query) {
  if (!query) return;
  try {
    const r = await fetch(`${ENGRAM_BASE}/api/recall`, {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ query, k: 6 })
    });
    const results = await r.json();
    renderContext(results, query);
  } catch {}
}

function renderContext(results, query) {
  const list = document.getElementById('context-list');
  const qLabel = document.getElementById('ctx-query-label');
  if (!list) return;
  if (qLabel) qLabel.textContent = `"${query}"`;
  list.innerHTML = '';
  if (!results || results.length === 0) {
    list.innerHTML = '<div class="ctx-empty">No relevant memories found.</div>';
    return;
  }
  results.forEach(m => {
    const { stalk, name } = parseConcept(m.concept);
    const crs  = m.crs || 0;
    const crsColor = crs >= 0.74 ? '#22c55e' : crs >= 0.5 ? '#fbbf24' : '#f87171';
    const div  = document.createElement('div');
    div.className = 'ctx-card';
    div.innerHTML = `
      <div class="ctx-top">
        <span class="ctx-concept" title="${m.concept}">${name}</span>
        <span class="ctx-score">${m.score ? m.score.toFixed(4) : '—'}</span>
      </div>
      <div class="ctx-provrow">
        <span class="prov-badge ${stalkClass(stalk)}">${stalk || 'user'}</span>
        <span style="font-size:.52rem;color:var(--muted);font-family:'JetBrains Mono',monospace">CRS ${crs.toFixed(2)}</span>
      </div>
      <div class="ctx-text">${(m.text || '').slice(0, 180)}</div>
      <div class="crs-bar-wrap"><div class="crs-bar" style="width:${Math.round(crs*100)}%;background:${crsColor}"></div></div>`;
    list.appendChild(div);
  });
}

// ── Save response to Engram memory ────────────────────────────────────────────
async function saveResponse(responseText, query, btn) {
  const concept = `session_${Date.now()}_${query.slice(0, 24).replace(/\s+/g,'_').replace(/[^a-z0-9_]/gi,'')}`;
  try {
    btn.textContent = '…';
    btn.disabled = true;
    await fetch(`${ENGRAM_BASE}/api/remember`, {
      method:  'POST',
      headers: { 'Content-Type': 'application/json' },
      body:    JSON.stringify({ concept, text: `Q: ${query}\n\nA: ${responseText}` }),
    });
    showToast('✅ Saved!', `Stored as "${concept}"`);
    btn.textContent = '✓ Saved';
    pollRecent();
  } catch {
    btn.textContent = '✗ Error';
    btn.disabled = false;
  }
}

// ── File ingestion ────────────────────────────────────────────────────────────
function handleDrop(e) {
  e.preventDefault();
  document.getElementById('drop-zone').classList.remove('drag-over');
  const files = Array.from(e.dataTransfer.files);
  ingestFiles(files);
}

function handleFileSelect(e) {
  ingestFiles(Array.from(e.target.files));
  e.target.value = '';
}

async function ingestFiles(files) {
  const log = document.getElementById('ingest-log');
  for (const file of files) {
    const line = document.createElement('div');
    line.className = 'ingest-line';
    line.textContent = `⏳ ${file.name} (${(file.size/1024).toFixed(1)}KB)`;
    log.prepend(line);

    try {
      const text    = await file.text();
      const concept = `file::${file.name.replace(/[^a-z0-9_.]/gi,'_')}`;

      // Chunk into ~4000-char pieces if large
      const CHUNK = 4000;
      const chunks = [];
      for (let i = 0; i < text.length; i += CHUNK) chunks.push(text.slice(i, i+CHUNK));

      for (let ci = 0; ci < chunks.length; ci++) {
        const key = chunks.length > 1 ? `${concept}_part${ci+1}` : concept;
        await fetch(`${ENGRAM_BASE}/api/remember`, {
          method:  'POST',
          headers: { 'Content-Type': 'application/json' },
          body:    JSON.stringify({ concept: key, text: chunks[ci] }),
        });
      }

      line.className = 'ingest-line done';
      line.textContent = `✓ ${file.name} → ${chunks.length} block${chunks.length > 1 ? 's' : ''}`;
      showToast('📂 Ingested', `${file.name} stored in ${chunks.length} block(s)`);
      pollRecent();
    } catch (err) {
      line.className = 'ingest-line err';
      line.textContent = `✗ ${file.name}: ${err.message}`;
    }
  }
}

// ── Image attach ──────────────────────────────────────────────────────────────
function handleMediaUpload(e) {
  const file = e.target.files[0];
  if (!file) return;
  const reader = new FileReader();
  reader.onload = ev => {
    pendingImage = ev.target.result;
    const prev = document.getElementById('img-preview');
    document.getElementById('img-preview-img').src = pendingImage;
    prev.style.display = 'flex';
  };
  reader.readAsDataURL(file);
  e.target.value = '';
}

function clearImage() {
  pendingImage = null;
  document.getElementById('img-preview').style.display = 'none';
  document.getElementById('img-preview-img').src = '';
}

// ── VSA Trace ─────────────────────────────────────────────────────────────────
async function runTrace() {
  const a  = document.getElementById('trace-a').value.trim();
  const b  = document.getElementById('trace-b').value.trim();
  const op = document.getElementById('trace-op').value;
  if (!a || !b) return;
  try {
    const r = await fetch(`${ENGRAM_BASE}/api/trace`, {
      method:  'POST',
      headers: { 'Content-Type': 'application/json' },
      body:    JSON.stringify({ term_a: a, term_b: b, op, k: 5 }),
    });
    const res = await r.json();
    const box = document.getElementById('trace-results');
    box.innerHTML = '';
    res.forEach(m => {
      const { name } = parseConcept(m.concept);
      const div = document.createElement('div');
      div.className = 'trace-row';
      div.innerHTML = `<span class="tc" title="${m.concept}">${name}</span><span class="ts">${m.score ? m.score.toFixed(4) : '—'}</span>`;
      box.appendChild(div);
    });
  } catch {}
}

// ── Tasks & Goals ─────────────────────────────────────────────────────────────
function openAddTask() {
  document.getElementById('task-modal').style.display = 'flex';
  document.getElementById('task-title').focus();
}
function closeAddTask() {
  document.getElementById('task-modal').style.display = 'none';
  document.getElementById('task-title').value = '';
  document.getElementById('task-desc').value  = '';
}
function saveTask() {
  const title = document.getElementById('task-title').value.trim();
  if (!title) return;
  tasks.push({
    id:       Date.now(),
    title,
    desc:     document.getElementById('task-desc').value.trim(),
    priority: document.getElementById('task-priority').value,
    done:     false,
    created:  Date.now(),
  });
  persistTasks();
  renderTasks();
  closeAddTask();

  // Also remember the goal in Engram
  fetch(`${ENGRAM_BASE}/api/remember`, {
    method: 'POST', headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ concept: `goal_${Date.now()}`, text: `Goal: ${title}\n${document.getElementById('task-desc').value}` })
  }).catch(() => {});
}
function toggleTask(id) {
  const t = tasks.find(t => t.id === id);
  if (t) t.done = !t.done;
  persistTasks();
  renderTasks();
}
function deleteTask(id) {
  tasks = tasks.filter(t => t.id !== id);
  persistTasks();
  renderTasks();
}
function persistTasks() {
  localStorage.setItem('eg_tasks', JSON.stringify(tasks));
}
function renderTasks() {
  const list = document.getElementById('task-list');
  if (!list) return;
  if (tasks.length === 0) {
    list.innerHTML = '<div class="task-empty">No active tasks. Add a goal to track agent progress.</div>';
    return;
  }
  list.innerHTML = '';
  const sorted = [...tasks].sort((a,b) => {
    const po = { high:0, medium:1, low:2 };
    if (a.done !== b.done) return a.done ? 1 : -1;
    return (po[a.priority]||1) - (po[b.priority]||1);
  });
  sorted.forEach(t => {
    const div = document.createElement('div');
    div.className = 'task-item';
    div.innerHTML = `
      <div class="task-check${t.done?' done':''}" onclick="toggleTask(${t.id})" title="${t.done?'Mark incomplete':'Mark done'}"></div>
      <div class="task-body">
        <div class="task-title${t.done?' done':''}">${esc(t.title)}</div>
        ${t.desc ? `<div class="task-desc">${esc(t.desc)}</div>` : ''}
      </div>
      <div class="task-pri ${t.priority}" title="${t.priority} priority"></div>
      <button class="task-del" onclick="deleteTask(${t.id})" title="Delete">✕</button>`;
    list.appendChild(div);
  });
}

// ── Chat helpers ──────────────────────────────────────────────────────────────
function appendMsg(role, text, time, isStreaming = false) {
  const msgs = document.getElementById('chat-messages');
  const div  = document.createElement('div');
  div.className = `msg ${role}`;
  div.innerHTML = `
    <div class="bubble">
      <div class="bubble-inner${isStreaming?' typing-dot':''}">${esc(text)}</div>
      <div class="bubble-meta">
        <span>${role === 'user' ? 'You' : 'Engram Agent'}</span>
        <span>${time}</span>
      </div>
    </div>`;
  if (isStreaming) div.querySelector('.bubble-inner').textContent = '';
  msgs.appendChild(div);
  msgs.scrollTop = msgs.scrollHeight;
  return div;
}

function autoResize(el) {
  el.style.height = 'auto';
  el.style.height = Math.min(el.scrollHeight, 160) + 'px';
}

// ── Utilities ─────────────────────────────────────────────────────────────────
function parseConcept(concept) {
  const parts = concept.split('::');
  if (parts.length >= 2) return { stalk: parts[0], name: parts.slice(1).join('::') };
  return { stalk: null, name: concept };
}

function stalkClass(stalk) {
  if (!stalk) return 'default';
  if (stalk.includes('codeland') || stalk.includes('monad')) return 'codeland';
  if (stalk.includes('engram'))  return 'engram';
  if (stalk === 'user')          return 'user';
  return 'default';
}

function esc(s) {
  return (s || '').replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;');
}

let _toastTimer = null;
function showToast(head, body) {
  const el = document.getElementById('toast');
  document.getElementById('toast-head').textContent = head;
  document.getElementById('toast-body').textContent = body;
  el.style.display = 'block';
  if (_toastTimer) clearTimeout(_toastTimer);
  _toastTimer = setTimeout(() => el.style.display = 'none', 4000);
}
