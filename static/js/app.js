'use strict';

// ── Theme definitions ──────────────────────────────────────────────────────
const THEMES = {
  hacker: {
    background: '#0d0d0d', foreground: '#00ff41', cursor: '#00ff41',
    cursorAccent: '#000', selectionBackground: '#00ff4133',
    black: '#0d0d0d',    red: '#ff4444',     green: '#00ff41',    yellow: '#ffaa00',
    blue: '#007bff',     magenta: '#cc00ff',  cyan: '#00ffcc',     white: '#c8c8c8',
    brightBlack: '#444', brightRed: '#ff6666', brightGreen: '#33ff66', brightYellow: '#ffcc44',
    brightBlue: '#3399ff', brightMagenta: '#ee44ff', brightCyan: '#44ffee', brightWhite: '#ffffff',
  },
  dark: {
    background: '#1e1e1e', foreground: '#d4d4d4', cursor: '#d4d4d4',
    cursorAccent: '#1e1e1e', selectionBackground: '#264f78aa',
    black: '#1e1e1e',   red: '#f44747',    green: '#6a9955',   yellow: '#d7ba7d',
    blue: '#569cd6',    magenta: '#c586c0', cyan: '#4ec9b0',   white: '#d4d4d4',
    brightBlack: '#808080', brightRed: '#f44747', brightGreen: '#6a9955', brightYellow: '#d7ba7d',
    brightBlue: '#569cd6',  brightMagenta: '#c586c0', brightCyan: '#4ec9b0', brightWhite: '#d4d4d4',
  },
  dracula: {
    background: '#282a36', foreground: '#f8f8f2', cursor: '#f8f8f2',
    cursorAccent: '#282a36', selectionBackground: '#44475a99',
    black: '#21222c', red: '#ff5555', green: '#50fa7b', yellow: '#f1fa8c',
    blue: '#bd93f9', magenta: '#ff79c6', cyan: '#8be9fd', white: '#f8f8f2',
    brightBlack: '#6272a4', brightRed: '#ff6e6e', brightGreen: '#69ff94', brightYellow: '#ffffa5',
    brightBlue: '#d6acff', brightMagenta: '#ff92df', brightCyan: '#a4ffff', brightWhite: '#ffffff',
  },
  solarized: {
    background: '#002b36', foreground: '#839496', cursor: '#839496',
    cursorAccent: '#002b36', selectionBackground: '#073642aa',
    black: '#073642', red: '#dc322f', green: '#859900', yellow: '#b58900',
    blue: '#268bd2', magenta: '#d33682', cyan: '#2aa198', white: '#eee8d5',
    brightBlack: '#002b36', brightRed: '#cb4b16', brightGreen: '#586e75', brightYellow: '#657b83',
    brightBlue: '#839496', brightMagenta: '#6c71c4', brightCyan: '#93a1a1', brightWhite: '#fdf6e3',
  },
  monokai: {
    background: '#272822', foreground: '#f8f8f2', cursor: '#f8f8f2',
    cursorAccent: '#272822', selectionBackground: '#49483e99',
    black: '#272822', red: '#f92672', green: '#a6e22e', yellow: '#f4bf75',
    blue: '#66d9e8', magenta: '#ae81ff', cyan: '#a1efe4', white: '#f8f8f2',
    brightBlack: '#75715e', brightRed: '#f92672', brightGreen: '#a6e22e', brightYellow: '#f4bf75',
    brightBlue: '#66d9e8', brightMagenta: '#ae81ff', brightCyan: '#a1efe4', brightWhite: '#f9f8f5',
  },
  light: {
    background: '#ffffff', foreground: '#333333', cursor: '#333333',
    cursorAccent: '#ffffff', selectionBackground: '#b5d5ff99',
    black: '#000000', red: '#c82829', green: '#718c00', yellow: '#eab700',
    blue: '#4271ae', magenta: '#8959a8', cyan: '#3e999f', white: '#ffffff',
    brightBlack: '#8e908c', brightRed: '#c82829', brightGreen: '#718c00', brightYellow: '#eab700',
    brightBlue: '#4271ae', brightMagenta: '#8959a8', brightCyan: '#3e999f', brightWhite: '#ffffff',
  },
  iterm2: {
    background: '#101421', foreground: '#fffbf6', cursor: '#fffbf6',
    cursorAccent: '#101421', selectionBackground: '#ffffff40',
    black: '#2e2e2e',      red: '#eb4129',      green: '#abe047',     yellow: '#f6c744',
    blue: '#47a0f3',       magenta: '#7b5cb0',  cyan: '#64dbe8',      white: '#831f1e',
    brightBlack: '#740c02', brightRed: '#eb4129', brightGreen: '#abe047', brightYellow: '#f6c744',
    brightBlue: '#47a0f3',  brightMagenta: '#7b5cb0', brightCyan: '#64dbe8', brightWhite: '#fffbf6',
  },
  nord: {
    background: '#2e3440', foreground: '#d8dee9', cursor: '#d8dee9',
    cursorAccent: '#2e3440', selectionBackground: '#4c566a99',
    black: '#3b4252', red: '#bf616a', green: '#a3be8c', yellow: '#ebcb8b',
    blue: '#81a1c1', magenta: '#b48ead', cyan: '#88c0d0', white: '#e5e9f0',
    brightBlack: '#4c566a', brightRed: '#bf616a', brightGreen: '#a3be8c', brightYellow: '#ebcb8b',
    brightBlue: '#81a1c1', brightMagenta: '#b48ead', brightCyan: '#8fbcbb', brightWhite: '#eceff4',
  },
  gruvbox: {
    background: '#282828', foreground: '#ebdbb2', cursor: '#ebdbb2',
    cursorAccent: '#282828', selectionBackground: '#504945aa',
    black: '#282828', red: '#cc241d', green: '#98971a', yellow: '#d79921',
    blue: '#458588', magenta: '#b16286', cyan: '#689d6a', white: '#a89984',
    brightBlack: '#928374', brightRed: '#fb4934', brightGreen: '#b8bb26', brightYellow: '#fabd2f',
    brightBlue: '#83a598', brightMagenta: '#d3869b', brightCyan: '#8ec07c', brightWhite: '#ebdbb2',
  },
  tokyo: {
    background: '#1a1b26', foreground: '#a9b1d6', cursor: '#c0caf5',
    cursorAccent: '#1a1b26', selectionBackground: '#28344a99',
    black: '#15161e', red: '#f7768e', green: '#9ece6a', yellow: '#e0af68',
    blue: '#7aa2f7', magenta: '#bb9af7', cyan: '#7dcfff', white: '#a9b1d6',
    brightBlack: '#414868', brightRed: '#f7768e', brightGreen: '#9ece6a', brightYellow: '#e0af68',
    brightBlue: '#7aa2f7', brightMagenta: '#bb9af7', brightCyan: '#7dcfff', brightWhite: '#c0caf5',
  },
  catppuccin: {
    background: '#1e1e2e', foreground: '#cdd6f4', cursor: '#f5e0dc',
    cursorAccent: '#1e1e2e', selectionBackground: '#31324499',
    black: '#45475a', red: '#f38ba8', green: '#a6e3a1', yellow: '#f9e2af',
    blue: '#89b4fa', magenta: '#cba6f7', cyan: '#94e2d5', white: '#bac2de',
    brightBlack: '#585b70', brightRed: '#f38ba8', brightGreen: '#a6e3a1', brightYellow: '#f9e2af',
    brightBlue: '#89b4fa', brightMagenta: '#cba6f7', brightCyan: '#94e2d5', brightWhite: '#a6adc8',
  },
  synthwave: {
    background: '#262335', foreground: '#e2e0e7', cursor: '#ff7edb',
    cursorAccent: '#262335', selectionBackground: '#ff7edb33',
    black: '#262335', red: '#fe4450', green: '#72f1b8', yellow: '#f97e72',
    blue: '#03edf9', magenta: '#ff7edb', cyan: '#03edf9', white: '#e2e0e7',
    brightBlack: '#495495', brightRed: '#fe4450', brightGreen: '#72f1b8', brightYellow: '#fede5d',
    brightBlue: '#03edf9', brightMagenta: '#ff7edb', brightCyan: '#03edf9', brightWhite: '#ffffff',
  },
};

// Per-theme accent color shown on the tab badge/border
const THEME_ACCENT = {
  hacker:    '#00ff41',
  dark:      '#569cd6',
  dracula:   '#bd93f9',
  solarized: '#268bd2',
  monokai:   '#a6e22e',
  light:     '#4271ae',
  iterm2:    '#47a0f3',
  nord:      '#88c0d0',
  gruvbox:   '#fabd2f',
  tokyo:     '#7aa2f7',
  catppuccin:'#cba6f7',
  synthwave: '#ff7edb',
};

// ── State ──────────────────────────────────────────────────────────────────
let hosts = [];
let keys  = [];
let tabs  = [];
let activeTab = null;
let tabCounter = 0;
let vaultPassword = null; // held in JS memory only
let currentUser = null;   // username string when logged in

const VAULT_PW_KEY = 'webssh_vault_pw';

// ── API helpers ────────────────────────────────────────────────────────────
async function api(method, path, body) {
  const res = await fetch(path, {
    method,
    headers: body ? { 'Content-Type': 'application/json' } : {},
    body: body ? JSON.stringify(body) : undefined,
  });
  return res.json();
}

// ── Auth ──────────────────────────────────────────────────────────────────
let authIsRegister = false;

async function checkAuth() {
  const st = await api('GET', '/api/auth/status');
  if (st.logged_in) {
    currentUser = st.username;
    document.getElementById('sidebar-user').textContent = st.username;
    document.getElementById('auth-modal').style.display = 'none';
    return true;
  }
  // Show auth modal
  authIsRegister = st.needs_register;
  showAuthModal();
  return false;
}

function showAuthModal() {
  const modal = document.getElementById('auth-modal');
  document.getElementById('auth-modal-title').textContent = authIsRegister ? 'Create Account' : 'Login';
  document.getElementById('auth-submit-btn').textContent  = authIsRegister ? 'Register' : 'Login';
  document.getElementById('auth-toggle-btn').textContent  = authIsRegister ? 'Login instead' : 'Register instead';
  document.getElementById('auth-hint').textContent = authIsRegister
    ? 'No accounts yet. Create the first account.'
    : '';
  document.getElementById('auth-hint').style.color = '';
  modal.style.display = 'flex';
}

document.getElementById('auth-toggle-btn').addEventListener('click', () => {
  authIsRegister = !authIsRegister;
  document.getElementById('auth-hint').textContent = '';
  showAuthModal();
});

document.getElementById('auth-submit-btn').addEventListener('click', async () => {
  const username = document.getElementById('auth-username').value.trim();
  const password = document.getElementById('auth-password').value;
  if (!username || !password) return;
  const endpoint = authIsRegister ? '/api/auth/register' : '/api/auth/login';
  const res = await api('POST', endpoint, { username, password });
  if (res.ok) {
    currentUser = res.username;
    document.getElementById('sidebar-user').textContent = res.username;
    document.getElementById('auth-modal').style.display = 'none';
    document.getElementById('auth-password').value = '';
    await onLoggedIn();
  } else {
    const hint = document.getElementById('auth-hint');
    hint.textContent = res.error || 'Failed';
    hint.style.color = '#ff4444';
  }
});

document.getElementById('auth-username').addEventListener('keydown', e => {
  if (e.key === 'Enter') document.getElementById('auth-password').focus();
});
document.getElementById('auth-password').addEventListener('keydown', e => {
  if (e.key === 'Enter') document.getElementById('auth-submit-btn').click();
});

document.getElementById('btn-logout').addEventListener('click', async () => {
  await api('POST', '/api/auth/logout');
  currentUser = null;
  vaultPassword = null;
  try { sessionStorage.removeItem(VAULT_PW_KEY); } catch {}
  hosts = [];
  keys  = [];
  tabs.forEach(t => closeTab(t.id));
  document.getElementById('sidebar-user').textContent = '';
  document.getElementById('host-list').innerHTML = '';
  document.getElementById('key-list').innerHTML = '';
  checkAuth();
});

// ── Settings modal ────────────────────────────────────────────────────────
document.getElementById('btn-settings').addEventListener('click', () => {
  document.getElementById('settings-modal').style.display = 'flex';
  document.getElementById('overlay').style.display = 'block';
  document.getElementById('settings-hint').textContent =
    'Your personal key overrides the server key for encrypting vault data. Leave empty to use the server-managed key.';
  document.getElementById('settings-hint').style.color = '';
});

document.getElementById('settings-cancel-btn').addEventListener('click', () => {
  document.getElementById('settings-modal').style.display = 'none';
  document.getElementById('overlay').style.display = 'none';
});

document.getElementById('settings-save-btn').addEventListener('click', async () => {
  const key = document.getElementById('settings-vault-key').value.trim();
  const res = await api('POST', '/api/auth/settings', { vault_key_hex: key });
  const hint = document.getElementById('settings-hint');
  if (res.ok) {
    hint.textContent = 'Saved.';
    hint.style.color = 'var(--accent)';
    setTimeout(() => {
      document.getElementById('settings-modal').style.display = 'none';
      document.getElementById('overlay').style.display = 'none';
    }, 800);
  } else {
    hint.textContent = res.error || 'Failed';
    hint.style.color = '#ff4444';
  }
});

// ── Vault ──────────────────────────────────────────────────────────────────
async function checkVault() {
  const st = await api('GET', '/api/vault/status');
  if (!st.exists) { showVaultModal(false); return; }
  if (vaultPassword) return; // already unlocked in this JS context
  // Server already has the vault unlocked for this login session — just sync the password from sessionStorage
  if (st.unlocked) {
    const saved = (() => { try { return sessionStorage.getItem(VAULT_PW_KEY); } catch { return null; } })();
    if (saved) { vaultPassword = saved; }
    return; // vault is open server-side regardless
  }
  // Try to restore from sessionStorage (survives refresh, not tab close)
  const saved = (() => { try { return sessionStorage.getItem(VAULT_PW_KEY); } catch { return null; } })();
  if (saved) {
    const res = await api('POST', '/api/vault/unlock', { password: saved });
    if (res.ok) { vaultPassword = saved; return; }
    try { sessionStorage.removeItem(VAULT_PW_KEY); } catch {}
  }
  showVaultModal(true);
}

function showVaultModal(exists) {
  const modal = document.getElementById('vault-modal');
  document.getElementById('vault-modal-title').textContent = exists ? 'Unlock Vault' : 'Create Vault';
  document.getElementById('vault-hint').textContent = exists
    ? 'Enter your vault password to decrypt stored credentials.'
    : 'Create a master password to encrypt stored SSH passwords.';
  document.getElementById('vault-submit-btn').textContent = exists ? 'Unlock' : 'Create';
  modal.style.display = 'flex';
}

document.getElementById('vault-submit-btn').addEventListener('click', async () => {
  const pw = document.getElementById('vault-pw').value;
  if (!pw) return;
  const st = await api('GET', '/api/vault/status');
  const endpoint = st.exists ? '/api/vault/unlock' : '/api/vault/init';
  const res = await api('POST', endpoint, { password: pw });
  if (res.ok) {
    vaultPassword = pw;
    try { sessionStorage.setItem(VAULT_PW_KEY, pw); } catch {}
    document.getElementById('vault-modal').style.display = 'none';
    document.getElementById('vault-pw').value = '';
  } else {
    const hint = document.getElementById('vault-hint');
    hint.textContent = res.error || 'Failed';
    hint.style.color = '#ff4444';
  }
});

document.getElementById('vault-pw').addEventListener('keydown', e => {
  if (e.key === 'Enter') document.getElementById('vault-submit-btn').click();
});

document.getElementById('vault-skip-btn').addEventListener('click', () => {
  document.getElementById('vault-modal').style.display = 'none';
});

document.getElementById('btn-vault-lock').addEventListener('click', () => {
  vaultPassword = null;
  try { sessionStorage.removeItem(VAULT_PW_KEY); } catch {}
  api('POST', '/api/vault/lock');
});

// ── Hosts ──────────────────────────────────────────────────────────────────
async function loadHosts() {
  hosts = await api('GET', '/api/hosts');
  renderHosts();
}

function renderHosts() {
  const el = document.getElementById('host-list');
  el.innerHTML = '';
  for (const h of hosts) {
    const item = document.createElement('div');
    item.className = 'host-item';
    item.dataset.id = h.id;
    item.innerHTML = `
      <div class="host-icon">&#9654;</div>
      <div class="host-info">
        <div class="host-label">${esc(h.label || h.hostname)}</div>
        <div class="host-addr">${esc(h.username)}@${esc(h.hostname)}:${h.port || 22}</div>
      </div>
      <div class="host-actions">
        <button class="sm-btn btn-mosh" title="Connect via Mosh">mosh</button>
        <button class="sm-btn btn-edit" title="Edit">&#9998;</button>
      </div>`;
    item.querySelector('.btn-mosh').addEventListener('click', e => { e.stopPropagation(); openMosh(h.id); });
    item.querySelector('.btn-edit').addEventListener('click', e => { e.stopPropagation(); openDrawer(h.id); });
    item.addEventListener('click', () => openSSH(h.id));
    el.appendChild(item);
  }
}

// ── Keys ───────────────────────────────────────────────────────────────────
async function loadKeys() {
  keys = await api('GET', '/api/keys');
  renderKeys();
  updateKeySelect();
}

function renderKeys() {
  const el = document.getElementById('key-list');
  el.innerHTML = '';
  for (const k of keys) {
    const item = document.createElement('div');
    item.className = 'key-item';
    item.innerHTML = `
      <span class="key-name">${esc(k.name)}</span>
      <span style="color:var(--text-dim);font-size:10px;overflow:hidden;text-overflow:ellipsis;max-width:90px">${esc(k.path)}</span>
      <button class="sm-btn">&#10005;</button>`;
    item.querySelector('button').addEventListener('click', async () => {
      await api('DELETE', `/api/keys/${k.id}`);
      loadKeys();
    });
    el.appendChild(item);
  }
}

function updateKeySelect() {
  const sel = document.getElementById('h-key');
  sel.innerHTML = '<option value="">-- none --</option>';
  for (const k of keys) {
    const opt = document.createElement('option');
    opt.value = k.id;
    opt.textContent = k.name;
    sel.appendChild(opt);
  }
}

document.getElementById('btn-add-key').addEventListener('click', () => {
  document.getElementById('key-modal').style.display = 'flex';
  document.getElementById('overlay').style.display = 'block';
});
document.getElementById('key-cancel-btn').addEventListener('click', () => {
  document.getElementById('key-modal').style.display = 'none';
  document.getElementById('overlay').style.display = 'none';
});
document.getElementById('key-save-btn').addEventListener('click', async () => {
  const name = document.getElementById('key-name').value.trim();
  const path = document.getElementById('key-path').value.trim();
  if (!name || !path) return;
  await api('POST', '/api/keys', { name, path });
  document.getElementById('key-name').value = '';
  document.getElementById('key-path').value = '';
  document.getElementById('key-modal').style.display = 'none';
  document.getElementById('overlay').style.display = 'none';
  loadKeys();
});

// ── Host drawer ────────────────────────────────────────────────────────────
let drawerHostId = null;

document.getElementById('btn-new-host').addEventListener('click', () => openDrawer(null));
document.getElementById('drawer-close').addEventListener('click', closeDrawer);
document.getElementById('overlay').addEventListener('click', closeDrawer);

function openDrawer(hostId) {
  drawerHostId = hostId;
  const title  = document.getElementById('drawer-title');
  const delBtn = document.getElementById('drawer-delete');
  if (hostId) {
    const h = hosts.find(x => x.id === hostId);
    title.textContent = 'Edit Host';
    delBtn.style.display = 'inline-flex';
    document.getElementById('h-label').value    = h.label || '';
    document.getElementById('h-hostname').value = h.hostname || '';
    document.getElementById('h-port').value     = h.port || 22;
    document.getElementById('h-username').value = h.username || '';
    document.getElementById('h-password').value = '';
    document.getElementById('h-key').value      = h.key_id || '';
    document.getElementById('h-passphrase').value = '';
    document.getElementById('h-jump').value     = h.jump_host || '';
    document.getElementById('h-ssh-cmd').value  = h.ssh_command || '';
    document.getElementById('h-theme').value    = h.theme || 'iterm2';
  } else {
    title.textContent = 'New Host';
    delBtn.style.display = 'none';
    ['h-label','h-hostname','h-username','h-password','h-passphrase','h-jump','h-ssh-cmd'].forEach(id =>
      (document.getElementById(id).value = ''));
    document.getElementById('h-port').value  = 22;
    document.getElementById('h-key').value   = '';
    document.getElementById('h-theme').value = 'iterm2';
  }
  document.getElementById('host-drawer').classList.add('open');
  document.getElementById('overlay').style.display = 'block';
}

function closeDrawer() {
  document.getElementById('host-drawer').classList.remove('open');
  document.getElementById('overlay').style.display = 'none';
  drawerHostId = null;
}

document.getElementById('drawer-save').addEventListener('click', async () => {
  const keyId  = document.getElementById('h-key').value;
  const keyObj = keys.find(k => k.id === keyId);
  const payload = {
    label:          document.getElementById('h-label').value.trim(),
    hostname:       document.getElementById('h-hostname').value.trim(),
    port:           parseInt(document.getElementById('h-port').value) || 22,
    username:       document.getElementById('h-username').value.trim(),
    password:       document.getElementById('h-password').value,
    key_id:         keyId,
    key_path:       keyObj ? keyObj.path : '',
    key_passphrase: document.getElementById('h-passphrase').value,
    jump_host:      document.getElementById('h-jump').value.trim(),
    ssh_command:    document.getElementById('h-ssh-cmd').value.trim(),
    theme:          document.getElementById('h-theme').value,
    vault_password: vaultPassword,
  };
  if (drawerHostId) {
    await api('PUT', `/api/hosts/${drawerHostId}`, payload);
  } else {
    await api('POST', '/api/hosts', payload);
  }
  await loadHosts();
  closeDrawer();
});

document.getElementById('drawer-delete').addEventListener('click', async () => {
  if (!drawerHostId || !confirm('Delete this host?')) return;
  await api('DELETE', `/api/hosts/${drawerHostId}`);
  closeDrawer();
  loadHosts();
});

// ── Session persistence (server-side SQLite + Redis) ───────────────────────
// Each tab has a stable UUID `sessionId` used as the key in the DB.
// On page load we fetch the list from the server and reopen them,
// replaying scrollback so the terminal history is restored.

function onTabPersistenceChange() {
  // No-op: session upsert happens server-side when the WS connects.
  // Deletion from the server DB happens explicitly via closeTab.
}

async function restoreSessions() {
  let sessions;
  try {
    sessions = await api('GET', '/api/sessions');
    if (!Array.isArray(sessions)) return;
  } catch { return; }

  for (const s of sessions) {
    const prefs = { theme: s.theme || 'iterm2', fontSize: s.font_size || 13 };
    if (s.session_type === 'local') {
      openLocal(s.id, true, prefs);
    } else if ((s.session_type === 'start_ssh' || s.session_type === 'ssh') && s.host_id) {
      const h = hosts.find(x => x.id === s.host_id);
      if (h) openSSH(s.host_id, s.id, true, prefs);
    } else if ((s.session_type === 'start_mosh' || s.session_type === 'mosh') && s.host_id) {
      const h = hosts.find(x => x.id === s.host_id);
      if (h) openMosh(s.host_id, s.id, true, prefs);
    }
  }
}

async function replayScrollback(tab) {
  if (!tab.sessionId) return;
  try {
    const res = await api('GET', `/api/sessions/${tab.sessionId}/scrollback`);
    if (res.chunks && res.chunks.length > 0) {
      tab.term?.write('\x1b[2J\x1b[H'); // clear screen before replay
      for (const chunk of res.chunks) {
        tab.term?.write(chunk);
      }
      return true;
    }
  } catch { /* ignore */ }
  return false;
}

function generateSessionId() {
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, c => {
    const r = Math.random() * 16 | 0;
    return (c === 'x' ? r : (r & 0x3 | 0x8)).toString(16);
  });
}

// ── Tabs & terminals ───────────────────────────────────────────────────────
function createTab(label, theme, sessionType, hostId, sessionId, fontSize) {
  const id = `tab_${++tabCounter}`;
  const tab = {
    id,
    label,
    connected: false,
    persistent: true,
    theme: theme || 'iterm2',
    fontSize: fontSize || 13,
    ws: null,
    sessionType: sessionType || 'local',
    hostId: hostId || null,
    sessionId: sessionId || generateSessionId(),
  };
  tabs.push(tab);

  // Tab button
  const tabEl = document.createElement('div');
  tabEl.className = 'tab';
  tabEl.dataset.id = id;
  tabEl.innerHTML = `<div class="tab-dot connecting"></div><span>${esc(label)}</span><span class="tab-close">&#10005;</span>`;
  tabEl.addEventListener('click', e => { if (!e.target.classList.contains('tab-close')) activateTab(id); });
  tabEl.querySelector('.tab-close').addEventListener('click', e => { e.stopPropagation(); closeTab(id); });
  document.getElementById('tabs').appendChild(tabEl);
  tab.tabEl = tabEl;

  // Pane
  const pane = document.createElement('div');
  pane.className = 'terminal-pane';
  pane.dataset.id = id;

  const toolbar = document.createElement('div');
  toolbar.className = 'term-toolbar';
  toolbar.innerHTML = `
    <span class="conn-label">${esc(label)}</span>
    <span class="conn-status">connecting...</span>
    <div class="spacer"></div>
    <select class="sm-btn sel-theme" title="Color theme"></select>
    <button class="sm-btn btn-font-dec" title="Decrease font size">A-</button>
    <button class="sm-btn btn-font-inc" title="Increase font size">A+</button>
    <button class="sm-btn btn-persist" title="Keep session on refresh" style="color:var(--accent)">&#128190; persist</button>
    <button class="sm-btn btn-select" title="Toggle mouse selection mode (disables mouse forwarding to terminal)">&#128392; select</button>
    <button class="sm-btn btn-reconnect" style="display:none">reconnect</button>`;
  const selTheme = toolbar.querySelector('.sel-theme');
  Object.keys(THEMES).forEach(name => {
    const opt = document.createElement('option');
    opt.value = name;
    opt.textContent = name;
    selTheme.appendChild(opt);
  });
  selTheme.value = tab.theme;
  selTheme.addEventListener('change', () => applyTheme(id, selTheme.value));
  toolbar.querySelector('.btn-font-dec').addEventListener('click', () => changeFontSize(id, -1));
  toolbar.querySelector('.btn-font-inc').addEventListener('click', () => changeFontSize(id, +1));
  toolbar.querySelector('.btn-persist').addEventListener('click',   () => togglePersist(id));
  toolbar.querySelector('.btn-select').addEventListener('click',    () => toggleSelectMode(id));
  toolbar.querySelector('.btn-reconnect').addEventListener('click', () => { if (tab._reconnect) tab._reconnect(); });
  pane.appendChild(toolbar);
  tab.toolbar = toolbar;

  const xtermDiv = document.createElement('div');
  xtermDiv.className = 'xterm-container';
  pane.appendChild(xtermDiv);
  tab.xtermDiv = xtermDiv;

  // Debug log strip
  const dbgStrip = document.createElement('div');
  dbgStrip.className = 'dbg-strip';
  dbgStrip.innerHTML = `
    <div class="dbg-header">
      <span class="dbg-title">DEBUG</span>
      <div class="spacer"></div>
      <button class="sm-btn dbg-clear">clear</button>
      <button class="sm-btn dbg-toggle">&#9660;</button>
    </div>
    <div class="dbg-log"></div>`;
  dbgStrip.querySelector('.dbg-clear').addEventListener('click', () => {
    dbgStrip.querySelector('.dbg-log').innerHTML = '';
  });
  dbgStrip.querySelector('.dbg-toggle').addEventListener('click', () => {
    dbgStrip.classList.toggle('collapsed');
    dbgStrip.querySelector('.dbg-toggle').textContent = dbgStrip.classList.contains('collapsed') ? '▲' : '▼';
    tab.fitAddon?.fit();
  });
  pane.appendChild(dbgStrip);
  tab.dbgStrip = dbgStrip;

  document.getElementById('terminals').appendChild(pane);
  tab.pane = pane;

  document.getElementById('empty-state').style.display = 'none';
  activateTab(id);
  requestAnimationFrame(() => { initTerm(tab); applyTheme(id, tab.theme); });
  return tab;
}

function initTerm(tab) {
  const term = new Terminal({
    theme: THEMES[tab.theme] || THEMES.iterm2,
    fontFamily: "'Cascadia Code','Fira Code','JetBrains Mono','Courier New',monospace",
    fontSize: tab.fontSize || 13,
    lineHeight: 1.2,
    cursorBlink: true,
    cursorStyle: 'block',
    allowTransparency: true,
    scrollback: 5000,
    macOptionIsMeta: true,
    copyOnSelect: true,
    rightClickSelectsWord: true,
  });
  const fitAddon   = new FitAddon.FitAddon();
  const linksAddon = new WebLinksAddon.WebLinksAddon();
  term.loadAddon(fitAddon);
  term.loadAddon(linksAddon);
  term.open(tab.xtermDiv);
  fitAddon.fit();
  // Ensure xterm starts with mouse tracking disabled
  term.write('\x1b[?1003l\x1b[?1002l\x1b[?1000l\x1b[?1006l\x1b[?1015l');
  tab.term     = term;
  tab.fitAddon = fitAddon;

  term.onData(data => {
    if (tab.ws && tab.ws.readyState === WebSocket.OPEN) {
      tab.ws.send(JSON.stringify({ type: 'input', data }));
    }
  });

  // Alt+W closes the tab even when xterm has focus
  term.onKey(({ domEvent: e }) => {
    if (e.altKey && e.key === 'w') { e.preventDefault(); closeTab(tab.id); }
  });

  const ro = new ResizeObserver(() => {
    if (tab.pane.classList.contains('active')) {
      fitAddon.fit();
      if (tab.ws && tab.ws.readyState === WebSocket.OPEN) {
        tab.ws.send(JSON.stringify({ type: 'resize', cols: term.cols, rows: term.rows }));
        dbgLog(tab, 'send', `resize ${term.cols}x${term.rows}`);
      }
    }
  });
  ro.observe(tab.xtermDiv);
  tab.ro = ro;
}

function activateTab(id) {
  const tab = tabs.find(t => t.id === id);
  if (!tab) return;
  if (activeTab && activeTab.id !== id) {
    activeTab.pane.classList.remove('active');
    activeTab.tabEl.classList.remove('active');
  }
  activeTab = tab;
  tab.pane.classList.add('active');
  tab.tabEl.classList.add('active');
  requestAnimationFrame(() => { tab.fitAddon?.fit(); tab.term?.focus(); });
}

function closeTab(id) {
  const tab = tabs.find(t => t.id === id);
  if (!tab) return;
  if (!confirm(`Close "${tab.label}"?`)) return;
  if (tab.ws) { tab.ws.send(JSON.stringify({ type: 'close' })); tab.ws.close(); }
  // Delete session + scrollback from the server DB
  if (tab.sessionId) {
    api('DELETE', `/api/sessions/${tab.sessionId}`).catch(() => {});
  }
  tab.pane.remove();
  tab.tabEl.remove();
  tab.ro?.disconnect();
  tabs = tabs.filter(t => t.id !== id);
  if (activeTab?.id === id) {
    activeTab = null;
    if (tabs.length) activateTab(tabs[tabs.length - 1].id);
    else document.getElementById('empty-state').style.display = 'flex';
  }
}

function setTabConnected(tab, val) {
  tab.connected = val;
  const dot = tab.tabEl.querySelector('.tab-dot');
  dot.className = 'tab-dot ' + (val ? '' : 'disconnected');
  const status  = tab.toolbar.querySelector('.conn-status');
  const reconBtn = tab.toolbar.querySelector('.btn-reconnect');
  status.textContent = val ? 'connected' : 'disconnected';
  status.style.color = val ? 'var(--accent)' : 'var(--danger)';
  reconBtn.style.display = val ? 'none' : 'inline-block';
}

function togglePersist(id) {
  // Sessions are always persisted server-side (SQLite). This button is a no-op
  // kept for UI compatibility.
}

function toggleSelectMode(id) {
  const tab = tabs.find(t => t.id === id);
  if (!tab) return;
  tab.selectMode = !tab.selectMode;
  const btn = tab.toolbar.querySelector('.btn-select');
  // .xterm-mouse-area is the layer xterm uses to capture mouse events for PTY forwarding.
  // Setting pointer-events:none lets browser handle mouse events for text selection instead.
  const mouseArea = tab.xtermDiv.querySelector('.xterm-mouse-area');
  if (mouseArea) mouseArea.style.pointerEvents = tab.selectMode ? 'none' : '';
  btn.style.color = tab.selectMode ? 'var(--accent)' : '';
  btn.title = tab.selectMode
    ? 'Selection mode ON — mouse clicks go to browser (click to disable)'
    : 'Toggle mouse selection mode (disables mouse forwarding to terminal)';
}

function applyTheme(id, name) {
  const tab = tabs.find(t => t.id === id);
  if (!tab || !THEMES[name]) return;
  tab.theme = name;
  if (tab.term) tab.term.options.theme = THEMES[name];
  tab.toolbar.querySelector('.sel-theme').value = name;
  // Update tab badge accent color
  const accent = THEME_ACCENT[name] || '#00ff41';
  tab.tabEl.style.setProperty('--tab-accent', accent);
  tab.tabEl.querySelector('.tab-dot').style.background = accent;
  // Persist to session DB only (does not affect the host profile)
  persistTabPrefs(tab);
}

function changeFontSize(id, delta) {
  const tab = tabs.find(t => t.id === id);
  if (!tab?.term) return;
  const next = Math.max(8, Math.min(32, tab.term.options.fontSize + delta));
  tab.term.options.fontSize = next;
  tab.fontSize = next;
  tab.fitAddon?.fit();
  persistTabPrefs(tab);
}

function persistTabPrefs(tab) {
  if (!tab.sessionId) return;
  api('PATCH', `/api/sessions/${tab.sessionId}`, {
    theme: tab.theme,
    font_size: tab.fontSize || tab.term?.options.fontSize || 13,
  }).catch(() => {});
}

// ── WebSocket connection ───────────────────────────────────────────────────
function openWs(tab, startMsg) {
  // Always include the stable session_id so the server can persist it
  const msg = Object.assign({ session_id: tab.sessionId }, startMsg);
  const proto = location.protocol === 'https:' ? 'wss' : 'ws';
  const ws = new WebSocket(`${proto}://${location.host}/ws`);
  tab.ws = ws;

  ws.onopen = () => {
    dbgLog(tab, 'info', `WS open → sending ${msg.type}`);
    ws.send(JSON.stringify(msg));
  };

  ws.onmessage = ({ data }) => {
    let m;
    try { m = JSON.parse(data); } catch { return; }
    switch (m.type) {
      case 'connected':
        dbgLog(tab, 'info', 'Session connected');
        setTabConnected(tab, true);
        // Double-rAF ensures the pane is fully laid out before fit+resize,
        // which matters on restore when multiple tabs are created at once.
        requestAnimationFrame(() => requestAnimationFrame(() => {
          if (!tab.term) return;
          tab.fitAddon?.fit();
          const resizeMsg = { type: 'resize', cols: tab.term.cols, rows: tab.term.rows };
          dbgLog(tab, 'send', `resize ${tab.term.cols}x${tab.term.rows}`);
          ws.send(JSON.stringify(resizeMsg));
        }));
        // Replay scrollback after initial connect (new session: no-op; restore: history)
        replayScrollback(tab);
        break;
      case 'output':
        tab.term?.write(m.data);
        break;
      case 'disconnected':
        dbgLog(tab, 'warn', 'Session disconnected');
        setTabConnected(tab, false);
        // Write mouse-disable sequences into xterm so it processes them as terminal
        // output and clears its own mouse-tracking state. This prevents xterm from
        // continuing to send mouse reports as PTY input after reconnect.
        // Do NOT send these as PTY input — the shell would print them as text.
        tab.term?.write('\x1b[?1003l\x1b[?1002l\x1b[?1000l\x1b[?1006l\x1b[?1015l');
        break;
      case 'error':
        dbgLog(tab, 'error', m.message);
        tab.term?.write(`\r\n\x1b[31m[ERROR] ${m.message}\x1b[0m\r\n`);
        setTabConnected(tab, false);
        break;
    }
  };

  ws.onclose = (e) => {
    dbgLog(tab, 'warn', `WS closed (code=${e.code})`);
    setTabConnected(tab, false);
  };
  ws.onerror = () => {
    dbgLog(tab, 'error', 'WS error');
    setTabConnected(tab, false);
  };
}

// ── Open sessions ──────────────────────────────────────────────────────────
function openLocal(sessionId, restore, prefs) {
  const theme    = prefs?.theme    || 'iterm2';
  const fontSize = prefs?.fontSize || 13;
  const tab = createTab('local', theme, 'local', null, sessionId, fontSize);
  tab._reconnect = () => openWs(tab, { type: 'start_local', cols: tab.term?.cols || 220, rows: tab.term?.rows || 50 });
  setTimeout(async () => {
    await replayScrollback(tab);
    tab._reconnect();
  }, 80);
}

function openSSH(hostId, sessionId, restore, prefs) {
  const h = hosts.find(x => x.id === hostId);
  if (!h) return;
  const theme    = prefs?.theme    || h.theme || 'iterm2';
  const fontSize = prefs?.fontSize || 13;
  const tab = createTab(h.label || h.hostname, theme, 'ssh', hostId, sessionId, fontSize);
  const doConnect = () => openWs(tab, {
    type: 'start_ssh',
    host_id: hostId,
    cols: tab.term?.cols || 220,
    rows: tab.term?.rows || 50,
    vault_password: vaultPassword,
  });
  tab._reconnect = doConnect;
  setTimeout(async () => {
    await replayScrollback(tab);
    doConnect();
  }, 80);
}

function openMosh(hostId, sessionId, restore, prefs) {
  const h = hosts.find(x => x.id === hostId);
  if (!h) return;
  const theme    = prefs?.theme    || h.theme || 'iterm2';
  const fontSize = prefs?.fontSize || 13;
  const tab = createTab(`mosh:${h.label || h.hostname}`, theme, 'mosh', hostId, sessionId, fontSize);
  const doConnect = () => openWs(tab, {
    type: 'start_mosh',
    host_id: hostId,
    cols: tab.term?.cols || 220,
    rows: tab.term?.rows || 50,
  });
  tab._reconnect = doConnect;
  setTimeout(async () => {
    await replayScrollback(tab);
    doConnect();
  }, 80);
}

// ── Warn before closing the browser tab/window ─────────────────────────────
window.addEventListener('beforeunload', e => {
  if (tabs.length > 0) {
    e.preventDefault();
    e.returnValue = ''; // required for Chrome to show the dialog
  }
});

// ── Toolbar ────────────────────────────────────────────────────────────────
document.getElementById('btn-new-tab').addEventListener('click', () => openLocal());

// ── Keyboard shortcuts ─────────────────────────────────────────────────────
// Use capture phase on window so we catch events before xterm consumes them.
// Note: Cmd+W and Ctrl+W are intercepted by the browser before JS ever fires —
// only Alt+W is reliably capturable from a web page.
window.addEventListener('keydown', e => {
  if (e.ctrlKey && e.key === 't') { e.preventDefault(); openLocal(); }
  if (e.altKey && e.key === 'w')  { e.preventDefault(); if (activeTab) closeTab(activeTab.id); }
}, { capture: true });

// ── Debug log ──────────────────────────────────────────────────────────────
function dbgLog(tab, level, msg) {
  if (!tab.dbgStrip) return;
  const log = tab.dbgStrip.querySelector('.dbg-log');
  const ts  = new Date().toISOString().slice(11, 23);
  const colors = { info: '#00ff41', warn: '#ffaa00', error: '#ff4444', recv: '#00ccff', send: '#cc88ff' };
  const line = document.createElement('div');
  line.className = 'dbg-line';
  line.innerHTML = `<span class="dbg-ts">${ts}</span><span class="dbg-level" style="color:${colors[level]||'#c8c8c8'}">${level.toUpperCase()}</span><span class="dbg-msg">${esc(msg)}</span>`;
  log.appendChild(line);
  while (log.children.length > 200) log.removeChild(log.firstChild);
  log.scrollTop = log.scrollHeight;
}

// ── Utility ────────────────────────────────────────────────────────────────
function esc(s) {
  return String(s || '').replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;');
}

// ── Post-login init ────────────────────────────────────────────────────────
async function onLoggedIn() {
  await Promise.all([loadHosts(), loadKeys()]);
  await checkVault();
  await restoreSessions();
}

// ── Init ───────────────────────────────────────────────────────────────────
(async () => {
  const loggedIn = await checkAuth();
  if (loggedIn) await onLoggedIn();
})();
