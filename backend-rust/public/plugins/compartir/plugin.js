;(function (window, undefined) {
  'use strict'

  var state = {
    docId: '',
    token: '',
    backendUrl: '',
    query: '',
    users: [],
    initialShared: Object.create(null),
    selection: Object.create(null),
    loading: false
  }
  var root
  var refs = {}

  function pluginData() {
    var info = window.Asc && window.Asc.plugin && window.Asc.plugin.info
    var data = info && info.pluginData ? info.pluginData : []
    state.docId = data[0] || ''
    state.token = data[1] || ''
    state.backendUrl = (data[2] || '').replace(/\/$/, '')
  }

  function el(tag, className, text) {
    var node = document.createElement(tag)
    if (className) node.className = className
    if (text !== undefined) node.textContent = text
    return node
  }

  function api(path, options) {
    options = options || {}
    options.headers = options.headers || {}
    options.headers.Authorization = 'Bearer ' + state.token
    if (options.body && typeof options.body !== 'string') {
      options.headers['Content-Type'] = 'application/json'
      options.body = JSON.stringify(options.body)
    }
    return fetch(state.backendUrl + path, options).then(function (response) {
      return response.text().then(function (text) {
        var body = text ? JSON.parse(text) : null
        if (!response.ok) throw new Error((body && body.message) || text || 'Error HTTP ' + response.status)
        return body
      })
    })
  }

  function render() {
    refs.results.innerHTML = ''
    if (state.loading) {
      refs.results.appendChild(el('div', 'state', 'Buscando usuarios...'))
      return
    }
    if (!state.users.length) {
      refs.results.appendChild(el('div', 'state', state.query ? 'No se encontraron usuarios.' : 'Escriba un DNI o nombre para buscar.'))
      return
    }

    state.users.forEach(function (user) {
      var row = el('label', 'user-row')
      var checkbox = document.createElement('input')
      checkbox.type = 'checkbox'
      checkbox.checked = state.selection[user.id] === true
      checkbox.addEventListener('change', function () {
        state.selection[user.id] = checkbox.checked
        row.classList.toggle('selected', checkbox.checked)
      })

      var details = el('span', 'user-details')
      details.appendChild(el('strong', '', user.name || user.username))
      var metadata = [user.dni, user.cargo].filter(Boolean).join(' | ')
      if (metadata) details.appendChild(el('small', '', metadata))
      row.appendChild(checkbox)
      row.appendChild(details)
      row.classList.toggle('selected', checkbox.checked)
      refs.results.appendChild(row)
    })
  }

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
    if (!query) {
      state.query = ''
      state.users = []
      render()
      return
    }
    state.query = query
    state.loading = true
    render()
    api('/api/documents/' + encodeURIComponent(state.docId) + '/shares/search?q=' + encodeURIComponent(query))
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

  function openConfirmation() {
    var delta = changes()
    if (!delta.add.length && !delta.remove.length) {
      showStatus('No hay cambios para guardar.', false)
      return
    }
    refs.confirmText.textContent = 'Se agregarán ' + delta.add.length + ' usuario(s) y se quitarán ' + delta.remove.length + ' usuario(s). ¿Desea continuar?'
    refs.confirm.classList.remove('hidden')
  }

  function closeConfirmation() {
    refs.confirm.classList.add('hidden')
  }

  function save() {
    var delta = changes()
    closeConfirmation()
    refs.save.disabled = true
    showStatus('Guardando cambios...', false)
    api('/api/documents/' + encodeURIComponent(state.docId) + '/shares/sync', {
      method: 'PUT',
      body: delta
    }).then(function () {
      delta.add.forEach(function (id) { state.initialShared[id] = true })
      delta.remove.forEach(function (id) {
        delete state.initialShared[id]
        state.selection[id] = false
      })
      showStatus('Permisos actualizados correctamente.', false)
      search()
    }).catch(function (error) {
      showStatus(error.message, true)
    }).then(function () {
      refs.save.disabled = false
    })
  }

  function cancel() {
    state.selection = Object.create(null)
    state.users.forEach(function (user) {
      state.selection[user.id] = state.initialShared[user.id] === true
    })
    render()
    showStatus('', false)
  }

  function showStatus(message, isError) {
    refs.status.textContent = message || ''
    refs.status.className = 'status-message' + (isError ? ' error' : '')
  }

  function setup() {
    root = document.getElementById('plugin-root') || document.body
    root.innerHTML = ''
    var panel = el('div', 'panel-container')
    panel.appendChild(el('div', 'panel-header', 'Compartir documento'))
    var searchBox = el('div', 'search-box')
    refs.search = document.createElement('input')
    refs.search.type = 'search'
    refs.search.placeholder = 'dni o nombre'
    refs.search.setAttribute('aria-label', 'Buscar por DNI o nombre')
    refs.searchButton = el('button', 'search-button')
    refs.searchButton.type = 'button'
    refs.searchButton.title = 'Buscar'
    refs.searchButton.setAttribute('aria-label', 'Buscar')
    refs.searchButton.innerHTML = '<svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="11" cy="11" r="7"></circle><path d="m16.5 16.5 4 4"></path></svg>'
    searchBox.appendChild(refs.search)
    searchBox.appendChild(refs.searchButton)
    panel.appendChild(searchBox)
    refs.results = el('div', 'results')
    panel.appendChild(refs.results)
    refs.status = el('div', 'status-message')
    panel.appendChild(refs.status)
    var footer = el('div', 'panel-footer')
    refs.cancel = el('button', 'button secondary', 'Cancelar')
    refs.save = el('button', 'button primary', 'Guardar')
    footer.appendChild(refs.cancel)
    footer.appendChild(refs.save)
    panel.appendChild(footer)
    root.appendChild(panel)

    refs.confirm = el('div', 'dialog-overlay hidden')
    var dialog = el('div', 'dialog-box')
    dialog.appendChild(el('div', 'dialog-title', 'Confirmar cambios'))
    refs.confirmText = el('p', 'dialog-text')
    dialog.appendChild(refs.confirmText)
    var dialogActions = el('div', 'dialog-actions')
    var no = el('button', 'button secondary', 'Cancelar')
    var yes = el('button', 'button primary', 'Confirmar')
    dialogActions.appendChild(no)
    dialogActions.appendChild(yes)
    dialog.appendChild(dialogActions)
    refs.confirm.appendChild(dialog)
    root.appendChild(refs.confirm)

    refs.searchButton.addEventListener('click', search)
    refs.search.addEventListener('keydown', function (event) {
      if (event.key === 'Enter') { event.preventDefault(); search() }
    })
    refs.save.addEventListener('click', openConfirmation)
    refs.cancel.addEventListener('click', cancel)
    refs.confirm.addEventListener('click', function (event) {
      if (event.target === refs.confirm) closeConfirmation()
    })
    no.addEventListener('click', closeConfirmation)
    yes.addEventListener('click', save)
    render()
  }

  window.Asc.plugin.init = function () {
    pluginData()
    setup()
  }

  window.Asc.plugin.onThemeChanged = function (theme) {
    window.Asc.plugin.onThemeChangedBase(theme)
    document.body.classList.toggle('dark-theme', theme && (theme.type === 'dark' || theme.type === 'contrast-dark'))
  }

  window.Asc.plugin.button = function () {
    this.executeCommand('close', '')
  }
})(window, undefined)
