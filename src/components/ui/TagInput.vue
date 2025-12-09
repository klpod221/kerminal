<template>
  <div class="space-y-0.5">
    <!-- Label -->
    <label
      v-if="label"
      :for="inputId"
      class="block text-sm font-medium text-gray-300"
    >
      {{ label }}
      <span
        v-if="props.rules && props.rules.includes('required')"
        class="text-red-400"
        >*</span
      >
    </label>

    <!-- Tags Display -->
    <div v-if="tags.length > 0" class="flex flex-wrap gap-2 mb-2">
      <span
        v-for="(tag, index) in tags"
        :key="index"
        class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-900/30 text-blue-300"
      >
        {{ tag }}
        <button
          type="button"
          class="ml-1 hover:text-blue-200 transition-colors"
          :disabled="disabled || readonly"
          @click="removeTag(index)"
        >
          ×
        </button>
      </span>
    </div>

    <!-- Input Area -->
    <div class="flex gap-2">
      <input
        :id="inputId"
        ref="inputRef"
        v-model="currentInput"
        type="text"
        :placeholder="placeholder"
        :disabled="disabled"
        :readonly="readonly"
        :class="[
          'flex-1 rounded-lg border transition-all duration-200',
          'focus:outline-none',
          'disabled:opacity-50 disabled:cursor-not-allowed',
          'readonly:bg-gray-700 readonly:cursor-default',
          sizeClasses,
          stateClasses,
        ]"
        @keydown.enter.prevent="addTag"
        @blur="handleBlur"
        @focus="handleFocus"
      />
      <Button
        type="button"
        variant="outline"
        :size="size"
        :disabled="disabled || !currentInput.trim()"
        @click="addTag"
      >
        Add
      </Button>
    </div>

    <div v-if="helper" class="min-h-5">
      <!-- Helper text (only show if no error) -->
      <p v-if="helperText && !errorMessage" class="text-xs text-gray-400">
        {{ helperText }}
      </p>

      <!-- Error message -->
      <p v-if="errorMessage" class="text-xs text-red-400 flex items-center">
        <TriangleAlert class="mr-1" :size="12" />
        {{ errorMessage }}
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { TriangleAlert } from "lucide-vue-next";
import Button from "./Button.vue";
import { useFormField } from "../../composables/useFormField";
import { useFormStyles } from "../../composables/useFormStyles";

interface TagInputProps {
  id: string;
  modelValue?: string[];
  label?: string;
  placeholder?: string;
  rules?: string;
  helperText?: string;
  errorMessage?: string;
  size?: "sm" | "md" | "lg";
  disabled?: boolean;
  readonly?: boolean;
  helper?: boolean;
  allowDuplicates?: boolean;
}

const props = withDefaults(defineProps<TagInputProps>(), {
  size: "md",
  disabled: false,
  readonly: false,
  helper: true,
  allowDuplicates: false,
  modelValue: () => [],
});

const emit = defineEmits(["update:modelValue", "blur", "focus"]);

// Use composables for shared logic
const { errorMessage, touched, inputId, validate, handleBlur, handleFocus } =
  useFormField(props, emit);

const { sizeClasses, stateClasses } = useFormStyles(props);

// Component-specific state
const inputRef = ref<HTMLInputElement>();
const currentInput = ref("");

const tags = computed(() => props.modelValue || []);

const addTag = () => {
  const tag = currentInput.value.trim();

  if (!tag) return;

  const currentTags = [...tags.value];

  if (!props.allowDuplicates && currentTags.includes(tag)) {
    currentInput.value = "";
    return;
  }

  currentTags.push(tag);
  emit("update:modelValue", currentTags);
  currentInput.value = "";

  if (touched.value) {
    validate();
  }
};

const removeTag = (index: number) => {
  if (props.disabled || props.readonly) return;

  const currentTags = [...tags.value];
  currentTags.splice(index, 1);
  emit("update:modelValue", currentTags);

  if (touched.value) {
    validate();
  }
};

defineExpose({
  focus: () => inputRef.value?.focus(),
  blur: () => inputRef.value?.blur(),
  clear: () => {
    currentInput.value = "";
    emit("update:modelValue", []);
  },
});
</script>
