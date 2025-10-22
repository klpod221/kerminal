<template>
  <form @submit.prevent="handleSubmit">
    <slot></slot>
  </form>
</template>

<script setup lang="ts">
import { provide, computed, reactive } from "vue";
import type { FormField, FormContext } from "../../types/form";

const emit = defineEmits(["submit"]);

const fields = reactive<Map<string, FormField>>(new Map());

const isValid = computed(() => {
  return Array.from(fields.values()).every((field) => field.validate() === "");
});

const validate = (): boolean => {
  let allValid = true;
  fields.forEach((field) => {
    if (field.validate() !== "") {
      allValid = false;
    }
  });
  return allValid;
};

defineExpose({
  validate,
  isValid,
});

const register = (field: FormField): void => {
  fields.set(field.id, field);
};

const unregister = (id: string): void => {
  fields.delete(id);
};

const getFieldValue = (id: string): unknown => {
  const field = fields.get(id);
  return field ? field.value : undefined;
};

const getAllFieldValues = (): Record<string, unknown> => {
  const allValues: Record<string, unknown> = {};
  fields.forEach((_field, id) => {
    allValues[id] = getFieldValue(id);
  });
  return allValues;
};

provide<FormContext>("form-context", {
  register,
  unregister,
  getFieldValue,
  getAllFieldValues,
});

const handleSubmit = (): void => {
  if (validate()) {
    const formData = getAllFieldValues();
    emit("submit", formData);
  }
};
</script>
