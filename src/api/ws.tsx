import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import type { WsClientMessage, WsServerMessage } from "./types";

type TopicHandler = (payload: unknown) => void;
type SocketStatus = "connecting" | "open" | "closed";

interface WsContextValue {
  status: SocketStatus;
  subscribe: (topic: string, handler: TopicHandler) => () => void;
}

const WsContext = createContext<WsContextValue | null>(null);

const MAX_RETRY_DELAY_MS = 15_000;

function socketUrl(): string {
  const scheme = location.protocol === "https:" ? "wss" : "ws";
  return `${scheme}://${location.host}/api/v1/ws`;
}

export function WebSocketProvider({ children }: { children: ReactNode }) {
  const [status, setStatus] = useState<SocketStatus>("connecting");
  const socketRef = useRef<WebSocket | null>(null);
  const handlersRef = useRef(new Map<string, Set<TopicHandler>>());

  const send = useCallback((message: WsClientMessage) => {
    const socket = socketRef.current;
    if (socket?.readyState === WebSocket.OPEN) socket.send(JSON.stringify(message));
  }, []);

  useEffect(() => {
    let closedByCleanup = false;
    let retryTimer: ReturnType<typeof setTimeout> | undefined;
    let attempts = 0;

    const connect = () => {
      const socket = new WebSocket(socketUrl());
      socketRef.current = socket;
      setStatus("connecting");

      socket.onopen = () => {
        attempts = 0;
        setStatus("open");
        for (const topic of handlersRef.current.keys()) send({ op: "subscribe", topic });
      };

      socket.onmessage = (event: MessageEvent<string>) => {
        let message: WsServerMessage;
        try {
          message = JSON.parse(event.data) as WsServerMessage;
        } catch {
          return;
        }
        const payload = message.payload ?? message.data;
        handlersRef.current.get(message.topic)?.forEach((handler) => handler(payload));
      };

      socket.onclose = () => {
        if (closedByCleanup) return;
        setStatus("closed");
        const delay = Math.min(1000 * 2 ** attempts, MAX_RETRY_DELAY_MS);
        attempts += 1;
        retryTimer = setTimeout(connect, delay);
      };
    };

    connect();
    return () => {
      closedByCleanup = true;
      clearTimeout(retryTimer);
      socketRef.current?.close();
    };
  }, [send]);

  const subscribe = useCallback(
    (topic: string, handler: TopicHandler) => {
      let handlers = handlersRef.current.get(topic);
      if (!handlers) {
        handlers = new Set();
        handlersRef.current.set(topic, handlers);
        send({ op: "subscribe", topic });
      }
      const topicHandlers = handlers;
      topicHandlers.add(handler);

      return () => {
        topicHandlers.delete(handler);
        if (topicHandlers.size === 0 && handlersRef.current.get(topic) === topicHandlers) {
          handlersRef.current.delete(topic);
          send({ op: "unsubscribe", topic });
        }
      };
    },
    [send],
  );

  const value = useMemo(() => ({ status, subscribe }), [status, subscribe]);
  return <WsContext.Provider value={value}>{children}</WsContext.Provider>;
}

export function useWebSocket(): WsContextValue {
  const context = useContext(WsContext);
  if (!context) throw new Error("useWebSocket must be used inside WebSocketProvider");
  return context;
}

export function useTopic(topic: string | null, onMessage: TopicHandler) {
  const { subscribe } = useWebSocket();
  const handlerRef = useRef(onMessage);
  handlerRef.current = onMessage;

  useEffect(() => {
    if (!topic) return;
    return subscribe(topic, (payload) => handlerRef.current(payload));
  }, [topic, subscribe]);
}
