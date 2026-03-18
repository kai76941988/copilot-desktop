/* send bookmarks data.  */

function isVisible (el) {
  // https://github.com/jquery/jquery/blob/305f193aa57014dc7d8fa0739a3fefd47166cd44/src/css/hiddenVisibleSelectors.js
  return el.offsetWidth || el.offsetHeight || (el.getClientRects && el.getClientRects().length)
}

var perfSettings = {
  lowMemoryMode: (typeof navigator.deviceMemory === 'number' && navigator.deviceMemory <= 4)
}

function updatePerfSettings (data) {
  if (data && typeof data === 'object' && typeof data.lowMemoryMode === 'boolean') {
    perfSettings.lowMemoryMode = data.lowMemoryMode
  }
}

try {
  ipc.on('perfSettings', function (e, data) {
    updatePerfSettings(data)
  })
  ipc.send('getPerfSettings')
} catch (e) {}

function getExtractionLimits () {
  if (perfSettings.lowMemoryMode) {
    return {
      maxTextLength: 80000,
      maxNodes: 8000,
      includeFrames: false,
      delayMs: 1500
    }
  }

  return {
    maxTextLength: 300000,
    maxNodes: 30000,
    includeFrames: true,
    delayMs: 500
  }
}

function extractPageText (doc, win, limits) {
  var maybeNodes = [].slice.call(doc.body.childNodes)
  var textNodes = []

  var ignore = 'link, style, script, noscript, .hidden, .visually-hidden, .visuallyhidden, [role=presentation], [hidden], [style*="display:none"], [style*="display: none"], .ad, .dialog, .modal, select, svg, details:not([open]), header, nav, footer'

  while (maybeNodes.length) {
    var node = maybeNodes.shift()

    // if the node should be ignored, skip it and all of it's child nodes
    if (node.matches && node.matches(ignore)) {
      continue
    }

    // if the node is a text node, add it to the list of text nodes

    if (node.nodeType === 3) {
      textNodes.push(node)
      if (limits.maxNodes && textNodes.length >= limits.maxNodes) {
        break
      }
      continue
    }

    if (!isVisible(node)) {
      continue
    }

    // otherwise, add the node's text nodes to the list of text, and the other child nodes to the list of nodes to check
    var childNodes = node.childNodes
    var cnl = childNodes.length

    for (var i = cnl - 1; i >= 0; i--) {
      var childNode = childNodes[i]
      maybeNodes.unshift(childNode)
    }

    if (limits.maxNodes && textNodes.length >= limits.maxNodes) {
      break
    }
  }

  var text = ''

  var tnl = textNodes.length

  // combine the text of all of the accepted text nodes together
  for (var i = 0; i < tnl; i++) {
    text += textNodes[i].textContent + ' '
    if (limits.maxTextLength && text.length >= limits.maxTextLength) {
      text = text.substring(0, limits.maxTextLength)
      break
    }
  }

  // special meta tags

  var mt = doc.head.querySelector('meta[name=description]')

  if (mt) {
    text += ' ' + mt.content
  }

  text = text.trim()

  text = text.replace(/[\n\t]/g, ' ') // remove useless newlines/tabs that increase filesize

  text = text.replace(/\s{2,}/g, ' ') // collapse multiple spaces into one
  return text
}

function getPageData (cb) {
  requestAnimationFrame(function () {
    var limits = getExtractionLimits()
    var text = extractPageText(document, window, limits)

    // try to also extract text for same-origin iframes (such as the reader mode frame)
    if (limits.includeFrames) {
      var frames = document.querySelectorAll('iframe')

      for (var x = 0; x < frames.length; x++) {
        try {
          text += '. ' + extractPageText(frames[x].contentDocument, frames[x].contentWindow, limits)
        } catch (e) {}
      }
    }

    // limit the amount of text that is collected
    if (limits.maxTextLength) {
      text = text.substring(0, limits.maxTextLength)
    }

    cb({
      extractedText: text
    })
  })
}

// send the data when the page loads
if (process.isMainFrame) {
  window.addEventListener('load', function (e) {
    var limits = getExtractionLimits()
    setTimeout(function () {
      getPageData(function (data) {
        ipc.send('pageData', data)
      })
    }, limits.delayMs)
  })

  setTimeout(function () {
    // https://stackoverflow.com/a/52809105
    electron.webFrame.executeJavaScript(`
      history.pushState = ( f => function pushState(){
        var ret = f.apply(this, arguments);
        window.postMessage('_minInternalLocationChange', '*')
        return ret;
    })(history.pushState);
    
    history.replaceState = ( f => function replaceState(){
        var ret = f.apply(this, arguments);
        window.postMessage('_minInternalLocationReplacement', '*')
        return ret;
    })(history.replaceState);
  `)
  }, 0)

  window.addEventListener('message', function (e) {
    if (e.data === '_minInternalLocationChange' || e.data === '_minInternalLocationReplacement') {
      var limits = getExtractionLimits()
      setTimeout(function () {
        getPageData(function (data) {
          ipc.send('pageData', data)
        })
      }, limits.delayMs)
    }
  })
}
