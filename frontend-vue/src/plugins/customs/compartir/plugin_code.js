;(function (window, undefined) {
  var inputEl, btnMostrar, dialogOverlay, dialogText, dialogClose, dialogCloseFooter

  function initUI() {
    inputEl = document.getElementById('participante-input')
    btnMostrar = document.getElementById('btn-mostrar')
    dialogOverlay = document.getElementById('dialog-overlay')
    dialogText = document.getElementById('dialog-text')
    dialogClose = document.getElementById('dialog-close')
    dialogCloseFooter = document.getElementById('dialog-close-footer')

    btnMostrar.addEventListener('click', onMostrarClick)
    dialogClose.addEventListener('click', closeDialog)
    dialogCloseFooter.addEventListener('click', closeDialog)

    // Cerrar tambien haciendo click fuera del cuadro
    dialogOverlay.addEventListener('click', function (e) {
      if (e.target === dialogOverlay) {
        closeDialog()
      }
    })

    // Permitir mostrar con Enter
    inputEl.addEventListener('keyup', function (e) {
      if (e.key === 'Enter') {
        onMostrarClick()
      }
    })
  }

  function onMostrarClick() {
    var texto = inputEl.value || ''
    texto = texto.trim()

    if (texto.length === 0) {
      texto = '(no se escribió ningún texto)'
    }

    dialogText.textContent = texto
    openDialog()
  }

  function openDialog() {
    dialogOverlay.classList.remove('hidden')
  }

  function closeDialog() {
    dialogOverlay.classList.add('hidden')
  }

  // ------------------------------------------------------------------
  // Integración con la API de plugins de OnlyOffice
  // ------------------------------------------------------------------
  window.Asc.plugin.init = function () {
    initUI()
  }

  // Se ejecuta cuando el plugin/panel se cierra o se oculta
  window.Asc.plugin.button = function (id) {
    this.executeCommand('close', '')
  }

  // Requerido por la API aunque no se use variación por botones
  window.Asc.plugin.onExternalPluginMessage = function () {}
})(window, undefined)
