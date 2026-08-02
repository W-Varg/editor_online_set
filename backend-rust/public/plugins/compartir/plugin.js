/*
 * Plugin "Compartir" para ONLYOFFICE.
 *
 * Comportamiento:
 *  - Se ejecuta dentro del panel lateral (isInsideMode) del editor.
 *  - El usuario escribe un DNI o nombre; al buscar, el backend devuelve los
 *    usuarios encontrados y los que ya tienen acceso al documento (marcados).
 *  - Con los checkboxes se seleccionan los accesos deseados (marcar = compartir,
 *    desmarcar = revocar).
 *  - "Compartir" abre una ventana modal de confirmación (confirmar.html) que
 *    resume los cambios y permite Aceptar o Rechazar (patrón de doc2md/saludar).
 *  - Al aceptar, se envía la sincronización al backend (PUT shares/sync).
 *
 * Datos de sesión:
 *  - El backend inyecta `editorConfig.plugins.options[guid]` con
 *    `{ docId, token, backendUrl }`, que el SDK expone como
 *    `window.Asc.plugin.info.options`. Allí se obtiene el JWT para autorizar
 *    las llamadas a la API y la URL pública del backend.
 *
 * Buenas prácticas:
 *  - Sin CSS propio: se reutilizan las clases de plugins.css del SDK.
 *  - `Asc.plugin.onThemeChangedBase` delega la adaptación de tema al SDK.
 *  - `Asc.plugin.button` cierra la ventana modal (si hay id) o el plugin.
 *  - Código en un IIFE con "use strict".
 */
;(function (window, undefined) {
  'use strict'

  window.Asc = window.Asc || {}
  window.Asc.plugin = window.Asc.plugin || {}
  // El GUID debe coincidir con "guid" del config.json.
  window.Asc.plugin.guid = 'asc.{8f2a1c40-7b3d-4e21-9a6f-000000000001}'

  var state = {
    docId: '',
    token: '',
    backendUrl: '',
    query: '',
    loading: false,
    users: [], // usuarios visibles en el listado
    initialShared: Object.create(null), // usuarios que ya tenían acceso
    selection: Object.create(null) // estado deseado (checked)
  }
  var refs = {}
  var confirmWindow = null
  var pendingDelta = null // { add: [ids], remove: [ids] } para la API
  var pendingSummary = null // { add: [nombres], remove: [nombres] } para el modal

  // ---- Sesión ----

  // Lee los datos inyectados por el backend desde `Asc.plugin.info.options`.
  function readOptions() {
    var info = window.Asc && window.Asc.plugin ? window.Asc.plugin.info : null
    var opts = (info && info.options) || {}
    state.docId = opts.docId || ''
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

  // Llama a la API del backend autenticada con el JWT del usuario.
  function api(path, options) {
    options = options || {}
    options.headers = options.headers || {}
    if (state.token) {
      options.headers.Authorization = 'Bearer ' + state.token
    }
    if (options.body && typeof options.body !== 'string') {
      options.headers['Content-Type'] = 'application/json'
      options.body = JSON.stringify(options.body)
    }
    return fetch(state.backendUrl + path, options).then(function (response) {
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
    refs.status.style.color = isError ? '#b91c1c' : 'inherit'
  }

  // ---- Listado ----

  function render() {
    refs.results.innerHTML = ''
    if (state.loading) {
      refs.results.appendChild(el('div', '', 'Buscando...'))
      return
    }
    if (!state.users.length) {
      refs.results.appendChild(el(
        'div',
        '',
        state.query ? 'No se encontraron usuarios.' : 'Escriba un DNI o nombre para buscar.'
      ))
      return
    }
    state.users.forEach(function (user) {
      var row = el('label', '', '')
      row.style.display = 'block'
      row.style.padding = '4px 0'
      row.style.cursor = 'pointer'

      var checkbox = document.createElement('input')
      checkbox.type = 'checkbox'
      checkbox.checked = state.selection[user.id] === true
      checkbox.addEventListener('change', function () {
        state.selection[user.id] = checkbox.checked
      })

      var details = el('span', '', '')
      details.appendChild(el('strong', '', user.name || user.username))
      var meta = [user.dni, user.cargo].filter(Boolean).join(' | ')
      if (meta) {
        details.appendChild(el('small', '', ' (' + meta + ')'))
      }

      row.appendChild(checkbox)
      row.appendChild(details)
      refs.results.appendChild(row)
    })
  }

  // Combina los compartidos (pre-marcados) con los encontrados, conservando la
  // selección actual del usuario.
  function mergeUsers(compartidos, encontrados) {
    var byId = Object.create(null)
    var users = []
    function addUser(user, shared) {
      if (!user || !user.id) return
      if (shared) state.initialShared[user.id] = true
      if (state.selection[user.id] === undefined) state.selection[user.id] = shared
      if (!byId[user.id]) {
        byId[user.id] = true
        users.push(user)
      }
    }
    ;(compartidos || []).forEach(function (user) { addUser(user, true) })
    ;(encontrados || []).forEach(function (user) { addUser(user, false) })
    state.users = users
  }

  function search() {
    var query = refs.search.value.trim()
    state.query = query
    state.loading = true
    render()
    var url = '/api/documents/' + encodeURIComponent(state.docId) +
      '/shares/search?q=' + encodeURIComponent(query)
    api(url)
      .then(function (response) {
        var data = response && response.data ? response.data : {}
        mergeUsers(data.compartidos, data.encontrados)
      })
      .catch(function (error) {
        state.users = []
        showStatus(error.message, true)
      })
      .then(function () {
        state.loading = false
        render()
      })
  }

  // Calcula los cambios reales respecto al estado compartido original.
  function changes() {
    var add = []
    var remove = []
    Object.keys(state.selection).forEach(function (userId) {
      var selected = state.selection[userId] === true
      var wasShared = state.initialShared[userId] === true
      if (selected && !wasShared) add.push(userId)
      if (!selected && wasShared) remove.push(userId)
    })
    return { add: add, remove: remove }
  }

  // Convierte ids en nombres para mostrar en el modal de confirmación.
  function userNames(ids) {
    return ids.map(function (id) {
      var found = state.users.filter(function (u) { return u.id === id })[0]
      return found ? (found.name || found.username) : id
    })
  }

  // ---- Modal de confirmación (patrón de doc2md / saludar) ----

  function closeConfirmWindow() {
    if (confirmWindow) confirmWindow.close()
  }

  // Aplica los cambios al backend (PUT shares/sync) y refresca el listado.
  function applySync() {
    api('/api/documents/' + encodeURIComponent(state.docId) + '/shares/sync', {
      method: 'PUT',
      body: pendingDelta
    }).then(function () {
      pendingDelta.add.forEach(function (id) { state.initialShared[id] = true })
      pendingDelta.remove.forEach(function (id) {
        delete state.initialShared[id]
        state.selection[id] = false
      })
      closeConfirmWindow()
      showStatus('Permisos actualizados correctamente.', false)
      search()
    }).catch(function (error) {
      closeConfirmWindow()
      showStatus(error.message, true)
    })
  }

  function openConfirmation() {
    var delta = changes()
    if (!delta.add.length && !delta.remove.length) {
      showStatus('No hay cambios para guardar.', false)
      return
    }
    pendingDelta = delta
    pendingSummary = {
      add: userNames(delta.add),
      remove: userNames(delta.remove)
    }

    var location = window.location
    var start = location.pathname.lastIndexOf('/') + 1
    var file = location.pathname.substring(start)
    var variation = {
      url: location.href.replace(file, 'confirmar.html'),
      description: window.Asc.plugin.tr('Confirmar cambios'),
      isVisual: true,
      isModal: true,
      isViewer: true,
      EditorsSupport: ['word', 'cell', 'slide'],
      buttons: [],
      size: [460, 300]
    }

    if (!confirmWindow) {
      confirmWindow = new window.Asc.PluginWindow()
      // Cuando el modal está listo se le envían los cambios a confirmar.
      confirmWindow.attachEvent('onConfirmReady', function () {
        confirmWindow.command('onConfirmData', pendingSummary)
      })
      confirmWindow.attachEvent('onConfirmAccept', applySync)
      confirmWindow.attachEvent('onConfirmReject', closeConfirmWindow)
    }
    confirmWindow.show(variation)
  }

  // ---- Inicialización ----

  window.Asc.plugin.init = function () {
    readOptions()
    refs.search = document.getElementById('search-input')
    refs.results = document.getElementById('results')
    refs.status = document.getElementById('status')
    refs.searchButton = document.getElementById('btn-buscar')
    refs.shareButton = document.getElementById('btn-compartir')
    refs.closeButton = document.getElementById('btn-cerrar')

    refs.searchButton.addEventListener('click', search)
    refs.search.addEventListener('keydown', function (event) {
      if (event.key === 'Enter') {
        event.preventDefault()
        search()
      }
    })
    refs.shareButton.addEventListener('click', openConfirmation)
    refs.closeButton.addEventListener('click', closeSidebar)
    render()
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
