// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

import { Ref } from "vue";

/**
 * Props interface for Form component
 */
export interface FormContext {
  register: (field: FormField) => void;
  unregister: (id: string) => void;
  getFieldValue: (id: string) => unknown;
  getAllFieldValues: () => { [key: string]: unknown };
}

/**
 * Interface representing a form field
 */
export interface FormField {
  id: string;
  value: Ref<unknown>;
  validate: () => string;
}
