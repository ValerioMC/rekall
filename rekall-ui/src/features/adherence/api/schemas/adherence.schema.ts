import { z } from 'zod'
import { asTaskId } from '@/model/branded'

const FiguresShape = {
  stepsTotal: z.number().int(),
  stepsDone: z.number().int(),
  stepsClaimed: z.number().int(),
  acceptedFirstTime: z.number().int(),
  sentBack: z.number().int(),
  passesTotal: z.number().int(),
  firstTimeShare: z.number().nullable(),
  deviationNotes: z.number().int()
}

export const StepVisitsSchema = z.object({
  stepNumber: z.number().int(),
  title: z.string(),
  visits: z.number().int(),
  maxVisits: z.number().int().nullable(),
  budgetSpent: z.boolean()
})

export const PlanAdherenceSchema = z.object({
  ...FiguresShape,
  deviationSteps: z.array(z.number().int()),
  visits: z.array(StepVisitsSchema)
})

export const TaskSentBackSchema = z.object({
  taskId: z.string().uuid().transform(asTaskId),
  label: z.string(),
  title: z.string(),
  sentBack: z.number().int(),
  passesTotal: z.number().int()
})

export const ProjectAdherenceSchema = z.object({
  ...FiguresShape,
  mostSentBack: z.array(TaskSentBackSchema)
})
