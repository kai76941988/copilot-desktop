const path = require('path')
const fs = require('fs')

if (typeof Dexie === 'undefined') {
  var Dexie = require('dexie')
}

const CHUNK_SIZE = 12
const SUMMARY_MAX_CHARS = 800

const db = new Dexie('minMemory')

db.version(1).stores({
  projects: '&project_id, updated_at, pinned',
  sessions: '&session_id, project_id, updated_at, archived',
  messages: '&id, session_id, project_id, created_at, role, tags, message_order',
  summaries: '&summary_id, project_id, session_id, summary_type, created_at',
  settings: '&key'
})

function now () {
  return Date.now()
}

function collapseWhitespace (input) {
  return (input || '').split(/\s+/).filter(Boolean).join(' ')
}

function summarizeText (input, maxChars) {
  const compact = collapseWhitespace(input)
  if (compact.length <= maxChars) return compact
  return compact.slice(0, maxChars) + '...'
}

function cleanTag (tag) {
  const trimmed = (tag || '').trim().replace(/^#/, '')
  if (!trimmed) return null
  return trimmed.toLowerCase()
}

function mergeTags (existing, incoming) {
  const set = new Set()
  ;(existing || []).forEach(t => {
    const v = cleanTag(t)
    if (v) set.add(v)
  })
  ;(incoming || []).forEach(t => {
    const v = cleanTag(t)
    if (v) set.add(v)
  })
  return Array.from(set)
}

function mergeUniqueLines (existing, incoming, maxLines) {
  const set = new Set(existing || [])
  ;(incoming || []).forEach(line => {
    if (line && !set.has(line)) {
      set.add(line)
    }
  })
  const out = Array.from(set)
  if (maxLines && out.length > maxLines) {
    return out.slice(0, maxLines)
  }
  return out
}

function autoTagFromText (text) {
  const lower = (text || '').toLowerCase()
  const tags = []
  const rules = [
    ['todo', ['todo', '待办', '下一步', '计划']],
    ['bug', ['bug', '错误', '异常', 'crash', '失败']],
    ['design', ['设计', '架构', '方案', 'design']],
    ['auth', ['登录', '认证', 'token', 'oauth']],
    ['summary', ['总结', '摘要', 'summary']],
    ['performance', ['慢', '卡', '性能', 'performance']]
  ]
  rules.forEach(([tag, keys]) => {
    if (keys.some(k => lower.includes(k))) tags.push(tag)
  })
  return tags
}

async function getSetting (key, fallback = null) {
  const item = await db.settings.get(key)
  if (item && Object.prototype.hasOwnProperty.call(item, 'value')) return item.value
  return fallback
}

async function setSetting (key, value) {
  await db.settings.put({ key, value })
}

async function ensureProject (projectId) {
  const existing = await db.projects.get(projectId)
  if (!existing) {
    await db.projects.put({
      project_id: projectId,
      name: projectId,
      description: '',
      created_at: now(),
      updated_at: now(),
      pinned: 0
    })
  } else {
    await db.projects.update(projectId, { updated_at: now() })
  }
}

async function ensureSession (sessionId, projectId) {
  const existing = await db.sessions.get(sessionId)
  if (!existing) {
    await db.sessions.put({
      session_id: sessionId,
      project_id: projectId,
      title: '',
      created_at: now(),
      updated_at: now(),
      source_url: '',
      archived: 0
    })
  } else {
    await db.sessions.update(sessionId, { updated_at: now() })
  }
}

async function getMessageCount (sessionId) {
  return db.messages.where('session_id').equals(sessionId).count()
}

async function fetchRecentMessages (sessionId, limit) {
  const items = await db.messages
    .where('session_id')
    .equals(sessionId)
    .reverse()
    .sortBy('created_at')
  const selected = items.slice(-limit)
  return selected
}

async function getLastChunkEnd (sessionId) {
  const last = await db.summaries
    .where({ session_id: sessionId, summary_type: 'chunk' })
    .reverse()
    .sortBy('created_at')
  if (last && last.length) return last[0].source_range_end
  return null
}

async function buildChunkSummary (sessionId) {
  const items = await fetchRecentMessages(sessionId, CHUNK_SIZE)
  if (items.length < CHUNK_SIZE) return null
  const lines = items.map(item => `${item.role === 'user' ? 'U' : 'A'}: ${item.content}`)
  const summary = summarizeText(lines.join('\n'), SUMMARY_MAX_CHARS)
  return {
    range_start: items[0].message_order || items[0].created_at,
    range_end: items[items.length - 1].message_order || items[items.length - 1].created_at,
    content: summary
  }
}

async function buildSessionSummary (sessionId) {
  const chunks = await db.summaries
    .where({ session_id: sessionId, summary_type: 'chunk' })
    .reverse()
    .sortBy('created_at')
  if (chunks.length) {
    const recent = chunks.slice(0, 3).reverse().map(c => c.content).join('\n')
    return summarizeText(recent, SUMMARY_MAX_CHARS)
  }
  const items = await fetchRecentMessages(sessionId, CHUNK_SIZE)
  return summarizeText(items.map(i => `${i.role}: ${i.content}`).join('\n'), SUMMARY_MAX_CHARS)
}

function extractLinesByKeywords (lines, keywords, maxLines) {
  const out = []
  lines.forEach(line => {
    const lower = line.toLowerCase()
    if (keywords.some(k => lower.includes(k))) {
      out.push(line)
      if (out.length >= maxLines) return
    }
  })
  return out
}

async function buildProjectStructSummary (projectId, sessionId) {
  let existing = await db.summaries
    .where({ project_id: projectId, summary_type: 'project_struct' })
    .reverse()
    .sortBy('created_at')
  existing = existing.length ? existing[0] : null

  let summary = existing ? JSON.parse(existing.content || '{}') : {}
  summary.background = summary.background || []
  summary.key_facts = summary.key_facts || []
  summary.progress = summary.progress || []
  summary.todo = summary.todo || []
  summary.constraints = summary.constraints || []

  const recent = await fetchRecentMessages(sessionId, 20)
  const lines = recent.map(i => `${i.role}: ${collapseWhitespace(i.content)}`)

  summary.todo = mergeUniqueLines(summary.todo, extractLinesByKeywords(lines, ['todo', '待办', '下一步', '计划', 'next'], 6), 8)
  summary.constraints = mergeUniqueLines(summary.constraints, extractLinesByKeywords(lines, ['必须', '不要', '禁止', 'constraint', '限制', 'must'], 6), 8)
  summary.key_facts = mergeUniqueLines(summary.key_facts, extractLinesByKeywords(lines, ['结论', '决定', '确认', 'final', '关键'], 6), 8)
  summary.progress = mergeUniqueLines(summary.progress, extractLinesByKeywords(lines, ['进展', '完成', 'done', 'working'], 6), 8)

  if (!summary.background.length) {
    const firstUser = recent.find(i => i.role === 'user')
    if (firstUser) {
      summary.background = [summarizeText(firstUser.content, 200)]
    }
  }

  summary.updated_at = now()
  return summary
}

function projectStructToText (summary) {
  const join = (arr) => (arr && arr.length ? arr.join('\n') : '暂无')
  return [
    '【项目背景】',
    join(summary.background),
    '',
    '【关键结论】',
    join(summary.key_facts),
    '',
    '【最近进展】',
    join(summary.progress),
    '',
    '【当前待办】',
    join(summary.todo),
    '',
    '【必须遵守的约束】',
    join(summary.constraints)
  ].join('\n')
}

async function upsertSummary (projectId, sessionId, summaryType, content, rangeStart, rangeEnd) {
  if (summaryType !== 'chunk') {
    await db.summaries
      .where({ project_id: projectId, session_id: sessionId, summary_type: summaryType })
      .delete()
  }
  const summaryId = crypto.randomUUID()
  await db.summaries.put({
    summary_id: summaryId,
    project_id: projectId,
    session_id: sessionId,
    summary_type: summaryType,
    source_range_start: rangeStart || null,
    source_range_end: rangeEnd || null,
    content,
    created_at: now()
  })
}

async function maybeUpdateSummaries (projectId, sessionId) {
  const count = await getMessageCount(sessionId)
  if (!count) return
  if (count % CHUNK_SIZE === 0) {
    const lastEnd = await getLastChunkEnd(sessionId)
    const chunk = await buildChunkSummary(sessionId)
    if (chunk && chunk.range_end !== lastEnd) {
      await upsertSummary(projectId, sessionId, 'chunk', chunk.content, chunk.range_start, chunk.range_end)
    }
  }
  if (count % CHUNK_SIZE === 0 || count === 1) {
    const sessionSummary = await buildSessionSummary(sessionId)
    await upsertSummary(projectId, sessionId, 'session', sessionSummary, null, null)

    const struct = await buildProjectStructSummary(projectId, sessionId)
    await upsertSummary(projectId, null, 'project_struct', JSON.stringify(struct), null, null)
    await upsertSummary(projectId, null, 'project', projectStructToText(struct), null, null)
  }
}

async function recordMessage (payload) {
  const role = payload.role
  const content = payload.content || ''
  if (!role || !content) return null
  const ts = payload.created_at || now()
  const projectId = payload.project_id || (await getSetting('activeProjectId', 'default'))
  const sessionId = payload.session_id || `session_${ts}`
  await ensureProject(projectId)
  await ensureSession(sessionId, projectId)

  const tags = mergeTags(payload.tags || [], autoTagFromText(content))

  const message = {
    id: payload.id || crypto.randomUUID(),
    session_id: sessionId,
    project_id: projectId,
    role,
    content,
    created_at: ts,
    source: payload.source || 'copilot',
    message_order: payload.message_order || ts,
    summary_group_id: payload.summary_group_id || null,
    tags,
    metadata_json: payload.metadata_json || null
  }
  await db.messages.put(message)
  await maybeUpdateSummaries(projectId, sessionId)
  return message
}

async function listProjects () {
  return db.projects.orderBy('updated_at').reverse().toArray()
}

async function createProject (name, description) {
  const id = crypto.randomUUID()
  await db.projects.put({
    project_id: id,
    name,
    description: description || '',
    created_at: now(),
    updated_at: now(),
    pinned: 0
  })
  return id
}

async function listSessions (projectId) {
  return db.sessions.where('project_id').equals(projectId).reverse().sortBy('updated_at')
}

async function listSummaries (projectId, sessionId, summaryType, limit) {
  let coll = db.summaries.where('project_id').equals(projectId)
  if (sessionId !== undefined && sessionId !== null) {
    coll = coll.and(s => s.session_id === sessionId)
  }
  if (summaryType) {
    coll = coll.and(s => s.summary_type === summaryType)
  }
  const items = await coll.reverse().sortBy('created_at')
  return items.slice(0, limit || 50)
}

async function listMessages (sessionId, limit, offset) {
  const items = await db.messages
    .where('session_id')
    .equals(sessionId)
    .reverse()
    .sortBy('created_at')
  const start = offset || 0
  const end = start + (limit || 50)
  return items.slice(start, end).reverse()
}

async function searchMessages (query, projectId, limit) {
  const text = query.toLowerCase()
  const items = await db.messages.toArray()
  const filtered = items.filter(m => (!projectId || m.project_id === projectId) && m.content.toLowerCase().includes(text))
  filtered.sort((a, b) => b.created_at - a.created_at)
  return filtered.slice(0, limit || 50)
}

async function searchSummaries (query, projectId, limit) {
  const text = query.toLowerCase()
  const items = await db.summaries.toArray()
  const filtered = items.filter(s => (!projectId || s.project_id === projectId) && (s.content || '').toLowerCase().includes(text))
  filtered.sort((a, b) => b.created_at - a.created_at)
  return filtered.slice(0, limit || 50)
}

async function getContextPack (projectId, sessionId) {
  const projectSummary = await db.summaries.where({ project_id: projectId, summary_type: 'project' }).reverse().sortBy('created_at')
  const projectStruct = await db.summaries.where({ project_id: projectId, summary_type: 'project_struct' }).reverse().sortBy('created_at')
  const sessionSummary = sessionId
    ? await db.summaries.where({ project_id: projectId, session_id: sessionId, summary_type: 'session' }).reverse().sortBy('created_at')
    : []

  let struct = null
  if (projectStruct.length) {
    try { struct = JSON.parse(projectStruct[0].content) } catch (e) {}
  }

  const recentMessages = sessionId
    ? await fetchRecentMessages(sessionId, 8)
    : await db.messages.where('project_id').equals(projectId).reverse().sortBy('created_at').then(items => items.slice(0, 8).reverse())

  const recentText = recentMessages.map(m => `${m.role}: ${m.content}`).join('\n')
  if (struct) {
    return buildContextFromStruct(struct, sessionSummary[0]?.content, recentText)
  }

  const projectText = projectSummary[0]?.content || '暂无'
  const sessionText = sessionSummary[0]?.content || '暂无'
  return buildContextText(projectText, sessionText, recentText)
}

function buildContextText (project, session, recent) {
  return [
    '【项目背景】',
    project || '暂无',
    '',
    '【最近进展】',
    session || '暂无',
    '',
    '【近期对话要点】',
    recent || '暂无',
    '',
    '【请你基于以上上下文继续】'
  ].join('\n')
}

function buildContextFromStruct (struct, sessionSummary, recent) {
  const join = (arr) => (arr && arr.length ? arr.join('\n') : '暂无')
  return [
    '【项目背景】',
    join(struct.background),
    '',
    '【已确认结论】',
    join(struct.key_facts),
    '',
    '【当前目标/待办】',
    join(struct.todo),
    '',
    '【最近进展】',
    sessionSummary || join(struct.progress),
    '',
    '【近期对话要点】',
    recent || '暂无',
    '',
    '【必须遵守的约束】',
    join(struct.constraints),
    '',
    '【继续要求】',
    '1. 请基于以上上下文继续。',
    '2. 如果有冲突，请优先遵守约束。',
    '3. 输出清晰步骤或建议。'
  ].join('\n')
}

async function setActiveProject (projectId) {
  await setSetting('activeProjectId', projectId)
}

async function getActiveProject () {
  return getSetting('activeProjectId', 'default')
}

async function listTags () {
  const items = await db.messages.toArray()
  const map = {}
  items.forEach(m => {
    ;(m.tags || []).forEach(tag => {
      map[tag] = (map[tag] || 0) + 1
    })
  })
  return Object.keys(map).map(tag => ({ tag, count: map[tag] })).sort((a, b) => b.count - a.count)
}

async function exportProject (projectId, exportPath) {
  const items = await db.messages.where('project_id').equals(projectId).sortBy('created_at')
  const outPath = exportPath || path.join(process.cwd(), `copilot_memory_${projectId}_${now()}.jsonl`)
  const stream = fs.createWriteStream(outPath, { encoding: 'utf-8' })
  items.forEach(item => {
    stream.write(JSON.stringify(item) + '\n')
  })
  stream.end()
  return { export_path: outPath, message_count: items.length }
}

module.exports = {
  db,
  recordMessage,
  listProjects,
  createProject,
  listSessions,
  listSummaries,
  listMessages,
  searchMessages,
  searchSummaries,
  getContextPack,
  setActiveProject,
  getActiveProject,
  listTags,
  exportProject
}
