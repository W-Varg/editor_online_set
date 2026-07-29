import { computed, ref } from "vue";

export type AppTheme = "light" | "dark";

const storedTheme = typeof window !== "undefined" ? window.localStorage.getItem("app-theme") : null;
const prefersDark =
  typeof window !== "undefined" && typeof window.matchMedia === "function"
    ? window.matchMedia("(prefers-color-scheme: dark)").matches
    : false;
const theme = ref<AppTheme>(
  storedTheme === "dark" || (!storedTheme && prefersDark) ? "dark" : "light",
);

function applyTheme(value: AppTheme) {
  if (typeof document === "undefined") return;
  document.documentElement.dataset.theme = value;
}

applyTheme(theme.value);

export function useTheme() {
  const isDark = computed(() => theme.value === "dark");

  function setTheme(value: AppTheme) {
    theme.value = value;
    window.localStorage.setItem("app-theme", value);
    applyTheme(value);
  }

  function toggleTheme() {
    setTheme(isDark.value ? "light" : "dark");
  }

  return { theme, isDark, setTheme, toggleTheme };
}
