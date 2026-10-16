<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppDialog from '@/components/ui/AppDialog.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import {
  NEW_MEMORY_TYPES,
  groupName,
  newResource,
  type MemoryGroup,
  type NewMemoryType,
  type ResourceSection
} from '@/features/claude-resources/model/claude-resources'

const props = defineProps<{
  section: ResourceSection
  groups: readonly MemoryGroup[]
  /** The group whose memory is open, offered first. */
  initialGroupId: string
  sending: boolean
}>()

const emit = defineEmits<{ cancel: []; create: [path: string, body: string] }>()

const SINGULAR: Readonly<Record<ResourceSection, string>> = { memories: 'memory', skills: 'skill', commands: 'command', agents: 'agent' }

const name = ref('')
const groupId = ref(props.initialGroupId)
const memoryType = ref<NewMemoryType>('feedback')
const field = ref<HTMLInputElement | null>(null)

const groupOptions = computed(() => [
  { value: 'global', label: 'Everywhere' },
  ...props.groups.filter((group) => !group.global).map((group) => ({ value: group.id, label: groupName(group) }))
])
const typeOptions = NEW_MEMORY_TYPES.map((type) => ({ value: type, label: type }))
const draft = computed(() => newResource(props.section, name.value, { groupId: groupId.value, memoryType: memoryType.value }))

function submit(): void {
  if (draft.value && !props.sending) emit('create', draft.value.path, draft.value.body)
}

onMounted(() => field.value?.focus())
</script>

<template>
  <AppDialog :label="`New ${SINGULAR[section]}`" panel-class="max-w-[520px]" :dismissible="!sending" @dismiss="emit('cancel')">
    <form @submit.prevent="submit">
      <header class="flex items-center gap-3 border-b border-border px-5 py-4">
        <span class="min-w-0 flex-1 text-title font-semibold tracking-[-0.01em] text-text">New {{ SINGULAR[section] }}</span>
        <button type="button" class="focus-ring grid size-7 shrink-0 place-items-center rounded-control text-text-subtle transition-colors hover:bg-surface-raised hover:text-text" aria-label="Close" :disabled="sending" @click="emit('cancel')">
          <CloseGlyph />
        </button>
      </header>

      <div class="flex flex-col gap-4 px-5 py-4">
        <label class="block">
          <span class="eyebrow mb-1.5 block text-caption">Name</span>
          <input ref="field" v-model="name" class="field h-9 w-full rounded-control px-3 text-body text-text" placeholder="a short, lowercase name" autocomplete="off" spellcheck="false" data-testid="new-resource-name" />
        </label>

        <div v-if="section === 'memories'" class="grid grid-cols-2 gap-3">
          <label class="block min-w-0">
            <span class="eyebrow mb-1.5 block text-caption">For</span>
            <AppSelect v-model="groupId" :options="groupOptions" />
          </label>
          <label class="block min-w-0">
            <span class="eyebrow mb-1.5 block text-caption">Type</span>
            <AppSelect :model-value="memoryType" :options="typeOptions" @update:model-value="memoryType = $event as NewMemoryType" />
          </label>
        </div>

        <p class="truncate font-mono text-caption text-text-subtle" data-testid="new-resource-path">
          {{ draft ? `~/.claude/${draft.path}` : 'The file is named after what you type.' }}
        </p>
      </div>

      <footer class="flex justify-end gap-2 border-t border-border bg-canvas px-5 py-3">
        <AppButton variant="ghost" size="sm" :disabled="sending" @click="emit('cancel')">Cancel</AppButton>
        <AppButton variant="primary" size="sm" type="submit" :disabled="!draft" :loading="sending" data-testid="new-resource-submit">Create</AppButton>
      </footer>
    </form>
  </AppDialog>
</template>
