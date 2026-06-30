export function hasTauri(): boolean {
  return typeof window !== "undefined" && window.__TAURI__ !== undefined;
}

export async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!hasTauri() || !window.__TAURI__) throw new Error("Not in Tauri");
  return window.__TAURI__.core.invoke(cmd, args);
}
