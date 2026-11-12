<script setup lang="ts">
/** The sentence that travels with a company or a project into every context it is loaded in. */
const description = defineModel<string>({ required: true })
</script>

<template>
  <label for="record-description" class="mb-1.5 mt-5 block eyebrow text-caption">Description</label>
  <textarea
    id="record-description"
    v-model="description"
    data-testid="record-description"
    rows="3"
    class="field text-text w-full resize-y rounded-control p-3 text-body leading-relaxed"
    placeholder="What it is, in a sentence. This travels with the record into every context."
  />
</template>
