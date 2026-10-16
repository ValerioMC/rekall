import { z } from 'zod'

export const ContextSizeSchema = z.object({
  characters: z.number().int(),
  estimatedTokens: z.number().int(),
  parts: z.array(
    z.object({
      label: z.string(),
      characters: z.number().int(),
      reference: z.boolean()
    })
  )
})
