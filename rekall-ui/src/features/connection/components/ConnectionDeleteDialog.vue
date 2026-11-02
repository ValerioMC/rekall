<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppDialog from '@/components/ui/AppDialog.vue'
import { impactOf, plural, projectsOf } from '@/features/connection/lib/connection-impact'
import { credentialNoun } from '@/features/connection/lib/credential-draft'
import type { Connection, ConnectionDetail } from '@/features/connection/model/connection'

/** Asks before a connection goes, saying first what it takes with it: the companies, the bound projects and the stored credential. */
const props = defineProps<{ connection: Connection; detail: ConnectionDetail | null }>()
const emit = defineEmits<{ cancel: []; confirm: [] }>()

const cancelButton = ref<InstanceType<typeof AppButton> | null>(null)

const impact = computed(() => (props.detail ? impactOf(props.detail) : null))
const consequences = computed(() => {
  const lines: string[] = []
  const detail = props.detail
  if (!detail) return ['Sessions on the projects that use it can no longer reach it.']
  const companies = detail.companies.map((company) => company.name)
  lines.push(companies.length ? `Shared with ${companies.join(', ')}.` : 'Shared with no company.')
  const projects = projectsOf(detail.projects).map((project) => project.title)
  lines.push(projects.length ? `Unbinds from ${projects.join(', ')}.` : 'No project binds it.')
  if (props.connection.credentialSetAt) lines.push(`Removes the stored ${credentialNoun(props.connection.kind)} from this machine.`)
  return lines
})
const confirmLabel = computed(() =>
  impact.value?.projects ? `Delete and unbind ${plural(impact.value.projects, 'project')}` : 'Delete connection'
)

onMounted(async () => {
  await nextTick()
  ;(cancelButton.value?.$el as HTMLElement | undefined)?.focus()
})
</script>

<template>
  <AppDialog :label="`Delete ${connection.title}?`" role="alertdialog" panel-class="max-w-[440px]" @dismiss="emit('cancel')">
    <div class="p-5">
      <h2 class="mb-2 text-title font-semibold text-text">Delete {{ connection.title }}?</h2>
      <ul class="flex flex-col gap-1 text-body leading-relaxed text-text-muted" data-testid="delete-consequences">
        <li v-for="line in consequences" :key="line">{{ line }}</li>
      </ul>
    </div>
    <div class="flex justify-end gap-2 border-t border-border bg-canvas px-5 py-3">
      <AppButton ref="cancelButton" variant="secondary" size="sm" class="max-sm:min-h-11" @click="emit('cancel')">Cancel</AppButton>
      <AppButton variant="danger" size="sm" class="max-sm:min-h-11" data-testid="confirm-delete-connection" @click="emit('confirm')">{{ confirmLabel }}</AppButton>
    </div>
  </AppDialog>
</template>
