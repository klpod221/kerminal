<template>
  <form @submit.prevent="handleSubmit">
    <slot></slot>
  </form>
</template>

<script setup lang="ts">
import { provide, computed, reactive } from "vue";
import type { FormField, FormContext } from "../../types/form";

const emit = defineEmits(["submit"]);

// Use a reactive Map to store registered fields
const fields = reactive<Map<string, FormField>>(new Map());

// Computed property to check if the entire form is valid
const isValid = computed(() => {
  return Array.from(fields.values()).every((field) => field.validate() === "");
});

// Function to validate all fields
const validate = (): boolean => {
  let allValid = true;
  // Trigger validation on all fields
  fields.forEach((field) => {
    if (field.validate() !== "") {
      allValid = false;
    }
  });
  return allValid;
};

// Expose validate function and isValid state for parent component to access via template ref
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

// Provide these functions to all child components within <slot>
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
