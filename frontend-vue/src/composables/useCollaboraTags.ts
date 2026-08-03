import { onBeforeUnmount, onMounted, ref, type Ref } from 'vue'

/**
 * Inserta etiquetas `{{key}}` en el cursor del documento abierto en el iframe
 * de Collabora usando el API de postMessage (`Send_UNO_Command` + `.uno:InsertText`).
 *
 * Collabora no tiene un sistema de plugins como ONLYOFFICE, así que la
 * "inserción" se hace desde la página que embebe el iframe una vez que
 * Collabora notifica `Host_PostmessageReady`.
 */
export function useCollaboraTags(iframeRef: Ref<HTMLIFrameElement | null>) {
  const ready = ref(false)

  function onMessage(event: MessageEvent) {
    try {
      const data = typeof event.data === 'string' ? JSON.parse(event.data) : event.data
      if (data && typeof data.MessageId === 'string' && data.MessageId.includes('PostmessageReady')) {
        ready.value = true
      }
    } catch {
      // noop: mensajes que no son JSON del iframe
    }
  }

  onMounted(() => window.addEventListener('message', onMessage))
  onBeforeUnmount(() => window.removeEventListener('message', onMessage))

  async function insertTag(key: string): Promise<boolean> {
    const iframe = iframeRef.value
    if (!iframe?.contentWindow) return false
    try {
      iframe.contentWindow.postMessage(
        JSON.stringify({
          MessageId: 'Send_UNO_Command',
          SendTime: Date.now(),
          Values: {
            Command: '.uno:InsertText',
            Args: {
              Text: { type: 'string', value: `{{${key}}}` },
            },
          },
        }),
        '*',
      )
      return true
    } catch {
      return false
    }
  }

  return { ready, insertTag }
}
