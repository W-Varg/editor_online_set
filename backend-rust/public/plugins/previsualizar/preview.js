/*
 * Script de la ventana modal del plugin "Previsualizar".
 *
 * Ciclo de vida (complementa a plugin.js):
 *  - Al terminar de inicializarse envía `onPreviewReady` al plugin padre mediante
 *    `sendToPlugin`, indicando que ya puede recibir los datos de sesión.
 *  - El plugin padre responde con `previewWindow.command('onPreviewData', ...)`
 *    y aquí se escucha con `attachEvent("onPreviewData", ...)`.
 *  - Con esos datos se solicita al backend el PDF del documento actual con las
 *    etiquetas {{key}} resueltas (GET /api/documents/{id}/preview) y se muestra
 *    en un iframe mediante un object URL.
 *
 * Buenas prácticas: mismo enfoque sin CSS propio, delegación de tema en
 * `onThemeChangedBase`, y código contenido en un IIFE con "use strict".
 */
;(function (window, undefined) {
  'use strict'

  window.Asc = window.Asc || {}
  window.Asc.plugin = window.Asc.plugin || {}

  function showLoading() {
    document.getElementById('preview-loading').style.display = 'flex'
    document.getElementById('preview-frame').style.display = 'none'
    document.getElementById('preview-error').style.display = 'none'
  }

  function showError(message) {
    var error = document.getElementById('preview-error')
    error.textContent = message || 'No se pudo generar la previsualización.'
    error.style.display = 'block'
    document.getElementById('preview-loading').style.display = 'none'
    document.getElementById('preview-frame').style.display = 'none'
  }

  function showPdf(blob) {
    var url = URL.createObjectURL(blob)
    var frame = document.getElementById('preview-frame')
    frame.src = url
    frame.style.display = 'block'
    document.getElementById('preview-loading').style.display = 'none'
    document.getElementById('preview-error').style.display = 'none'
  }

  // Solicita el PDF al backend con el JWT del usuario.
  function fetchPreview(data) {
    var backendUrl = (data.backendUrl || '').replace(/\/$/, '')
    var headers = {}
    if (data.token) {
      headers.Authorization = 'Bearer ' + data.token
    }
    showLoading()
    // 1) Fuerza el guardado en ONLYOFFICE (forcesave) para que el backend tenga la
    //    última versión del documento antes de generar el PDF. El endpoint espera a
    //    que el callback status 6 haya persistido el archivo.
    forceSave(backendUrl, data)
      .catch(function () {
        // Si el forcesave falla se continúa igualmente con lo último guardado.
      })
      .then(function () {
        // 2) Genera el PDF con el contenido ya persistido.
        return fetch(backendUrl + '/api/documents/' + encodeURIComponent(data.docId) + '/preview', {
          headers: headers
        })
      })
      .then(function (response) {
        if (!response.ok) {
          return response.text().then(function (text) {
            throw new Error(text || 'Error HTTP ' + response.status)
          })
        }
        return response.blob()
      })
      .then(function (blob) {
        if (!blob || !blob.size) {
          throw new Error('La previsualización está vacía.')
        }
        showPdf(blob)
      })
      .catch(function (error) {
        showError(error.message)
      })
  }

  function forceSave(backendUrl, data) {
    var headers = { 'Content-Type': 'application/json' }
    if (data.token) {
      headers.Authorization = 'Bearer ' + data.token
    }
    return fetch(backendUrl + '/api/documents/' + encodeURIComponent(data.docId) + '/force-save', {
      method: 'POST',
      headers: headers,
      body: JSON.stringify({ key: data.key || '' })
    }).then(function (response) {
      if (!response.ok) {
        return response.text().then(function (text) {
          throw new Error('El guardado previo falló: ' + (text || 'Error HTTP ' + response.status))
        })
      }
      return response.json()
    })
  }

  // Avisa al plugin padre que el modal está listo para recibir los datos.
  window.Asc.plugin.init = function () {
    window.Asc.plugin.sendToPlugin('onPreviewReady')
  }

  // Recibe los datos de sesión desde el plugin padre y genera el PDF.
  window.Asc.plugin.attachEvent('onPreviewData', function (data) {
    fetchPreview(data || {})
  })

  window.Asc.plugin.onThemeChanged = function (theme) {
    window.Asc.plugin.onThemeChangedBase(theme)
  }

  window.Asc.plugin.button = function (id) {
    this.executeCommand('close', '')
  }
})(window, undefined)
