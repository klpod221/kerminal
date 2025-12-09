<template>
  <div
    class="command-preview-container"
    :class="props.containerClass"
    :style="{ maxHeight: props.maxHeight }"
    @click.stop
  >
    <SyntaxHighlight
      :code="props.command"
      language="shell"
      class="command-preview-code"
    />
  </div>
</template>

<script setup lang="ts">
import SyntaxHighlight from "./SyntaxHighlight.vue";

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
.command-preview-container {
  position: relative;
  border-radius: 0.5rem;
  overflow: hidden;
  border: 1px solid rgba(75, 85, 99, 0.4);
  background: linear-gradient(
    135deg,
    rgba(0, 0, 0, 0.4) 0%,
    rgba(0, 0, 0, 0.2) 100%
  );
  backdrop-filter: blur(4px);
  overflow-y: auto;
  transition: all 0.2s;
  /* Firefox scrollbar */
  scrollbar-width: thin;
  scrollbar-color: rgba(156, 163, 175, 0.3) transparent;
}

.group:hover .command-preview-container {
  border-color: rgba(75, 85, 99, 0.6);
  background: linear-gradient(
    135deg,
    rgba(0, 0, 0, 0.5) 0%,
    rgba(0, 0, 0, 0.3) 100%
  );
}

.command-preview-container::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

.command-preview-container::-webkit-scrollbar-track {
  background: transparent;
}

.command-preview-container::-webkit-scrollbar-thumb {
  background-color: rgba(156, 163, 175, 0.3);
  border-radius: 3px;
  transition: background-color 0.2s;
}

.command-preview-container::-webkit-scrollbar-thumb:hover {
  background-color: rgba(156, 163, 175, 0.5);
}

.command-preview-code {
  margin: 0;
  padding: 0.375rem 0.625rem;
  font-size: 0.8125rem;
  line-height: 1.6;
  font-family: "Fira Code", "Monaco", "Menlo", "Ubuntu Mono", monospace;
}
</style>
