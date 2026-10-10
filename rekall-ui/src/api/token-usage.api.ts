import { apiClient, request } from './client'
import { TokenTallyListSchema } from './schemas/token-usage.schema'
import type { TokenTally } from '@/model/token-usage'

export async function fetchTokenTallies(): Promise<TokenTally[]> {
  return request(async () => TokenTallyListSchema.parse(await apiClient('/api/token-usage')))
}
