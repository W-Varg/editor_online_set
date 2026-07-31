;(function (window, undefined) {
  'use strict'

  window.Asc.plugin.init = function () {}

  window.Asc.plugin.onThemeChanged = function (theme) {
    window.Asc.plugin.onThemeChangedBase(theme)
  }

  window.Asc.plugin.button = function (id) {
    this.executeCommand('close', '')
  }

  window.Asc.plugin.attachEvent('onSaludoData', function (data) {
    document.getElementById('saludo-container').innerHTML = '¡Hola! Dijiste: <strong>' + (data.texto || '(vacío)') + '</strong>'
  })

  function autoInit() {
    if (!window.Asc.plugin._initInternal) {
      window.Asc.plugin._initInternal = true
      window.parent.postMessage(JSON.stringify({
        type: 'initialize',
        guid: window.Asc.plugin.guid || 'asc.{8f2a1c40-7b3d-4e21-9a6f-000000000002}'
      }), '*')
    }
  }

  function onMessage(e) {
    var d
    try { d = JSON.parse(e.data) } catch (_) { return }
    if (d && d.type === 'plugin_init' && d.data) {
      eval(d.data)
    }
  }

  window.addEventListener('message', onMessage, false)
  if (document.readyState === 'complete') autoInit()
  else window.addEventListener('load', autoInit)
})(window, undefined)
