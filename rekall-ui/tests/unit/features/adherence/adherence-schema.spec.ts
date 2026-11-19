import { describe, expect, it } from 'vitest'
import { ProjectAdherenceSchema } from '@/features/adherence/api/schemas/adherence.schema'

describe('ProjectAdherenceSchema', () => {
  it('reads the task code the server sends for the most sent back tasks', () => {
    const payload = {
      stepsTotal: 4,
      stepsDone: 4,
      stepsClaimed: 0,
      acceptedFirstTime: 1,
      sentBack: 3,
      passesTotal: 9,
      firstTimeShare: 0.25,
      deviations: 1,
      mostSentBack: [{ taskId: '0b0c8a0e-6a5e-4a3e-9d0e-3f6a5b7c8d9e', code: 'RK-12', title: 'Import', sentBack: 2, passesTotal: 7 }]
    }

    expect(ProjectAdherenceSchema.parse(payload).mostSentBack.map((task) => task.code)).toEqual(['RK-12'])
  })
})
