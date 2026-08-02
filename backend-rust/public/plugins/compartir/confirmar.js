/*
 * Script de la ventana modal de confirmación del plugin "Compartir".
 *
 * Ciclo de vida (complementa a plugin.js):
 *  - Al inicializarse envía `onConfirmReady` al plugin padre (sendToPlugin).
 *  - El padre responde con `confirmWindow.command('onConfirmData', { add, remove })`
 *    y aquí se escucha con `attachEvent("onConfirmData", ...)` para mostrar el
 *    resumen de cambios (nombres de usuarios).
 *  - "Aceptar" envía `onConfirmAccept` al padre, que ejecuta la sincronización.
 *  - "Rechazar" envía `onConfirmReject` al padre, que cierra el modal sin cambios.
 *
 * Buenas prácticas: sin CSS propio, delegación de tema en `onThemeChangedBase`
 * y código en un IIFE con "use strict".
 */
;(function (window, undefined) {
  'use strict'

  window.Asc = window.Asc || {}
  window.Asc.plugin = window.Asc.plugin || {}

  function el(tag, className, text) {
    var node = document.createElement(tag)
    if (className) node.className = className
    if (text !== undefined) node.textContent = text
    return node
  }

  // Dibuja el resumen de cambios (usuarios a compartir / a los que quitar acceso).
  function renderSummary(data) {
    var container = document.getElementById('resumen')
    container.innerHTML = ''
    var add = (data && data.add) || []
    var remove = (data && data.remove) || []

    container.appendChild(el('div', '', '¿Confirma los siguientes cambios de acceso al documento?'))

    if (add.length) {
      container.appendChild(el('div', '', ''))
      container.appendChild(el('strong', '', 'Compartir con (' + add.length + '):'))
      var ulAdd = el('ul', '', '')
      add.forEach(function (name) { ulAdd.appendChild(el('li', '', name)) })
      container.appendChild(ulAdd)
    }
    if (remove.length) {
      container.appendChild(el('div', '', ''))
      container.appendChild(el('strong', '', 'Quitar acceso a (' + remove.length + '):'))
      var ulRemove = el('ul', '', '')
      remove.forEach(function (name) { ulRemove.appendChild(el('li', '', name)) })
      container.appendChild(ulRemove)
    }
  }

  // Avisa al plugin padre que el modal está listo para recibir el resumen.
  window.Asc.plugin.init = function () {
    window.Asc.plugin.sendToPlugin('onConfirmReady')
  }

  // Recibe el resumen de cambios desde el plugin padre y lo muestra.
  window.Asc.plugin.attachEvent('onConfirmData', function (data) {
    renderSummary(data)
  })

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

  function onReady() {
    document.getElementById('btn-aceptar').addEventListener('click', function () {
      window.Asc.plugin.sendToPlugin('onConfirmAccept')
    })
    document.getElementById('btn-rechazar').addEventListener('click', function () {
      window.Asc.plugin.sendToPlugin('onConfirmReject')
    })
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', onReady)
  } else {
    onReady()
  }
})(window, undefined)
