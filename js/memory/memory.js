const store = require('memory/store.js')

const memory = {
  initialize: function () {
    if (!window.ipc) {
      return
    }

    window.ipc.on('memory-capture', function (e, payload) {
      if (!payload || !payload.content) return
      store.recordMessage(payload).catch(function (err) {
        console.warn('memory record failed', err)
      })
    })
  }
}

module.exports = memory
