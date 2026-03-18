const store = require('../../js/memory/store.js')

const state = {
  projects: [],
  sessions: [],
  selectedProjectId: null,
  selectedSessionId: null,
  messageOffset: 0
}

const elProjectList = document.getElementById('projectList')
const elSessionList = document.getElementById('sessionList')
const elContext = document.getElementById('contextPack')
const elStatus = document.getElementById('status')
const elSearchInput = document.getElementById('searchInput')
const elSearchResults = document.getElementById('searchResults')
const elSearchScope = document.getElementById('searchScope')
const elSearchProjectOnly = document.getElementById('searchProjectOnly')
const elProjectSummary = document.getElementById('projectSummary')
const elSummaryList = document.getElementById('summaryList')
const elMessageList = document.getElementById('messageList')

const messageLimit = 40

function setStatus (text) {
  elStatus.textContent = text
}

function formatTime (ms) {
  try {
    return new Date(ms).toLocaleString()
  } catch (e) {
    return ''
  }
}

function renderProjects () {
  elProjectList.innerHTML = ''
  if (!state.projects.length) {
    elProjectList.innerHTML = '<div class="list-item">No projects yet</div>'
    return
  }
  state.projects.forEach(p => {
    const item = document.createElement('div')
    item.className = 'list-item' + (p.project_id === state.selectedProjectId ? ' active' : '')
    item.innerHTML = `<div>${p.name}</div><small>${formatTime(p.updated_at)}</small>`
    item.onclick = () => selectProject(p.project_id)
    elProjectList.appendChild(item)
  })
}

function renderSessions () {
  elSessionList.innerHTML = ''
  if (!state.sessions.length) {
    elSessionList.innerHTML = '<div class="list-item">No sessions yet</div>'
    return
  }
  state.sessions.forEach(s => {
    const item = document.createElement('div')
    item.className = 'list-item' + (s.session_id === state.selectedSessionId ? ' active' : '')
    item.innerHTML = `<div>${s.title || s.session_id.slice(0, 10)}</div><small>${formatTime(s.updated_at)}</small>`
    item.onclick = () => {
      state.selectedSessionId = s.session_id
      renderSessions()
      loadSummaries()
      loadMessages(false)
    }
    elSessionList.appendChild(item)
  })
}

function renderSummaries (items) {
  elSummaryList.innerHTML = ''
  if (!items.length) {
    elSummaryList.innerHTML = '<div class="list-item">No summaries yet</div>'
    return
  }
  items.forEach(s => {
    const item = document.createElement('div')
    item.className = 'list-item'
    item.innerHTML = `<div>[${s.summary_type}] ${s.content.slice(0, 160)}</div><small>${formatTime(s.created_at)}</small>`
    elSummaryList.appendChild(item)
  })
}

function renderMessages (items, append) {
  if (!append) elMessageList.innerHTML = ''
  if (!items.length && !append) {
    elMessageList.innerHTML = '<div class="list-item">No messages yet</div>'
    return
  }
  items.forEach(m => {
    const item = document.createElement('div')
    item.className = 'list-item'
    item.innerHTML = `<div>${m.role}: ${m.content.slice(0, 200)}</div><small>${formatTime(m.created_at)}</small>`
    elMessageList.appendChild(item)
  })
}

function renderSearchResults (items) {
  elSearchResults.innerHTML = ''
  if (!items.length) {
    elSearchResults.innerHTML = '<div class="list-item">No results</div>'
    return
  }
  items.forEach(m => {
    const item = document.createElement('div')
    item.className = 'list-item'
    const preview = (m.content || '').slice(0, 160)
    const type = m.summary_type ? `[${m.summary_type}] ` : ''
    item.innerHTML = `<div>${type}${m.role ? m.role + ':' : ''} ${preview}</div><small>${formatTime(m.created_at)}</small>`
    elSearchResults.appendChild(item)
  })
}

function renderProjectSummary (content) {
  if (!content) {
    elProjectSummary.textContent = 'No summary yet.'
    return
  }
  try {
    const data = JSON.parse(content)
    const join = (arr) => (Array.isArray(arr) && arr.length ? arr.join('\n') : '暂无')
    elProjectSummary.textContent = [
      '【项目背景】',
      join(data.background),
      '',
      '【关键结论】',
      join(data.key_facts),
      '',
      '【最近进展】',
      join(data.progress),
      '',
      '【当前待办】',
      join(data.todo),
      '',
      '【必须遵守的约束】',
      join(data.constraints)
    ].join('\n')
  } catch (e) {
    elProjectSummary.textContent = content
  }
}

async function loadProjects () {
  setStatus('Loading projects...')
  state.projects = await store.listProjects()
  if (!state.selectedProjectId && state.projects.length) {
    state.selectedProjectId = state.projects[0].project_id
  }
  renderProjects()
  await loadSessions()
  await loadSummaries()
  await loadMessages(false)
  setStatus('Ready')
}

async function loadSessions () {
  if (!state.selectedProjectId) return
  state.sessions = await store.listSessions(state.selectedProjectId)
  if (!state.selectedSessionId && state.sessions.length) {
    state.selectedSessionId = state.sessions[0].session_id
  }
  renderSessions()
}

async function loadSummaries () {
  if (!state.selectedProjectId) return
  const structSummary = await store.listSummaries(state.selectedProjectId, null, 'project_struct', 1)
  const projectSummary = await store.listSummaries(state.selectedProjectId, null, 'project', 1)
  if (structSummary && structSummary.length) {
    renderProjectSummary(structSummary[0].content)
  } else if (projectSummary && projectSummary.length) {
    renderProjectSummary(projectSummary[0].content)
  } else {
    renderProjectSummary('')
  }
  const summaries = await store.listSummaries(state.selectedProjectId, state.selectedSessionId, null, 30)
  renderSummaries(summaries || [])
}

async function loadMessages (append) {
  if (!state.selectedSessionId) return
  if (!append) state.messageOffset = 0
  const items = await store.listMessages(state.selectedSessionId, messageLimit, state.messageOffset)
  renderMessages(items || [], append)
  state.messageOffset += messageLimit
}

async function selectProject (projectId) {
  state.selectedProjectId = projectId
  state.selectedSessionId = null
  renderProjects()
  await loadSessions()
  await loadSummaries()
  await loadMessages(false)
}

async function generateContext () {
  if (!state.selectedProjectId) return
  setStatus('Generating context pack...')
  const pack = await store.getContextPack(state.selectedProjectId, state.selectedSessionId)
  elContext.value = pack
  setStatus('Context pack ready')
}

async function doSearch () {
  const query = elSearchInput.value.trim()
  if (!query) {
    renderSearchResults([])
    return
  }
  setStatus('Searching...')
  const scope = elSearchScope.value
  const projectOnly = elSearchProjectOnly.checked
  if (scope === 'summaries') {
    const results = await store.searchSummaries(query, projectOnly ? state.selectedProjectId : null, 50)
    renderSearchResults(results)
    setStatus(`Found ${results.length} summary results`)
  } else {
    const results = await store.searchMessages(query, projectOnly ? state.selectedProjectId : null, 50)
    renderSearchResults(results)
    setStatus(`Found ${results.length} message results`)
  }
}

async function setActiveProject () {
  if (!state.selectedProjectId) return
  await store.setActiveProject(state.selectedProjectId)
  setStatus('Active project set')
}

document.getElementById('btnRefresh').onclick = loadProjects
document.getElementById('btnNewProject').onclick = async () => {
  const name = prompt('Project name')
  if (!name) return
  const description = prompt('Description (optional)') || ''
  await store.createProject(name, description)
  await loadProjects()
}
document.getElementById('btnGenerate').onclick = generateContext
document.getElementById('btnCopy').onclick = async () => {
  if (elContext.value) {
    await navigator.clipboard.writeText(elContext.value)
    setStatus('Context pack copied')
  }
}
document.getElementById('btnSetActive').onclick = setActiveProject
document.getElementById('btnSearch').onclick = doSearch
document.getElementById('btnRefreshSummary').onclick = loadSummaries
document.getElementById('btnLoadMore').onclick = () => loadMessages(true)
elSearchInput.addEventListener('keydown', (e) => {
  if (e.key === 'Enter') {
    doSearch()
  }
})

loadProjects().catch(err => {
  console.error(err)
  setStatus('Failed to load')
})
