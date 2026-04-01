import { invoke } from "@tauri-apps/api/core";

export async function evaluateDeckRolesCommand(deckId) {
  try {
    return await invoke("evaluate_deck_roles", { deckId });
  } catch (e) {
    console.error(e);
    throw e;
  }
}
