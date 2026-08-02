/*
 * Plugin "Etiquetas" para ONLYOFFICE.
 *
 * Comportamiento:
 *  - Se ejecuta dentro del panel lateral (isInsideMode) del editor.
 *  - Al abrir el panel consulta GET /api/tags al backend y lista las etiquetas
 *    dinámicas disponibles ({{key}}).
 *  - Al hacer clic en una etiqueta, se inserta su texto literal en la posición
 *    del cursor del documento (executeMethod "PasteText").
 *  - Las etiquetas quedan guardadas sin resolver en el archivo; el backend las
 *    sustituye por sus valores reales al previsualizar o convertir a PDF.
 *
 * Datos de sesión:
 *  - El backend inyecta `editorConfig.plugins.options[guid]` con
 *    `{ docId, token, backendUrl }`, que el SDK expone como
 *    `window.Asc.plugin.info.options`. Allí se obtiene el JWT para autorizar
 *    las llamadas a la API y la URL pública del backend.
 *
 * Buenas prácticas:
 *  - Estilos en resources/styles/style.css (cargado después de plugins.css).
 *  - `Asc.plugin.onThemeChangedBase` delega la adaptación de tema al SDK.
 *  - `Asc.plugin.button` cierra el plugin (sin ventana modal).
 *  - Lista estática embebida como respaldo si la API no responde.
 *  - Código en un IIFE con "use strict".
 */
;(function (window, undefined) {
  'use strict'

  window.Asc = window.Asc || {}
  window.Asc.plugin = window.Asc.plugin || {}
  // El GUID debe coincidir con "guid" del config.json.
  window.Asc.plugin.guid = 'asc.{8f2a1c40-7b3d-4e21-9a6f-000000000003}'

  // Respaldo estático por si el backend no está disponible al abrir el panel.
  var FALLBACK_TAGS = [
    { key: 'fecha_actual', label: 'Fecha actual', description: 'Fecha del día en que se previsualiza.' },
    { key: 'nombre_usuario', label: 'Nombre del usuario', description: 'Nombre del usuario que previsualiza.' },
    { key: 'cargo_usuario', label: 'Cargo del usuario', description: 'Cargo del usuario que previsualiza.' },
    { key: 'dni', label: 'DNI del usuario', description: 'DNI del usuario que previsualiza.' },
    { key: 'email', label: 'Email del usuario', description: 'Email del usuario que previsualiza.' }
  ]

  var state = {
    token: '',
    backendUrl: '',
    loading: false,
    tags: []
  }
  var refs = {}

  // ---- Sesión ----

  // Lee los datos inyectados por el backend desde `Asc.plugin.info.options`.
  function readOptions() {
    var info = window.Asc && window.Asc.plugin ? window.Asc.plugin.info : null
    var opts = (info && info.options) || {}
    state.token = opts.token || ''
    state.backendUrl = (opts.backendUrl || '').replace(/\/$/, '')
  }

  // ---- Utilidades DOM / API ----

  function el(tag, className, text) {
    var node = document.createElement(tag)
    if (className) node.className = className
    if (text !== undefined) node.textContent = text
    return node
  }

  function api(path) {
    var headers = {}
    if (state.token) {
      headers.Authorization = 'Bearer ' + state.token
    }
    return fetch(state.backendUrl + path, { headers: headers }).then(function (response) {
      return response.text().then(function (text) {
        var body = text ? JSON.parse(text) : null
        if (!response.ok) {
          throw new Error((body && body.message) || text || 'Error HTTP ' + response.status)
        }
        return body
      })
    })
  }

  function closeSidebar() {
    window.Asc.plugin.executeCommand('close', '')
  }

  function showStatus(message, isError) {
    refs.status.textContent = message || ''
    refs.status.className = 'share-status' + (isError ? ' error' : '')
  }

  // ---- Listado ----

  function render() {
    refs.results.innerHTML = ''
    if (state.loading) {
      refs.results.appendChild(el('div', 'share-empty', 'Cargando...'))
      return
    }
    if (!state.tags.length) {
      refs.results.appendChild(el('div', 'share-empty', 'No hay etiquetas disponibles.'))
      return
    }
    state.tags.forEach(function (tag) {
      var row = el('div', 'share-row', '')
      row.addEventListener('click', function () { insertTag(tag) })

      var details = el('span', 'share-row-text', '')
      details.appendChild(el('span', 'share-row-name', '{{' + tag.key + '}}'))
      if (tag.description) {
        details.appendChild(el('small', 'share-row-meta', ' — ' + tag.label))
      }

      row.appendChild(details)
      refs.results.appendChild(row)
    })
  }

  function loadTags() {
    state.loading = true
    render()
    api('/api/tags')
      .then(function (list) {
        state.tags = Array.isArray(list) ? list : FALLBACK_TAGS
      })
      .catch(function (error) {
        state.tags = FALLBACK_TAGS
        showStatus('Usando lista local: ' + error.message, true)
      })
      .then(function () {
        state.loading = false
        render()
      })
  }

  // ---- Inserción en el documento ----

  // Inserta el texto literal de la etiqueta en la posición del cursor.
  // "PasteText" funciona en los editores de texto y hoja de cálculo; si no
  // existe, se usa "InsertText" (Word).
  function insertTag(tag) {
    var text = '{{' + tag.key + '}}'
    var plugin = window.Asc.plugin

    function fallback() {
      plugin.executeMethod('InsertText', [text])
    }

    if (typeof plugin.executeMethod === 'function') {
      plugin.executeMethod('PasteText', [text], fallback)
    } else {
      fallback()
    }
  }

  // ---- Inicialización ----

  window.Asc.plugin.init = function () {
    readOptions()
    refs.results = document.getElementById('results')
    refs.status = document.getElementById('status')
    refs.closeButton = document.getElementById('btn-cerrar')

    refs.closeButton.addEventListener('click', closeSidebar)
    loadTags()
  }

  window.Asc.plugin.onThemeChanged = function (theme) {
    window.Asc.plugin.onThemeChangedBase(theme)
  }

  window.Asc.plugin.button = function (id, windowId) {
    if (windowId) {
      this.executeMethod('CloseWindow', [windowId])
    } else {
      this.executeCommand('close', '')
    }
  }
})(window, undefined)
