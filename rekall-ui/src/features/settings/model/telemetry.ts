export interface TelemetrySettings {
  readonly enabled: boolean
  readonly host: string
  readonly publicKey: string
  readonly logContent: boolean
  /** The secret key itself never comes back: only whether one is stored. */
  readonly secretSaved: boolean
}

export interface TelemetryChange {
  readonly enabled: boolean
  readonly host: string
  readonly publicKey: string
  readonly logContent: boolean
  /** Left out, the stored key is kept. */
  readonly secretKey?: string
}
