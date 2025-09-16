<template>
  <form @submit.prevent="handleSubmit">
    <slot></slot>
  </form>
</template>

<script setup lang="ts">
import { provide, computed, reactive } from "vue";
import type { Ref } from "vue";
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
  if (
    field &&
    field.value &&
    typeof field.value === "object" &&
    "value" in field.value
  ) {
    return (field.value as Ref<unknown>).value;
  }
  return undefined;
};

const getAllFieldValues = (): Record<string, unknown> => {
  const allValues: Record<string, unknown> = {};
  fields.forEach((field, id) => {
    if (
      field.value &&
      typeof field.value === "object" &&
      "value" in field.value
    ) {
      allValues[id] = (field.value as Ref<unknown>).value;
    } else {
      allValues[id] = undefined;
    }
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
    // Thu thập tất cả dữ liệu từ các child
    const formData: Record<string, unknown> = {};
    fields.forEach((field, id) => {
      if (
        field.value &&
        typeof field.value === "object" &&
        "value" in field.value
      ) {
        formData[id] = (field.value as Ref<unknown>).value;
      } else {
        formData[id] = undefined;
      }
    });
    emit("submit", formData);
  }
};
</script>
