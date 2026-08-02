/*
 * Script de la ventana modal del plugin "Saludar".
 *
 * Ciclo de vida (complementa a plugin.js):
 *  - Al terminar de inicializarse envía `onSaludoReady` al plugin padre mediante
 *    `sendToPlugin`, indicando que ya puede recibir el texto.
 *  - El plugin padre responde con `previewWindow.command('onSaludoData', { texto })`
 *    y aquí se escucha con `attachEvent("onSaludoData", ...)` para pintar el texto.
 *
 * Buenas prácticas: mismo enfoque sin CSS propio, delegación de tema en
 * `onThemeChangedBase`, y código contenido en un IIFE con "use strict".
 */
;(function (window, undefined) {
  'use strict'

  window.Asc = window.Asc || {}
  window.Asc.plugin = window.Asc.plugin || {}

  // Avisa al plugin padre que el modal está listo para recibir el saludo.
  window.Asc.plugin.init = function () {
    window.Asc.plugin.sendToPlugin('onSaludoReady')
  }

  // Recibe el texto desde el plugin padre y lo muestra en el contenedor.
  window.Asc.plugin.attachEvent('onSaludoData', function (data) {
    var container = document.getElementById('saludo-container')
    if (!container) return
    var text = data && data.texto ? data.texto : '(sin texto)'
    container.textContent = '¡Hola! Dijiste: ' + text
  })

  window.Asc.plugin.onThemeChanged = function (theme) {
    window.Asc.plugin.onThemeChangedBase(theme)
  }

  window.Asc.plugin.button = function (id) {
    this.executeCommand('close', '')
  }
})(window, undefined)
