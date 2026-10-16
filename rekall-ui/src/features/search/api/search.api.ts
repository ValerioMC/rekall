import { apiClient, request } from '../../../api/client'
import { SearchHitListSchema } from './schemas/search.schema'
import type { SearchHit } from '@/features/search/model/search'

export async function searchText(term: string): Promise<SearchHit[]> {
  return request(async () => SearchHitListSchema.parse(await apiClient('/api/search', { query: { q: term } })))
}
