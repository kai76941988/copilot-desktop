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
    item.innerHTML = `<div>${m.role}: ${m.content.slice(0, 160)}</div><small>${formatTime(
      m.created_at
    )}</small>`;
    elSearchResults.appendChild(item);
  });
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

async function selectProject(projectId) {
  state.selectedProjectId = projectId;
  state.selectedSessionId = null;
  renderProjects();
  await loadSessions();
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
  const results = await invoke("memory_search_messages", {
    query,
    project_id: state.selectedProjectId,
    limit: 50,
  });
  renderSearchResults(results);
  setStatus(`Found ${results.length} results`);
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
elSearchInput.addEventListener("keydown", (e) => {
  if (e.key === "Enter") {
    doSearch();
  }
});

loadProjects().catch((e) => {
  console.error(e);
  setStatus("Failed to load");
});
