/*
 * Plugin "Previsualizar" para ONLYOFFICE.
 *
 * Comportamiento:
 *  - Se ejecuta dentro del panel lateral (isInsideMode) del editor.
 *  - Al pulsar "Previsualizar" se abre una ventana modal (preview.html) que
 *    solicita al backend el PDF del documento actual con las etiquetas {{key}}
 *    ya resueltas, replicando la vista previa de la lista de documentos.
 *
 * Datos de sesión:
 *  - El backend inyecta `editorConfig.plugins.options[guid]` con
 *    `{ docId, token, backendUrl }`, que el SDK expone como
 *    `window.Asc.plugin.info.options`. Allí se obtiene el documento, el JWT
 *    para autorizar las llamadas a la API y la URL pública del backend.
 *
 * Arquitectura de la ventana modal:
 *  - La ventana se crea con `window.Asc.PluginWindow` y `show(variation)`.
 *  - Cuando el modal termina de inicializarse envía `onPreviewReady` (mediante
 *    `sendToPlugin`), y aquí lo escuchamos con `attachEvent` para responderle
 *    con `previewWindow.command('onPreviewData', { docId, token, backendUrl })`.
 *  - El modal recibe esos datos con `window.Asc.plugin.attachEvent("onPreviewData")`
 *    y genera el PDF. (Ver preview.js).
 *
 * Buenas prácticas aplicadas:
 *  - Sin CSS propio: se reutilizan las clases de plugins.css del SDK.
 *  - `Asc.plugin.onThemeChangedBase` se delega al SDK para que adapte los colores.
 *  - `Asc.plugin.button` cierra la ventana modal (si hay id) o el plugin.
 *  - Todo el código va en un IIFE con "use strict" para no ensuciar el scope global.
 */
;(function (window, undefined) {
  'use strict'

  window.Asc = window.Asc || {}
  window.Asc.plugin = window.Asc.plugin || {}
  // El GUID debe coincidir con "guid" del config.json.
  window.Asc.plugin.guid = 'asc.{8f2a1c40-7b3d-4e21-9a6f-000000000004}'

  var state = {
    docId: '',
    token: '',
    backendUrl: '',
    key: ''
  }
  var previewWindow = null

  // ---- Sesión ----

  // Lee los datos inyectados por el backend desde `Asc.plugin.info.options`.
  function readOptions() {
    var info = window.Asc && window.Asc.plugin ? window.Asc.plugin.info : null
    var opts = (info && info.options) || {}
    state.docId = opts.docId || ''
    state.token = opts.token || ''
    state.backendUrl = (opts.backendUrl || '').replace(/\/$/, '')
    state.key = opts.key || ''
  }

  function closeSidebar() {
    window.Asc.plugin.executeCommand('close', '')
  }

  // ---- Ventana modal de previsualización ----

  // Envía los datos de sesión al modal cuando este avisa que está listo.
  function sendPreviewData() {
    previewWindow.command('onPreviewData', {
      docId: state.docId,
      token: state.token,
      backendUrl: state.backendUrl,
      key: state.key
    })
  }

  function openPreviewWindow() {
    if (!state.docId) {
      showStatus(window.Asc.plugin.tr('Could not get the document identifier.'), true)
      return
    }

    var location = window.location
    var start = location.pathname.lastIndexOf('/') + 1
    var file = location.pathname.substring(start)

    var variation = {
      url: location.href.replace(file, 'preview.html'),
      description: window.Asc.plugin.tr('Document preview'),
      isVisual: true,
      isModal: true,
      isViewer: true,
      EditorsSupport: ['word', 'cell'],
      buttons: [{ text: 'Close', primary: false }],
      size: [900, 640]
    }

    if (!previewWindow) {
      previewWindow = new window.Asc.PluginWindow()
      // Se ejecuta cuando el modal envía "onPreviewReady" tras inicializarse.
      previewWindow.attachEvent('onPreviewReady', sendPreviewData)
    }
    previewWindow.show(variation)
  }

  function showStatus(message, isError) {
    var status = document.getElementById('preview-status')
    if (status) {
      status.textContent = message || ''
      status.className = 'preview-status' + (isError ? ' error' : '')
    }
  }

  // ---- Inicialización ----

  // Aplica las traducciones cargadas por el editor (langs.json / es-ES.json).
  window.Asc.plugin.onTranslate = function () {
    var label = document.getElementById('label-title')
    if (label) label.innerHTML = window.Asc.plugin.tr('Preview')
    var hint = document.getElementById('preview-hint')
    if (hint) hint.innerHTML = window.Asc.plugin.tr('Generates a PDF of the current document with the {{key}} tags already resolved.')
    var btnText = document.getElementById('btn-text')
    if (btnText) btnText.innerHTML = window.Asc.plugin.tr('Generate preview')
    var close = document.getElementById('btn-cerrar')
    if (close) close.innerHTML = window.Asc.plugin.tr('Close')
  }

  window.Asc.plugin.init = function () {
    readOptions()
    var button = document.getElementById('btn-previsualizar')
    if (button) {
      button.addEventListener('click', openPreviewWindow)
    }
    var closeButton = document.getElementById('btn-cerrar')
    if (closeButton) {
      closeButton.addEventListener('click', closeSidebar)
    }
  }

  // Permite que el SDK adapte los colores al cambiar el tema del editor.
  window.Asc.plugin.onThemeChanged = function (theme) {
    window.Asc.plugin.onThemeChangedBase(theme)
  }

  // Botones del modal ("Cerrar"): cierra la ventana o el plugin según el contexto.
  window.Asc.plugin.button = function (id, windowId) {
    if (windowId) {
      this.executeMethod('CloseWindow', [windowId])
    } else {
      this.executeCommand('close', '')
    }
  }
})(window, undefined)
