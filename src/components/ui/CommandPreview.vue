<template>
  <div
    class="command-preview-wrapper"
    :style="{ maxHeight: props.maxHeight }"
    @click.stop
  >
    <SimpleCodeEditor
      id="command-preview-editor"
      :model-value="props.command"
      language="shell"
      :fit-content="true"
      :read-only="true"
      class="command-preview-editor"
    />
  </div>
</template>

<script setup lang="ts">
import SimpleCodeEditor from "./SimpleCodeEditor.vue";

interface Props {
  command: string;
  containerClass?: string;
  maxHeight?: string;
}

const props = withDefaults(defineProps<Props>(), {
  containerClass: "",
  maxHeight: "120px",
});
</script>

<style scoped>
.command-preview-wrapper {
  width: 100%;
  overflow-y: auto;
  border-radius: 0.5rem;
  /* Firefox scrollbar */
  scrollbar-width: thin;
  scrollbar-color: rgba(156, 163, 175, 0.3) transparent;
}

.command-preview-wrapper::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

.command-preview-wrapper::-webkit-scrollbar-track {
  background: transparent;
}

.command-preview-wrapper::-webkit-scrollbar-thumb {
  background-color: rgba(156, 163, 175, 0.3);
  border-radius: 3px;
  transition: background-color 0.2s;
}

.command-preview-wrapper::-webkit-scrollbar-thumb:hover {
  background-color: rgba(156, 163, 175, 0.5);
}

:deep(.simple-code-editor) {
  background: linear-gradient(
    135deg,
    rgba(0, 0, 0, 0.4) 0%,
    rgba(0, 0, 0, 0.2) 100%
  );
  backdrop-filter: blur(4px);
}
</style>
