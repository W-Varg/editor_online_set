(function (window, undefined) {
  var docId = "";
  var token = "";
  var backendUrl = "";
  var selectedUserId = null;

  var UI = {
    container: null,
    searchInput: null,
    resultsDiv: null,
    shareBtn: null,
    shareListDiv: null,
  };

  function initUI() {
    UI.container = document.getElementById("asc-sidebar-container") ||
                   document.querySelector(".sidebar-content") ||
                   document.body;

    var html =
      '<div style="padding:12px;font-family:-apple-system,sans-serif;font-size:13px">' +
      '<div style="font-weight:600;margin:0 0 8px;padding-bottom:4px;border-bottom:2px solid #e5e7eb">Compartir documento</div>' +
      '<input type="text" id="share-search-input" placeholder="Buscar por nombre, DNI..." ' +
      'style="width:100%;padding:8px;border:1px solid #ddd;border-radius:4px;margin-bottom:8px;box-sizing:border-box">' +
      '<div id="share-search-results"></div>' +
      '<button id="share-do-btn" disabled ' +
      'style="width:100%;padding:8px;background:#2563eb;color:#fff;border:none;border-radius:4px;cursor:pointer;font-size:13px;margin:8px 0">Compartir</button>' +
      '<div style="font-weight:600;margin:12px 0 8px;padding-bottom:4px;border-bottom:2px solid #e5e7eb">Usuarios con acceso</div>' +
      '<div id="share-list-container"><p style="color:#666;font-size:12px">Cargando...</p></div>' +
      "</div>";

    if (UI.container) {
      UI.container.innerHTML = html;
    }

    UI.searchInput = document.getElementById("share-search-input");
    UI.resultsDiv = document.getElementById("share-search-results");
    UI.shareBtn = document.getElementById("share-do-btn");
    UI.shareListDiv = document.getElementById("share-list-container");

    if (UI.searchInput) {
      UI.searchInput.addEventListener("input", function () {
        selectedUserId = null;
        UI.shareBtn.disabled = true;
        searchUsers(this.value);
      });
    }

    if (UI.shareBtn) {
      UI.shareBtn.addEventListener("click", shareDocument);
    }
  }

  window.Asc.plugin.init = function () {
    var data = this.info && this.info.pluginData ? this.info.pluginData : [];
    docId = data[0] || "";
    token = data[1] || "";
    backendUrl = data[2] || "";

    var text = this.info && this.info.text;
    if (text) {
      try {
        var parsed = JSON.parse(text);
        docId = parsed.docId || docId;
        token = parsed.token || token;
        backendUrl = parsed.backendUrl || backendUrl;
      } catch (e) {}
    }

    initUI();
    loadCurrentShares();
  };

  window.Asc.plugin.button = function (id) {
    if (id === "restringida-share") {
      this.executeCommand("sidebar", "show");
    }
  };

  window.Asc.plugin.onExternalMouseUp = function () {};

  function apiGet(path) {
    return fetch(backendUrl + path, {
      headers: { Authorization: "Bearer " + token }
    }).then(function (r) {
      if (!r.ok) throw new Error("HTTP " + r.status);
      return r.json();
    });
  }

  function apiPost(path, body) {
    return fetch(backendUrl + path, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer " + token
      },
      body: JSON.stringify(body)
    }).then(function (r) {
      if (!r.ok) return r.text().then(function (t) { throw new Error(t); });
      return r.json();
    });
  }

  function apiDelete(path) {
    return fetch(backendUrl + path, {
      method: "DELETE",
      headers: { Authorization: "Bearer " + token }
    }).then(function (r) {
      if (!r.ok) throw new Error("HTTP " + r.status);
    });
  }

  function loadCurrentShares() {
    apiGet("/api/documents/" + docId + "/shares")
      .then(function (shares) {
        if (!UI.shareListDiv) return;
        UI.shareListDiv.innerHTML = "";
        if (shares.length === 0) {
          UI.shareListDiv.innerHTML = '<p style="color:#666;font-size:12px">Sin compartir</p>';
          return;
        }
        shares.forEach(function (s) {
          var div = document.createElement("div");
          div.style.cssText =
            "display:flex;justify-content:space-between;align-items:center;padding:8px;border-bottom:1px solid #eee";
          div.innerHTML =
            '<div style="flex:1"><strong>' + s.user_name +
            '</strong><br><small style="color:#666">Compartido por: ' + s.shared_by_name + "</small></div>" +
            '<button class="share-remove-btn" data-user="' + s.user_id + '" ' +
            'style="background:#fee2e2;border:1px solid #fca5a5;color:#dc2626;padding:4px 10px;border-radius:4px;cursor:pointer;font-size:12px">Quitar</button>';
          UI.shareListDiv.appendChild(div);
        });
        UI.shareListDiv.querySelectorAll(".share-remove-btn").forEach(function (btn) {
          btn.addEventListener("click", function () {
            removeShare(this.getAttribute("data-user"));
          });
        });
      })
      .catch(function (e) { console.error("Error loading shares:", e); });
  }

  function searchUsers(query) {
    if (query.length < 2) {
      if (UI.resultsDiv) UI.resultsDiv.innerHTML = "";
      return;
    }
    apiGet("/api/users/search?q=" + encodeURIComponent(query))
      .then(function (users) {
        if (!UI.resultsDiv) return;
        UI.resultsDiv.innerHTML = "";
        if (users.length === 0) {
          UI.resultsDiv.innerHTML = '<p style="color:#666;font-size:12px">Sin resultados</p>';
          return;
        }
        users.forEach(function (u) {
          var div = document.createElement("div");
          div.style.cssText =
            "padding:8px;cursor:pointer;border-bottom:1px solid #eee;transition:background 0.2s";
          div.innerHTML =
            "<strong>" + u.name + "</strong><br>" +
            '<small style="color:#666">' + (u.dni || "") + " | " + (u.cargo || "") + "</small>";
          div.addEventListener("click", function () {
            selectedUserId = u.id;
            if (UI.shareBtn) UI.shareBtn.disabled = false;
            UI.resultsDiv.querySelectorAll("div").forEach(function (el) {
              el.style.background = "";
            });
            div.style.background = "#dbeafe";
          });
          div.addEventListener("mouseenter", function () {
            if (selectedUserId !== u.id) div.style.background = "#f0f4ff";
          });
          div.addEventListener("mouseleave", function () {
            if (selectedUserId !== u.id) div.style.background = "";
          });
          UI.resultsDiv.appendChild(div);
        });
      })
      .catch(function (e) { console.error("Error searching users:", e); });
  }

  function removeShare(userId) {
    apiDelete("/api/documents/" + docId + "/shares/" + userId)
      .then(function () {
        selectedUserId = null;
        if (UI.shareBtn) UI.shareBtn.disabled = true;
        loadCurrentShares();
      })
      .catch(function (e) { console.error("Error removing share:", e); });
  }

  function shareDocument() {
    if (!selectedUserId) return;
    apiPost("/api/documents/" + docId + "/shares", { user_id: selectedUserId })
      .then(function () {
        if (UI.searchInput) UI.searchInput.value = "";
        if (UI.resultsDiv) UI.resultsDiv.innerHTML = "";
        selectedUserId = null;
        if (UI.shareBtn) UI.shareBtn.disabled = true;
        loadCurrentShares();
      })
      .catch(function (e) {
        alert("Error: " + e.message);
        console.error("Error sharing:", e);
      });
  }
})(window);
