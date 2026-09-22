import { apiClient, request } from './client'
import { SearchHitListSchema } from './schemas/search.schema'
import type { SearchHit } from '@/model/search'

export async function searchText(term: string): Promise<SearchHit[]> {
  return request(async () => SearchHitListSchema.parse(await apiClient('/api/search', { query: { q: term } })))
}
