declare module '@tauri-apps/api/tauri' {
  export function invoke<T = any>(cmd: string, args?: Record<string, any>): Promise<T>
}

declare module '@tauri-apps/api/dialog' {
  export type DialogFilter = { name?: string; extensions?: string[] }
  export type OpenOptions = {
    multiple?: boolean
    directory?: boolean
    filters?: DialogFilter[]
    defaultPath?: string
    title?: string
  }
  export function open(options?: OpenOptions): Promise<string | string[] | null>
}
