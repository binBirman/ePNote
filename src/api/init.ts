import { invoke } from "@tauri-apps/api/core";

export interface InitStatus {
  initialized: boolean;
  root?: string;
}

export function tauri_check_init_default(): Promise<InitStatus> {
  return invoke<InitStatus>("tauri_check_init_default");
}

export async function tauri_init_note(root: string): Promise<void> {
  await invoke('tauri_init_note', { root })
}
