import { ref, computed, onMounted, onUnmounted, reactive } from 'vue';
import { createLogStream, downloadLogs } from '../services/logs';
import type { LogEntry } from '../types/logs';
import type { LogLevel } from '../types/logs';

export function useLogs() {
  const logs = ref<LogEntry[]>([]);
  const loading = ref(true);
  const error = ref<string | null>(null);
  const isPaused = ref(false);
  const levels = ref<LogLevel[]>(['INFO', 'WARN', 'ERROR', 'DEBUG']);
  const keyword = ref('');
  let es: EventSource | null = null;
  let buffered: LogEntry[] = [];

  function connect() {
    loading.value = true;
    error.value = null;
    es = createLogStream(
      (entry) => {
        if (isPaused.value) {
          buffered.push(entry);
        } else {
          logs.value.push(entry);
          if (logs.value.length > 1000) {
            logs.value.shift();
          }
        }
      },
      (err) => {
        error.value = 'SSE connection error';
        console.error('SSE error:', err);
      }
    );
    loading.value = false;
  }

  function disconnect() {
    if (es) {
      es.close();
      es = null;
    }
  }

  function togglePause() {
    isPaused.value = !isPaused.value;
    if (!isPaused.value && buffered.length > 0) {
      logs.value.push(...buffered);
      buffered = [];
      if (logs.value.length > 1000) {
        logs.value = logs.value.slice(-1000);
      }
    }
  }

  function clearLogs() {
    logs.value = [];
    buffered = [];
  }

  async function download() {
    error.value = null;
    try {
      const blob = await downloadLogs();
      const text = await blob.text();
      const sanitized = maskSensitive(text);
      const sanitizedBlob = new Blob([sanitized], { type: blob.type || 'application/jsonl' });
      const url = URL.createObjectURL(sanitizedBlob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `logs-${new Date().toISOString()}.jsonl`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Download failed';
    }
  }

  function maskSensitive(text: string): string {
    // Mask common key-value patterns (api_key, apikey, api-key, token, secret, password)
    // and BasicAuth Bearer headers. Keep the key/prefix visible; replace the value.
    return text
      .replace(
        /((?:api[_-]?key|token|secret|password)\s*[:=]\s*)[^\n"']*/gi,
        '$1***REDACTED***'
      )
      .replace(/(Authorization:\s*Bearer\s+)[^\n]+/gi, '$1***REDACTED***');
  }

  const filteredLogs = computed(() => {
    return logs.value.filter((log) => {
      if (!levels.value.includes(log.level)) return false;
      if (keyword.value && !log.message.toLowerCase().includes(keyword.value.toLowerCase())) return false;
      return true;
    });
  });

  onMounted(connect);
  onUnmounted(disconnect);

  return reactive({
    logs,
    filteredLogs,
    loading,
    error,
    isPaused,
    levels,
    keyword,
    togglePause,
    clearLogs,
    reconnect: connect,
    download,
  });
}
