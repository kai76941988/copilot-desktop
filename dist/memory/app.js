const { invoke } = window.__TAURI__ || {};

const state = {
  projects: [],
  sessions: [],
  selectedProjectId: null,
  selectedSessionId: null,
};

const elProjectList = document.getElementById("projectList");
const elSessionList = document.getElementById("sessionList");
const elContext = document.getElementById("contextPack");
const elStatus = document.getElementById("status");
const elSearchInput = document.getElementById("searchInput");
const elSearchResults = document.getElementById("searchResults");
const elSearchScope = document.getElementById("searchScope");
const elSearchProjectOnly = document.getElementById("searchProjectOnly");
const elProjectSummary = document.getElementById("projectSummary");
const elSummaryList = document.getElementById("summaryList");
const elMessageList = document.getElementById("messageList");
const elSummarizerMode = document.getElementById("summarizerMode");
const elSummarizerEndpoint = document.getElementById("summarizerEndpoint");
const elSummarizerApiKey = document.getElementById("summarizerApiKey");
const elSummarizerModel = document.getElementById("summarizerModel");
const elSummarizerPrompt = document.getElementById("summarizerPrompt");

let messageOffset = 0;
const messageLimit = 40;

function setStatus(text) {
  elStatus.textContent = text;
}

function formatTime(ms) {
  const d = new Date(ms);
  return d.toLocaleString();
}

function renderProjects() {
  elProjectList.innerHTML = "";
  if (!state.projects.length) {
    elProjectList.innerHTML = '<div class="list-item">No projects yet</div>';
    return;
  }

  state.projects.forEach((p) => {
    const item = document.createElement("div");
    item.className = "list-item" + (p.project_id === state.selectedProjectId ? " active" : "");
    item.innerHTML = `<div>${p.name}</div><small>${formatTime(p.updated_at)}</small>`;
    item.onclick = () => selectProject(p.project_id);
    elProjectList.appendChild(item);
  });
}

function renderSessions() {
  elSessionList.innerHTML = "";
  if (!state.sessions.length) {
    elSessionList.innerHTML = '<div class="list-item">No sessions yet</div>';
    return;
  }

  state.sessions.forEach((s) => {
    const item = document.createElement("div");
    item.className = "list-item" + (s.session_id === state.selectedSessionId ? " active" : "");
    item.innerHTML = `<div>${s.title || s.session_id.slice(0, 10)}</div><small>${formatTime(
      s.updated_at
    )}</small>`;
    item.onclick = () => {
      state.selectedSessionId = s.session_id;
      renderSessions();
      loadSummaries();
      loadMessages(false);
    };
    elSessionList.appendChild(item);
  });
}

function renderSearchResults(items) {
  elSearchResults.innerHTML = "";
  if (!items.length) {
    elSearchResults.innerHTML = '<div class="list-item">No results</div>';
    return;
  }
  items.forEach((m) => {
    const item = document.createElement("div");
    item.className = "list-item";
    const preview = (m.content || "").slice(0, 160);
    const type = m.summary_type ? `[${m.summary_type}] ` : "";
    item.innerHTML = `<div>${type}${m.role ? m.role + ":" : ""} ${preview}</div><small>${formatTime(
      m.created_at
    )}</small>`;
    elSearchResults.appendChild(item);
  });
}

function renderSummaries(items) {
  elSummaryList.innerHTML = "";
  if (!items.length) {
    elSummaryList.innerHTML = '<div class="list-item">No summaries yet</div>';
    return;
  }
  items.forEach((s) => {
    const item = document.createElement("div");
    item.className = "list-item";
    item.innerHTML = `<div>[${s.summary_type}] ${s.content.slice(0, 160)}</div><small>${formatTime(
      s.created_at
    )}</small>`;
    elSummaryList.appendChild(item);
  });
}

function renderMessages(items, append) {
  if (!append) {
    elMessageList.innerHTML = "";
  }
  if (!items.length && !append) {
    elMessageList.innerHTML = '<div class="list-item">No messages yet</div>';
    return;
  }
  items.forEach((m) => {
    const item = document.createElement("div");
    item.className = "list-item";
    item.innerHTML = `<div>${m.role}: ${m.content.slice(0, 200)}</div><small>${formatTime(
      m.created_at
    )}</small>`;
    elMessageList.appendChild(item);
  });
}

function renderProjectSummary(content) {
  if (!content) {
    elProjectSummary.textContent = "No summary yet.";
    return;
  }
  try {
    const data = JSON.parse(content);
    const join = (arr) => (Array.isArray(arr) && arr.length ? arr.join("\n") : "暂无");
    const text = [
      "【项目背景】",
      join(data.background),
      "",
      "【关键结论】",
      join(data.key_facts),
      "",
      "【最近进展】",
      join(data.progress),
      "",
      "【当前待办】",
      join(data.todo),
      "",
      "【必须遵守的约束】",
      join(data.constraints),
    ].join("\n");
    elProjectSummary.textContent = text;
  } catch (e) {
    elProjectSummary.textContent = content;
  }
}

async function loadProjects() {
  if (!invoke) return;
  setStatus("Loading projects...");
  state.projects = await invoke("memory_list_projects");
  if (!state.selectedProjectId && state.projects.length) {
    state.selectedProjectId = state.projects[0].project_id;
  }
  renderProjects();
  await loadSessions();
  await loadSummaries();
  await loadMessages(false);
  await loadSummarizerConfig();
  setStatus("Ready");
}

async function loadSessions() {
  if (!invoke || !state.selectedProjectId) return;
  state.sessions = await invoke("memory_list_sessions", { project_id: state.selectedProjectId });
  if (!state.selectedSessionId && state.sessions.length) {
    state.selectedSessionId = state.sessions[0].session_id;
  }
  renderSessions();
}

async function loadSummaries() {
  if (!invoke || !state.selectedProjectId) return;
  const projectSummary = await invoke("memory_list_summaries", {
    project_id: state.selectedProjectId,
    summary_type: "project",
    limit: 1,
  });
  const structSummary = await invoke("memory_list_summaries", {
    project_id: state.selectedProjectId,
    summary_type: "project_struct",
    limit: 1,
  });
  if (structSummary && structSummary.length) {
    renderProjectSummary(structSummary[0].content);
  } else if (projectSummary && projectSummary.length) {
    renderProjectSummary(projectSummary[0].content);
  } else {
    renderProjectSummary("");
  }

  const summaries = await invoke("memory_list_summaries", {
    project_id: state.selectedProjectId,
    session_id: state.selectedSessionId,
    limit: 30,
  });
  renderSummaries(summaries || []);
}

async function loadMessages(append) {
  if (!invoke || !state.selectedSessionId) return;
  if (!append) {
    messageOffset = 0;
  }
  const items = await invoke("memory_list_messages", {
    session_id: state.selectedSessionId,
    limit: messageLimit,
    offset: messageOffset,
  });
  renderMessages(items || [], append);
  messageOffset += messageLimit;
}

async function loadSummarizerConfig() {
  if (!invoke) return;
  try {
    const cfg = await invoke("memory_get_summarizer_config");
    elSummarizerMode.value = cfg.mode || "rule";
    elSummarizerEndpoint.value = cfg.endpoint || "";
    elSummarizerApiKey.value = cfg.api_key || "";
    elSummarizerModel.value = cfg.model || "";
    elSummarizerPrompt.value = cfg.prompt || "";
  } catch (e) {
    console.warn("load summarizer config failed", e);
  }
}

async function saveSummarizerConfig() {
  if (!invoke) return;
  const cfg = {
    mode: elSummarizerMode.value,
    endpoint: elSummarizerEndpoint.value || null,
    api_key: elSummarizerApiKey.value || null,
    model: elSummarizerModel.value || null,
    prompt: elSummarizerPrompt.value || null,
    timeout_ms: 15000,
  };
  await invoke("memory_set_summarizer_config", cfg);
  setStatus("Summarizer config saved");
}

async function runSummarizer() {
  if (!invoke || !state.selectedProjectId) return;
  setStatus("Running summarizer...");
  await invoke("memory_refresh_summaries", {
    project_id: state.selectedProjectId,
    session_id: state.selectedSessionId,
  });
  await loadSummaries();
  setStatus("Summaries refreshed");
}

async function selectProject(projectId) {
  state.selectedProjectId = projectId;
  state.selectedSessionId = null;
  renderProjects();
  await loadSessions();
  await loadSummaries();
  await loadMessages(false);
}

async function generateContext() {
  if (!invoke || !state.selectedProjectId) return;
  setStatus("Generating context pack...");
  const pack = await invoke("memory_get_context_pack", {
    project_id: state.selectedProjectId,
    session_id: state.selectedSessionId,
  });
  elContext.value = pack;
  setStatus("Context pack ready");
}

async function continueInCopilot() {
  if (!invoke || !state.selectedProjectId) return;
  setStatus("Sending to Copilot...");
  const pack = await invoke("memory_continue_project", {
    project_id: state.selectedProjectId,
    session_id: state.selectedSessionId,
    open_new: true,
  });
  elContext.value = pack;
  setStatus("Context pack sent to Copilot");
}

async function setActiveProject() {
  if (!invoke || !state.selectedProjectId) return;
  await invoke("memory_set_active_project", { project_id: state.selectedProjectId });
  setStatus("Active project set for capture");
}

async function doSearch() {
  if (!invoke) return;
  const query = elSearchInput.value.trim();
  if (!query) {
    renderSearchResults([]);
    return;
  }
  setStatus("Searching...");
  const scope = elSearchScope.value;
  const projectOnly = elSearchProjectOnly.checked;
  if (scope === "summaries") {
    const results = await invoke("memory_search_summaries", {
      query,
      project_id: projectOnly ? state.selectedProjectId : null,
      limit: 50,
    });
    renderSearchResults(results);
    setStatus(`Found ${results.length} summary results`);
  } else {
    const results = await invoke("memory_search_messages", {
      query,
      project_id: projectOnly ? state.selectedProjectId : null,
      limit: 50,
    });
    renderSearchResults(results);
    setStatus(`Found ${results.length} message results`);
  }
}

document.getElementById("btnRefresh").onclick = loadProjects;
document.getElementById("btnNewProject").onclick = async () => {
  if (!invoke) return;
  const name = prompt("Project name");
  if (!name) return;
  const description = prompt("Description (optional)") || null;
  await invoke("memory_create_project", { name, description });
  await loadProjects();
};
document.getElementById("btnGenerate").onclick = generateContext;
document.getElementById("btnCopy").onclick = async () => {
  if (elContext.value) {
    await navigator.clipboard.writeText(elContext.value);
    setStatus("Context pack copied");
  }
};
document.getElementById("btnContinue").onclick = continueInCopilot;
document.getElementById("btnSetActive").onclick = setActiveProject;
document.getElementById("btnSearch").onclick = doSearch;
document.getElementById("btnRefreshSummary").onclick = loadSummaries;
document.getElementById("btnLoadMore").onclick = () => loadMessages(true);
document.getElementById("btnSaveSummarizer").onclick = saveSummarizerConfig;
document.getElementById("btnRunSummarizer").onclick = runSummarizer;
elSearchInput.addEventListener("keydown", (e) => {
  if (e.key === "Enter") {
    doSearch();
  }
});

loadProjects().catch((e) => {
  console.error(e);
  setStatus("Failed to load");
});
