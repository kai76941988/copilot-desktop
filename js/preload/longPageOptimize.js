(function () {
  if (!process.isMainFrame) {
    return
  }

  var perfSettings = {
    lowMemoryMode: false,
    longArticleMode: true
  }

  function isLowMemoryDevice () {
    return typeof navigator.deviceMemory === 'number' && navigator.deviceMemory <= 4
  }

  function updatePerfSettings (data) {
    if (data && typeof data === 'object') {
      if (typeof data.lowMemoryMode === 'boolean') {
        perfSettings.lowMemoryMode = data.lowMemoryMode
      }
      if (typeof data.longArticleMode === 'boolean') {
        perfSettings.longArticleMode = data.longArticleMode
      }
      return
    }

    perfSettings.lowMemoryMode = isLowMemoryDevice()
    perfSettings.longArticleMode = true
  }

  function shouldOptimize () {
    if (!perfSettings.longArticleMode) return false
    if (!document || !document.body) return false

    var scrollHeight = Math.max(document.documentElement.scrollHeight || 0, document.body.scrollHeight || 0)
    var textLength = (document.body.innerText || '').length

    return scrollHeight > 6000 || textLength > 20000
  }

  function applyLongPageStyles () {
    if (!shouldOptimize()) return

    if (document.documentElement.dataset.minLongPageOptimized === '1') return
    document.documentElement.dataset.minLongPageOptimized = '1'

    var style = document.createElement('style')
    style.id = 'min-long-article-opt'
    style.textContent = `
      article, main, #content, .content, .post, .article, .entry-content, .post-content, .article-content {
        content-visibility: auto;
        contain-intrinsic-size: 1000px 2000px;
      }
      body > * {
        content-visibility: auto;
        contain-intrinsic-size: 800px 1200px;
      }
    `
    document.documentElement.appendChild(style)

    if (perfSettings.lowMemoryMode) {
      var images = document.querySelectorAll('img:not([loading])')
      images.forEach(function (img) {
        try {
          img.loading = 'lazy'
          img.decoding = 'async'
        } catch (e) {}
      })
    }
  }

  function scheduleApply () {
    setTimeout(applyLongPageStyles, 600)
  }

  try {
    ipc.on('perfSettings', function (e, data) {
      updatePerfSettings(data)
      scheduleApply()
    })
    ipc.send('getPerfSettings')
  } catch (e) {}

  window.addEventListener('load', scheduleApply)

  window.addEventListener('message', function (e) {
    if (e && (e.data === '_minInternalLocationChange' || e.data === '_minInternalLocationReplacement')) {
      scheduleApply()
    }
  })
})()
