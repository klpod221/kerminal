import type { TunnelWithStatus } from "../types/tunnel";
import { api } from "./api";

type Unsubscribe = () => void;

export type TunnelEvents =
  | { type: "tunnel_started"; tunnel: TunnelWithStatus }
  | { type: "tunnel_stopped"; tunnel: TunnelWithStatus }
  | { type: "tunnel_status_changed"; tunnel: TunnelWithStatus };

export type AuthSessionEvent = {
  type:
    | "session_unlocked"
    | "session_locked"
    | "session_updated";
  payload: any; // keep generic; consumer can type-narrow
};

class RealtimeService {
  async subscribeTunnels(
    onEvent: (evt: TunnelEvents) => void,
  ): Promise<() => void> {
    const unsubs: Unsubscribe[] = [];

    unsubs.push(
      await api.listen<TunnelWithStatus>("tunnel_started", (tunnel) => {
        onEvent({ type: "tunnel_started", tunnel });
      }),
    );

    unsubs.push(
      await api.listen<TunnelWithStatus>("tunnel_stopped", (tunnel) => {
        onEvent({ type: "tunnel_stopped", tunnel });
      }),
    );

    unsubs.push(
      await api.listen<TunnelWithStatus>(
        "tunnel_status_changed",
        (tunnel) => {
          onEvent({ type: "tunnel_status_changed", tunnel });
        },
      ),
    );

    return () => {
      unsubs.forEach((u) => u());
    };
  }

  async subscribeAuth(
    onEvent: (evt: AuthSessionEvent) => void,
  ): Promise<() => void> {
    const unsubs: Unsubscribe[] = [];

    unsubs.push(
      await api.listen<any>("auth_session_unlocked", (payload) => {
        onEvent({ type: "session_unlocked", payload });
      }),
    );

    unsubs.push(
      await api.listen<any>("auth_session_locked", (payload) => {
        onEvent({ type: "session_locked", payload });
      }),
    );

    unsubs.push(
      await api.listen<any>("auth_session_updated", (payload) => {
        onEvent({ type: "session_updated", payload });
      }),
    );

    return () => {
      unsubs.forEach((u) => u());
    };
  }
}

export const realtimeService = new RealtimeService();


