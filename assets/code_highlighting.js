// Arborium's browser plugin loader uses dynamic JS imports to instantiate WASM.
// The pinned module also pins its host and grammar plugins to 2.18.1.
export async function highlightCode(language, source) {
  try {
    const arborium = await import("https://cdn.jsdelivr.net/npm/@arborium/arborium@2.18.1/dist/arborium.js");
    const resolved = language
      ? arborium.normalizeLanguage(language)
      : arborium.detectLanguage(source);
    if (!resolved || !(await arborium.isLanguageAvailable(resolved))) return null;
    return await arborium.highlight(resolved, source);
  } catch (error) {
    console.warn("Code highlighting unavailable", error);
    return null;
  }
}
