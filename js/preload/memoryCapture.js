function isCopilotHost () {
  try {
    return /(^|\.)copilot\.microsoft\.com$/i.test(location.hostname)
  } catch (e) {
    return false
  }
}

function isVisible (el) {
  if (!el || !el.getBoundingClientRect) return false
  const rect = el.getBoundingClientRect()
  const style = window.getComputedStyle(el)
  if (style.display === 'none' || style.visibility === 'hidden') return false
  if (rect.width <= 0 || rect.height <= 0) return false
  return true
}

function getOrCreateSessionId () {
  const key = '__min_memory_session_id'
  let id = sessionStorage.getItem(key)
  if (!id) {
    id = 'sess_' + Date.now().toString(36) + '_' + Math.random().toString(36).slice(2, 8)
    sessionStorage.setItem(key, id)
  }
  return id
}

function sendCapture (payload) {
  try {
    ipc.send('memory-capture', payload)
  } catch (e) {}
}

function hashText (str) {
  let h = 0
  for (let i = 0; i < str.length; i++) {
    h = (h << 5) - h + str.charCodeAt(i)
    h |= 0
  }
  return String(h)
}

function initCopilotCapture () {
  if (!isCopilotHost()) return

  const seen = new Set()
  const assistantLastText = new WeakMap()
  const assistantTimers = new WeakMap()

  function record (role, content, meta) {
    const text = (content || '').trim()
    if (!text) return
    const key = role + ':' + hashText(text)
    if (seen.has(key)) return
    seen.add(key)
    sendCapture({
      role,
      content: text,
      session_id: getOrCreateSessionId(),
      created_at: Date.now(),
      source: 'copilot',
      metadata_json: meta ? JSON.stringify(meta) : null
    })
  }

  function inferRole (el) {
    const attrs =
      (el.getAttribute('data-testid') || '') +
      ' ' +
      (el.getAttribute('data-role') || '') +
      ' ' +
      (el.getAttribute('data-author') || '') +
      ' ' +
      (el.getAttribute('aria-label') || '') +
      ' ' +
      (el.className || '')
    const lower = attrs.toLowerCase()
    if (lower.includes('user') || lower.includes('me')) return 'user'
    if (lower.includes('assistant') || lower.includes('copilot') || lower.includes('bot')) return 'assistant'
    if (el.closest && el.closest('[data-testid*="user"]')) return 'user'
    if (el.closest && el.closest('[data-testid*="assistant"]')) return 'assistant'
    return ''
  }

  const messageSelectors = [
    '[data-testid*="message"]',
    '[data-testid*="chat-message"]',
    '[data-testid*="assistant"]',
    '[data-testid*="bot"]',
    'div[role="article"]',
    'article'
  ]

  function scanMessages () {
    const root = document.querySelector('main') || document.body
    if (!root) return
    const candidates = new Set()
    messageSelectors.forEach(sel => {
      root.querySelectorAll(sel).forEach(el => candidates.add(el))
    })
    candidates.forEach(el => {
      if (!isVisible(el)) return
      const text = (el.innerText || '').trim()
      if (!text || text.length < 2) return
      const role = inferRole(el)
      if (!role) return

      if (role === 'assistant') {
        const last = assistantLastText.get(el) || ''
        if (text === last) return
        assistantLastText.set(el, text)
        const t = assistantTimers.get(el)
        if (t) clearTimeout(t)
        const timer = setTimeout(function () {
          const current = (el.innerText || '').trim()
          if (current && current === text) {
            record('assistant', current, { source: 'dom-stable' })
          }
        }, 1000)
        assistantTimers.set(el, timer)
      } else {
        record('user', text, { source: 'dom' })
      }
    })
  }

  const observer = new MutationObserver(function () {
    scanMessages()
  })
  observer.observe(document.documentElement, { childList: true, subtree: true, characterData: true })

  scanMessages()
}

if (process.isMainFrame) {
  window.addEventListener('DOMContentLoaded', function () {
    initCopilotCapture()
  })
}
