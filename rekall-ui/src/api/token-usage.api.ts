import { apiClient, request } from './client'
import { TokenDayListSchema, TokenTallyListSchema } from './schemas/token-usage.schema'
import type { TokenDay, TokenTally } from '@/model/token-usage'

export async function fetchTokenTallies(): Promise<TokenTally[]> {
  return request(async () => TokenTallyListSchema.parse(await apiClient('/api/token-usage')))
}

export async function fetchTokenDays(): Promise<TokenDay[]> {
  return request(async () => TokenDayListSchema.parse(await apiClient('/api/token-usage/daily')))
}
