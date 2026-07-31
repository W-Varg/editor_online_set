;(function (window, undefined) {
  'use strict'

  window.Asc = window.Asc || {}
  window.Asc.plugin = window.Asc.plugin || {}
  window.Asc.plugin.guid = 'asc.{8f2a1c40-7b3d-4e21-9a6f-000000000002}'

  var previewWindow = null

  window.Asc.plugin.init = function () {
    document.getElementById('btn-mostrar').onclick = function () {
      var location = window.location
      var start = location.pathname.lastIndexOf('/') + 1
      var file = location.pathname.substring(start)

      var variation = {
        url: location.href.replace(file, 'saludo.html'),
        description: window.Asc.plugin.tr('Saludo'),
        isVisual: true,
        isModal: true,
        isViewer: true,
        EditorsSupport: ['word', 'cell', 'slide'],
        buttons: [
          { text: 'Cerrar', primary: false }
        ],
        size: [400, 250]
      }

      if (!previewWindow) {
        previewWindow = new window.Asc.PluginWindow()
        previewWindow.attachEvent('onSaludoMessage', function () {
          var texto = document.getElementById('txt').value || '(vacío)'
          previewWindow.command('onSaludoData', { texto: texto })
        })
      }
      previewWindow.show(variation)
    }

    document.getElementById('btn-cerrar').onclick = function () {
      window.Asc.plugin.executeCommand('close', '')
    }
  }

  window.Asc.plugin.onThemeChanged = function (theme) {
    window.Asc.plugin.onThemeChangedBase(theme)
  }

  window.Asc.plugin.button = function (id) {
    this.executeCommand('close', '')
  }

  function autoInit() {
    if (!window.Asc.plugin._initInternal) {
      window.Asc.plugin._initInternal = true
      window.parent.postMessage(JSON.stringify({
        type: 'initialize',
        guid: window.Asc.plugin.guid
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
