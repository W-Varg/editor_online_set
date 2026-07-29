(function (window, undefined) {
  var docId = "";
  var token = "";
  var backendUrl = "";

  var UI = {
    container: null,
    input: null,
    showBtn: null,
    dialogOverlay: null,
    dialogBody: null,
    dialogClose: null,
    dialogOk: null,
  };

  function escapeHtml(value) {
    return String(value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/\"/g, "&quot;")
      .replace(/'/g, "&#39;");
  }

  function readPluginData() {
    var data = (window.Asc && window.Asc.plugin && window.Asc.plugin.info && window.Asc.plugin.info.pluginData)
      ? window.Asc.plugin.info.pluginData
      : [];

    docId = data[0] || "";
    token = data[1] || "";
    backendUrl = data[2] || "";

    var text = window.Asc && window.Asc.plugin && window.Asc.plugin.info && window.Asc.plugin.info.text;
    if (!text) return;

    try {
      var parsed = JSON.parse(text);
      docId = parsed.docId || docId;
      token = parsed.token || token;
      backendUrl = parsed.backendUrl || backendUrl;
    } catch (e) {}
  }

  function mountUI() {
    UI.container = document.getElementById("asc-sidebar-container") ||
      document.querySelector(".sidebar-content") ||
      document.body;

    if (!UI.container) return;

    UI.container.innerHTML =
      '<div class="plugin-shell">' +
        '<div class="section-title">Participantes</div>' +
        '<p class="helper-text">Escribe un texto y presiona <strong>Mostrar</strong> para verlo en un dialogo.</p>' +
        '<label class="field-label" for="participant-text">Texto</label>' +
        '<input id="participant-text" class="text-input" type="text" placeholder="Escribe aqui el texto a mostrar" autocomplete="off">' +
        '<button id="show-btn" class="btn-action" type="button">Mostrar</button>' +
        '<div id="dialog-overlay" class="dialog-overlay" hidden>' +
          '<div class="dialog-card" role="dialog" aria-modal="true" aria-labelledby="dialog-title">' +
            '<div class="dialog-header">' +
              '<div id="dialog-title" class="dialog-title">Texto capturado</div>' +
              '<button id="dialog-close" class="dialog-close" type="button" aria-label="Cerrar dialogo">×</button>' +
            '</div>' +
            '<div id="dialog-body" class="dialog-body"><span class="empty-state">Sin contenido</span></div>' +
            '<div class="dialog-footer">' +
              '<button id="dialog-ok" class="btn-secondary" type="button">Cerrar</button>' +
            '</div>' +
          '</div>' +
        '</div>' +
      '</div>';

    UI.input = document.getElementById("participant-text");
    UI.showBtn = document.getElementById("show-btn");
    UI.dialogOverlay = document.getElementById("dialog-overlay");
    UI.dialogBody = document.getElementById("dialog-body");
    UI.dialogClose = document.getElementById("dialog-close");
    UI.dialogOk = document.getElementById("dialog-ok");

    if (UI.showBtn) {
      UI.showBtn.addEventListener("click", function () {
        openDialog(UI.input ? UI.input.value : "");
      });
    }

    if (UI.input) {
      UI.input.addEventListener("keydown", function (event) {
        if (event.key === "Enter") {
          event.preventDefault();
          openDialog(UI.input.value);
        }
      });
    }

    if (UI.dialogClose) {
      UI.dialogClose.addEventListener("click", closeDialog);
    }

    if (UI.dialogOk) {
      UI.dialogOk.addEventListener("click", closeDialog);
    }

    if (UI.dialogOverlay) {
      UI.dialogOverlay.addEventListener("click", function (event) {
        if (event.target === UI.dialogOverlay) closeDialog();
      });
    }

    document.addEventListener("keydown", onDocumentKeydown);

    if (UI.input) {
      setTimeout(function () { UI.input.focus(); }, 50);
    }
  }

  function onDocumentKeydown(event) {
    if (event.key === "Escape" && UI.dialogOverlay && !UI.dialogOverlay.hidden) {
      closeDialog();
    }
  }

  function openDialog(text) {
    if (!UI.dialogOverlay || !UI.dialogBody) return;

    var value = String(text || "").trim();
    UI.dialogBody.innerHTML = value
      ? '<div class="dialog-value">' + escapeHtml(value) + '</div>'
      : '<span class="empty-state">No has escrito ningun texto.</span>';

    UI.dialogOverlay.hidden = false;
  }

  function closeDialog() {
    if (UI.dialogOverlay) {
      UI.dialogOverlay.hidden = true;
    }
  }

  window.Asc = window.Asc || {};
  window.Asc.plugin = window.Asc.plugin || {};

  window.Asc.plugin.init = function () {
    readPluginData();
    mountUI();
  };

  window.Asc.plugin.button = function (id) {
    if (id === "participantes") {
      this.executeCommand("sidebar", "show");
    }
  };

  window.Asc.plugin.onExternalMouseUp = function () {};
})(window);
