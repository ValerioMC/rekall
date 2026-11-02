<script setup lang="ts">
import { computed } from 'vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import { impactOf, plural } from '@/features/connection/lib/connection-impact'
import type { Connection, ConnectionDetail } from '@/features/connection/model/connection'

/** Asks before a connection goes, saying first what it takes with it: the companies, the bound projects and the stored credential. */
const props = defineProps<{ connection: Connection; detail: ConnectionDetail | null }>()
const emit = defineEmits<{ cancel: []; confirm: [] }>()

const impact = computed(() => (props.detail ? impactOf(props.detail) : null))
const body = computed(() => {
  if (!impact.value) return 'Sessions on the projects that use it can no longer reach it.'
  const shared = `Shared with ${plural(impact.value.companies, 'company', 'companies')}.`
  if (!impact.value.bindings) return `${shared} No project binds it.`
  return `${shared} ${plural(impact.value.projects, 'project')} lose${impact.value.projects === 1 ? 's' : ''} it, with ${plural(impact.value.bindings, 'binding')} and their targets.`
})
const confirmLabel = computed(() =>
  impact.value?.projects ? `Delete and unbind ${plural(impact.value.projects, 'project')}` : 'Delete connection'
)
</script>

<template>
  <AppConfirm
    :title="`Delete ${connection.title}?`"
    :body="body"
    blast="Its stored credential is removed from this machine."
    :confirm-label="confirmLabel"
    @cancel="emit('cancel')"
    @confirm="emit('confirm')"
  />
</template>
