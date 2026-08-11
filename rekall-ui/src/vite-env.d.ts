/// <reference types="vite/client" />

/**
 * Only the variables the app actually reads. `env.ts` validates them at boot, so anything
 * declared here that is missing at runtime stops the application rather than surfacing later.
 */
interface ImportMetaEnv {
  readonly VITE_API_BASE_URL?: string
  readonly VITE_APP_TITLE?: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv & { readonly MODE: string; readonly BASE_URL: string }
}
