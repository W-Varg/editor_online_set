#!/usr/bin/env bash
# Oculta la pestaña "File/Archivo" en DocumentServer (edición community).
# La edición community no permite ocultar la tab File vía configuración
# (solo customization.layout.* con licencia extendida), por lo que se
# parchean los assets de web-apps dentro del contenedor.
#
# IMPORTANTE: el parche se pierde si el contenedor se recrea (docker compose up
# --force-recreate / rm). Vuelve a ejecutar este script tras recrearlo.
#
# Uso: scripts/patch_onlyoffice_hide_file_tab.sh
set -euo pipefail

CONTAINER="${CONTAINER:-editor_online_set-onlyoffice-1}"
BASE="/var/www/onlyoffice/documentserver/web-apps/apps"

for editor in documenteditor spreadsheeteditor presentationeditor; do
    file="$BASE/$editor/main/app/view/Toolbar.js"
    docker exec "$CONTAINER" bash -c "set -euo pipefail; file='$file'; [ -f \"\$file\" ] || { echo \"SKIP \$file (no existe)\"; exit 0; }; cp -n \"\$file\" \"\$file.bak\" 2>/dev/null || true; chmod +w \"\$file\"; sed -i 's|.*caption: me.textTabFile.*|// &|' \"\$file\""
    count=$(docker exec "$CONTAINER" grep -c "caption: me.textTabFile" "$file")
    echo "$editor: entradas de tab File restantes sin comentar = $count"
done

echo "Reiniciando DocumentServer..."
docker restart "$CONTAINER"
