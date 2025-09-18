type FormValues = { [key: string]: unknown };

export function validate(
  value: unknown,
  rules: string,
  allValues: FormValues,
): string {
  const ruleParts = rules.split("|");

  for (const rule of ruleParts) {
    if (!rule) continue;

    const [ruleName, ...params] = rule.split(":");

    switch (ruleName) {
      case "required":
        if (!value && typeof value !== "boolean")
          return "This field is required.";
        if (typeof value === "boolean" && !value)
          return "You must select this option.";
        break;

      case "min":
        if (String(value).length < Number(params[0]))
          return `Must be at least ${params[0]} characters.`;
        break;

      case "max":
        if (String(value).length > Number(params[0]))
          return `Must not exceed ${params[0]} characters.`;
        break;

      case "between": {
        const len = String(value).length;
        if (len < Number(params[0]) || len > Number(params[1]))
          return `Must be between ${params[0]} and ${params[1]} characters.`;
        break;
      }

      case "password": {
        const passwordRegex = /^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)[\w\W]{8,}$/;
        if (!passwordRegex.test(String(value)))
          return "Password must be at least 8 characters long and include uppercase, lowercase letters, and numbers.";
        break;
      }

      case "same": {
        const otherValue = allValues[params[0]];

        if (value !== otherValue) return `Values must match with ${params[0]}.`;
        break;
      }

      case "different": {
        const otherValue = allValues[params[0]];

        if (value === otherValue)
          return `Values must be different from ${params[0]}.`;
        break;
      }
    }
  }

  return ""; // No errors
}
