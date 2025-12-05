<template>
  <div class="space-y-2">
    <div
      v-for="(item, index) in localVars"
      :key="index"
      class="flex gap-2 mb-2"
    >
      <Input
        :id="`env-key-${index}`"
        v-model="item.key"
        placeholder="KEY"
        class="flex-1"
        @update:model-value="emitUpdate"
      />
      <Input
        :id="`env-val-${index}`"
        v-model="item.value"
        placeholder="VALUE"
        class="flex-1"
        @update:model-value="emitUpdate"
      />
      <Button
        type="button"
        variant="ghost"
        :icon="Trash2"
        @click="removeEnvVar(index)"
      />
    </div>
    <Button
      type="button"
      variant="outline"
      size="sm"
      :icon="Plus"
      @click="addEnvVar"
    >
      Add Variable
    </Button>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import Input from "./Input.vue";
import Button from "./Button.vue";
import { Plus, Trash2 } from "lucide-vue-next";

const props = defineProps<{
  modelValue?: Record<string, string>;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", value: Record<string, string>): void;
}>();

const localVars = ref<{ key: string; value: string }[]>([]);

const initLocalVars = () => {
  if (props.modelValue) {
    localVars.value = Object.entries(props.modelValue).map(([key, value]) => ({
      key,
      value,
    }));
  } else {
    localVars.value = [];
  }
};

watch(
  () => props.modelValue,
  (newValue) => {
    // Only update if the object structure is different to avoid cursor jumps
    // This is a simple check, might need refinement if bidirectional binding causes issues
    const currentObj = localVars.value.reduce(
      (acc, item) => {
        if (item.key) acc[item.key] = item.value;
        return acc;
      },
      {} as Record<string, string>,
    );

    if (JSON.stringify(newValue) !== JSON.stringify(currentObj)) {
      initLocalVars();
    }
  },
  { deep: true },
);

onMounted(() => {
  initLocalVars();
});

const addEnvVar = () => {
  localVars.value.push({ key: "", value: "" });
  emitUpdate();
};

const removeEnvVar = (index: number) => {
  localVars.value.splice(index, 1);
  emitUpdate();
};

const emitUpdate = () => {
  const env: Record<string, string> = {};
  localVars.value.forEach((item) => {
    if (item.key) {
      env[item.key] = item.value;
    }
  });
  emit("update:modelValue", env);
};
</script>
