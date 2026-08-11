<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import AppField from '@/components/ui/AppField.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import AppTextarea from '@/components/ui/AppTextarea.vue'
import { renderMarkdown } from '@/common/utils/markdown'
import { DOCUMENT_KINDS } from '@/model/catalog'
import type { RekallDocument } from '@/model/catalog'
import type { DocumentId } from '@/model/branded'

const props = defineProps<{ documents: readonly RekallDocument[] }>()

const emit = defineEmits<{
  create: [input: { title: string; kind: string; bodyMarkdown: string }]
  save: [id: DocumentId, input: { title: string; kind: string; bodyMarkdown: string }]
  remove: [id: DocumentId]
}>()

const KIND_OPTIONS = DOCUMENT_KINDS.map((kind) => ({ value: kind, label: kind }))

// The preview below uses v-html, which the linter is disabled for in this template.
// renderMarkdown escapes its input before adding any tags, so no part of a document body can
// reach the page as markup.

const openId = ref<DocumentId | null>(null)
const isEditing = ref(false)
const isCreating = ref(false)
const draft = ref({ title: '', kind: 'notes', bodyMarkdown: '' })
const editable = ref({ title: '', kind: 'notes', bodyMarkdown: '' })

const open = computed(() => props.documents.find((document) => document.id === openId.value) ?? null)

// Opening the first document by default: the panel is empty otherwise, and there is almost
// always exactly one that matters.
watch(
  () => props.documents,
  (documents) => {
    if (openId.value && documents.some((document) => document.id === openId.value)) return
    openId.value = documents[0]?.id ?? null
  },
  { immediate: true }
)

watch(open, (document) => {
  if (!document) return
  editable.value = { title: document.title, kind: document.kind, bodyMarkdown: document.bodyMarkdown }
  isEditing.value = false
})

function submitNew(): void {
  emit('create', { ...draft.value })
  isCreating.value = false
  draft.value = { title: '', kind: 'notes', bodyMarkdown: '' }
}
</script>

<template>
  <!-- eslint-disable vue/no-v-html -->
  <section class="space-y-3.5">
    <div class="flex items-center gap-3">
      <h2 class="text-[16px] font-semibold text-text">Documents</h2>
      <div class="flex-1" />
      <AppButton size="sm" @click="isCreating = !isCreating">
        {{ isCreating ? 'Cancel' : 'Add document' }}
      </AppButton>
    </div>

    <AppCard v-if="isCreating">
      <div class="grid gap-x-4 sm:grid-cols-2">
        <AppField v-slot="{ fieldId, describedBy }" label="Title" required>
          <AppInput
        :id="fieldId"
        v-model="draft.title" :described-by="describedBy" placeholder="CONTEXT.md" />
        </AppField>
        <AppField v-slot="{ fieldId, describedBy }" label="Kind">
          <AppSelect
        :id="fieldId"
        v-model="draft.kind" :described-by="describedBy" :options="KIND_OPTIONS" />
        </AppField>
      </div>
      <AppField v-slot="{ fieldId, describedBy }" label="Content">
        <AppTextarea
        :id="fieldId"
        v-model="draft.bodyMarkdown" :described-by="describedBy" placeholder="# Contesto" :rows="8" />
      </AppField>
      <AppButton variant="primary" :disabled="!draft.title" @click="submitNew">Add</AppButton>
    </AppCard>

    <div v-if="documents.length" class="flex flex-wrap gap-2">
      <button
        v-for="document in documents"
        :key="document.id"
        data-testid="document-tab"
        class="focus-ring inline-flex items-center gap-2 rounded-[var(--radius-control)] border px-3 py-1.5 text-[12.5px] transition-colors"
        :class="
          openId === document.id
            ? 'border-accent bg-accent-soft text-accent'
            : 'border-border bg-surface text-text-muted hover:border-border-strong hover:text-text'
        "
        @click="openId = document.id"
      >
        {{ document.title }}
        <AppBadge>{{ document.kind }}</AppBadge>
      </button>
    </div>

    <AppEmptyState
      v-else-if="!isCreating"
      title="No documents"
      description="This is where the markdown that used to live in CONTEXT.md goes."
    />

    <AppCard v-if="open">
      <div class="flex flex-wrap items-center gap-2">
        <div class="w-full max-w-[280px]">
          <AppInput v-model="editable.title" />
        </div>
        <div class="w-[150px]">
          <AppSelect v-model="editable.kind" :options="KIND_OPTIONS" />
        </div>
        <div class="flex-1" />
        <AppButton variant="ghost" size="sm" @click="isEditing = !isEditing">
          {{ isEditing ? 'Preview' : 'Edit' }}
        </AppButton>
        <AppButton variant="danger" size="sm" @click="emit('remove', open.id)">Delete</AppButton>
        <AppButton variant="primary" size="sm" @click="emit('save', open.id, { ...editable })">
          Save
        </AppButton>
      </div>

      <hr class="my-4 border-border" />

      <AppTextarea v-if="isEditing" v-model="editable.bodyMarkdown" :rows="22" />
      <div
        v-else
        data-testid="document-preview"
        class="text-[13.5px] leading-relaxed text-text-muted"
        v-html="renderMarkdown(editable.bodyMarkdown)"
      />
    </AppCard>
  </section>
</template>
