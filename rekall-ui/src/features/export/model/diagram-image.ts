/** The picture of one mermaid fence, found again by its `source` on the server. */
export type DiagramImage = Readonly<{ source: string; png: Uint8Array }>

export type DiagramFailure = Readonly<{ source: string; reason: string }>

/** How far drawing has got: every source finished, drawn or not, out of the total. */
export type DiagramProgress = Readonly<{ finished: number; total: number; failures: readonly DiagramFailure[] }>
