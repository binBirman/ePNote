import { invoke } from "@tauri-apps/api/core";

export async function call<T>(
  cmd: string,
  payload?: Record<string, unknown>
): Promise<T> {
  try {
    console.debug('invoke', cmd, payload)
    return await invoke<T>(cmd, payload);
  } catch (e) {
    console.error("Backend error:", e);
    throw new Error(String(e));
  }
}
