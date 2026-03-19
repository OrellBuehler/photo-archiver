import type { ProgressMessage } from './types';

type Listener = (msg: ProgressMessage) => void;

let ws: WebSocket | null = null;
let listeners: Listener[] = [];
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;

function getWsUrl(): string {
  const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  return `${proto}//${window.location.host}/api/ws`;
}

function connect() {
  if (ws && ws.readyState <= 1) return;

  ws = new WebSocket(getWsUrl());

  ws.onmessage = (event) => {
    try {
      const msg: ProgressMessage = JSON.parse(event.data);
      listeners.forEach(fn => fn(msg));
    } catch {}
  };

  ws.onclose = () => {
    reconnectTimer = setTimeout(connect, 3000);
  };

  ws.onerror = () => {
    ws?.close();
  };
}

export function subscribe(fn: Listener): () => void {
  if (listeners.length === 0) connect();
  listeners.push(fn);

  return () => {
    listeners = listeners.filter(l => l !== fn);
    if (listeners.length === 0 && ws) {
      if (reconnectTimer) clearTimeout(reconnectTimer);
      ws.close();
      ws = null;
    }
  };
}
