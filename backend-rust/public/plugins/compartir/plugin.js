/*
 * Plugin "Compartir" para ONLYOFFICE.
 *
 * Comportamiento:
 *  - Se ejecuta dentro del panel lateral (isInsideMode) del editor.
 *  - Al abrir el panel se listan únicamente los usuarios con los que YA se
 *    compartió el documento (marcados), para saber de entrada con quién se ha
 *    compartido.
 *  - Se puede desmarcar a alguien (revocar acceso) o buscar por DNI/nombre para
 *    marcarlo (otorgar acceso).
 *  - "Guardar" abre una ventana modal de confirmación (confirmar.html) que
 *    resume los cambios y permite Aceptar o Rechazar.
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
 *  - La ventana modal se crea FRESCA en cada "Guardar" (una única instancia
 *    reutilizada no re-dispara `onConfirmReady` ni los botones).
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
    refs.status.className = 'share-status' + (isError ? ' error' : '')
  }

  // ---- Listado ----

  function render() {
    refs.results.innerHTML = ''
    if (state.loading) {
      refs.results.appendChild(el('div', 'share-empty', 'Cargando...'))
      return
    }
    if (!state.users.length) {
      refs.results.appendChild(el(
        'div',
        'share-empty',
        state.query
          ? 'No se encontraron usuarios.'
          : 'Aún no has compartido el documento con nadie. Busca para compartir.'
      ))
      return
    }
    state.users.forEach(function (user) {
      var row = el('label', 'share-row', '')

      var checkbox = document.createElement('input')
      checkbox.type = 'checkbox'
      checkbox.className = 'share-row-checkbox'
      checkbox.checked = state.selection[user.id] === true
      checkbox.addEventListener('change', function () {
        state.selection[user.id] = checkbox.checked
      })

      var details = el('span', 'share-row-text', '')
      details.appendChild(el('span', 'share-row-name', user.name || user.username))
      var meta = [user.dni, user.cargo].filter(Boolean).join(' | ')
      if (meta) {
        details.appendChild(el('small', 'share-row-meta', ' — ' + meta))
      }

      row.appendChild(checkbox)
      row.appendChild(details)
      refs.results.appendChild(row)
    })
  }

  // Combina compartidos (pre-marcados) y encontrados, conservando la selección
  // actual del usuario. `showShared` indica si solo se muestran los compartidos.
  function mergeUsers(compartidos, encontrados, showShared) {
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
    if (!showShared) {
      ;(encontrados || []).forEach(function (user) { addUser(user, false) })
    }
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
        mergeUsers(data.compartidos, data.encontrados, false)
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

  // Carga inicial y refresco tras guardar: muestra SOLO los usuarios ya
  // compartidos, marcados, para ver con quién se ha compartido de entrada.
  function loadShared() {
    state.loading = true
    render()
    var url = '/api/documents/' + encodeURIComponent(state.docId) + '/shares/search?q='
    api(url)
      .then(function (response) {
        var data = response && response.data ? response.data : {}
        mergeUsers(data.compartidos, data.encontrados, true)
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

  // Tras guardar: si hay una búsqueda activa se refresca la misma; si no, se
  // vuelve a mostrar solo los compartidos.
  function refresh() {
    if (state.query) {
      search()
    } else {
      loadShared()
    }
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
    confirmWindow = null
  }

  // Aplica los cambios al backend (PUT shares/sync) y refresca el listado.
  function applySync() {
    var delta = pendingDelta || { add: [], remove: [] }
    api('/api/documents/' + encodeURIComponent(state.docId) + '/shares/sync', {
      method: 'PUT',
      body: delta
    }).then(function () {
      delta.add.forEach(function (id) { state.initialShared[id] = true })
      delta.remove.forEach(function (id) {
        delete state.initialShared[id]
        state.selection[id] = false
      })
      closeConfirmWindow()
      showStatus('Permisos actualizados correctamente.', false)
      refresh()
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

    // Instancia FRESCA en cada "Guardar": reutilizar una única ventana no
    // vuelve a disparar onConfirmReady ni los botones del modal.
    closeConfirmWindow()
    confirmWindow = new window.Asc.PluginWindow()
    // Cuando el modal está listo se le envían los cambios a confirmar.
    confirmWindow.attachEvent('onConfirmReady', function () {
      confirmWindow.command('onConfirmData', pendingSummary)
    })
    confirmWindow.attachEvent('onConfirmAccept', applySync)
    confirmWindow.attachEvent('onConfirmReject', closeConfirmWindow)
    confirmWindow.show(variation)
  }

  // ---- Inicialización ----

  window.Asc.plugin.init = function () {
    readOptions()
    refs.search = document.getElementById('search-input')
    refs.results = document.getElementById('results')
    refs.status = document.getElementById('status')
    refs.searchButton = document.getElementById('btn-buscar')
    refs.saveButton = document.getElementById('btn-guardar')
    refs.closeButton = document.getElementById('btn-cerrar')

    refs.searchButton.addEventListener('click', search)
    refs.search.addEventListener('keydown', function (event) {
      if (event.key === 'Enter') {
        event.preventDefault()
        search()
      }
    })
    refs.saveButton.addEventListener('click', openConfirmation)
    refs.closeButton.addEventListener('click', closeSidebar)
    loadShared()
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
