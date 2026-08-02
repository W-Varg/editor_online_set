/*
 * Plugin "Saludar" para ONLYOFFICE.
 *
 * Comportamiento:
 *  - Se ejecuta dentro del panel lateral (isInsideMode) del editor.
 *  - El usuario escribe un texto en el campo de entrada.
 *  - Al pulsar "Mostrar saludo" se abre una ventana modal (saludo.html) que
 *    muestra el texto, replicando el patrón de previsualización de doc2md.
 *
 * Arquitectura de la ventana modal:
 *  - La ventana se crea con `window.Asc.PluginWindow` y `show(variation)`.
 *  - Cuando el modal termina de inicializarse envía `onSaludoReady` (mediante
 *    `sendToPlugin`), y aquí lo escuchamos con `attachEvent` para responderle
 *    con `previewWindow.command('onSaludoData', { texto })`.
 *  - El modal recibe esos datos con `window.Asc.plugin.attachEvent("onSaludoData")`
 *    y los pinta. (Ver saludo.js).
 *
 * Buenas prácticas aplicadas:
 *  - No se define CSS propio: se reutilizan las clases de plugins.css del SDK.
 *  - `Asc.plugin.onThemeChangedBase` se delega al SDK para que adapte los colores.
 *  - `Asc.plugin.button` cierra la ventana modal (si hay id) o el plugin.
 *  - Todo el código va en un IIFE con "use strict" para no ensuciar el scope global.
 */
;(function (window, undefined) {
  'use strict'

  window.Asc = window.Asc || {}
  window.Asc.plugin = window.Asc.plugin || {}
  // El GUID debe coincidir con "guid" del config.json.
  window.Asc.plugin.guid = 'asc.{8f2a1c40-7b3d-4e21-9a6f-000000000002}'

  // Se mantiene una única instancia reutilizable de la ventana modal.
  var previewWindow = null

  // Lee el texto del input y lo envía al modal cuando este avisa que está listo.
  function sendGreeting() {
    var input = document.getElementById('saludo-input')
    var text = input && input.value.trim() ? input.value.trim() : '(sin texto)'
    previewWindow.command('onSaludoData', { texto: text })
  }

  // Construye la variación del modal y lo muestra, igual que hace doc2md con
  // su ventana de previsualización (ver plugins/comunity/doc2md/scripts/d2md.js).
  function openGreetingWindow() {
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
      buttons: [{ text: 'Cerrar', primary: false }],
      size: [420, 200]
    }

    if (!previewWindow) {
      previewWindow = new window.Asc.PluginWindow()
      // Se ejecuta cuando el modal envía "onSaludoReady" tras inicializarse.
      previewWindow.attachEvent('onSaludoReady', sendGreeting)
    }
    previewWindow.show(variation)
  }

  // Hook de inicialización: enlaza el botón del panel lateral.
  window.Asc.plugin.init = function () {
    var button = document.getElementById('btn-saludar')
    if (button) {
      button.addEventListener('click', openGreetingWindow)
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
