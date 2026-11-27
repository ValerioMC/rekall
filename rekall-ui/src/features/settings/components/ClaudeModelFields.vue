<script setup lang="ts">
/**
 * The Claude Code model and effort rows of the provider column. A read of the catalog only changes
 * what is offered: a saved choice changes when the person picks one.
 */
import { computed, nextTick, ref } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import ProviderField from '@/features/settings/components/ProviderField.vue'
import { preferredEffort, preferredModel, setPreferredEffort, setPreferredModel } from '@/common/config/claude-launch'
import {
  claudeCatalogEntry,
  claudeEffortOptions,
  claudeEffortSupport,
  claudeModelName,
  claudeModelOptions,
  offersClaudeEffort
} from '@/features/claude/lib/claude-model-choices'
import { CLAUDE_EFFORT_CHOICES, claudeEffortChoiceLabel } from '@/features/claude/model/claude'
import { useClaudeModelsStore } from '@/features/claude/store/claudeModels.store'

const catalog = useClaudeModelsStore()
const { models, reading, failed } = storeToRefs(catalog)

const model = ref(preferredModel())
const effort = ref(preferredEffort())

const modelOptions = computed(() => claudeModelOptions(models.value, model.value))
const entry = computed(() => claudeCatalogEntry(models.value, model.value))
const modelName = computed(() => claudeModelName(models.value, model.value))
const resolvedModel = computed(() => {
  const resolved = entry.value?.resolvedModel
  return resolved && resolved !== model.value ? resolved : null
})
const unlisted = computed(() => model.value !== 'default' && models.value !== null && !entry.value)

const support = computed(() => claudeEffortSupport(entry.value))
const effortOptions = computed(() => claudeEffortOptions(support.value, effort.value))
const effortOffered = computed(() => offersClaudeEffort(support.value, effort.value))
const effortUnreported = computed(() => model.value !== 'default' && entry.value !== null && support.value.kind === 'unreported')

function chooseModel(value: string): void {
  model.value = value
  setPreferredModel(value)
  const chosen = claudeEffortSupport(claudeCatalogEntry(models.value, value))
  if (!offersClaudeEffort(chosen, effort.value)) chooseEffort('default')
}

function chooseEffort(value: string): void {
  const choice = CLAUDE_EFFORT_CHOICES.find((candidate) => candidate === value)
  if (!choice) return
  effort.value = choice
  setPreferredEffort(choice)
}

async function retry(): Promise<void> {
  await catalog.load()
  if (failed.value) return
  await nextTick()
  document.getElementById('claude-model')?.focus()
}
</script>

<template>
  <ProviderField label="Model" field-id="claude-model">
    <template #control="{ describedBy }">
      <AppSelect
        id="claude-model"
        :model-value="model"
        :options="modelOptions"
        :described-by="describedBy"
        data-testid="claude-model"
        @update:model-value="chooseModel"
      />
    </template>
    <div v-if="failed" class="flex flex-wrap items-center gap-2" data-testid="claude-models-failed">
      <p role="status">
        <template v-if="models">Couldn’t refresh the models. This is the last list Claude Code answered.</template>
        <template v-else>Couldn’t read the models from Claude Code. These are the family names.</template>
      </p>
      <AppButton size="sm" variant="secondary" :loading="reading" data-testid="claude-models-retry" @click="retry">
        Retry
      </AppButton>
    </div>
    <p v-else-if="reading" role="status">Reading the models your Claude Code offers…</p>
    <p v-else-if="model === 'default'">Adds no <code class="font-mono text-text-muted">--model</code>: your own Claude Code setting applies.</p>
    <p v-else>
      {{ unlisted ? 'Claude Code doesn’t list this model. New sessions still get' : 'Adds' }}
      <code class="break-all font-mono text-text-muted">--model {{ model }}</code><template v-if="resolvedModel">, today
      <code class="break-all font-mono text-text-muted">{{ resolvedModel }}</code></template>.
    </p>
  </ProviderField>

  <ProviderField label="Reasoning effort" field-id="claude-effort">
    <template #control="{ describedBy }">
      <AppSelect
        id="claude-effort"
        :model-value="effort"
        :options="effortOptions"
        :described-by="describedBy"
        :disabled="effortOptions.length < 2"
        data-testid="claude-effort"
        @update:model-value="chooseEffort"
      />
    </template>
    <p v-if="!effortOffered">
      {{ modelName }} doesn’t list {{ claudeEffortChoiceLabel(effort) }}. New sessions still get it until you choose another.
    </p>
    <p v-else-if="support.kind === 'none'">{{ modelName }} takes no reasoning effort.</p>
    <p v-else-if="effortUnreported">Claude Code doesn’t say which levels {{ modelName }} takes.</p>
    <p v-else>Adds <code class="font-mono text-text-muted">--effort</code>. A higher level thinks longer and spends more.</p>
  </ProviderField>
</template>
