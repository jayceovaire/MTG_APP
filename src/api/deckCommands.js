import { invoke } from "@tauri-apps/api/core";

/** @typedef {import('./types.js').Deck} Deck */

/** @returns {Promise<Deck>} */
export async function createDeckCommand(name) {
  try {
    return await invoke("create_deck", { name });
  } catch (e) {
    console.error(e);
    throw e;
  }
}

/** @returns {Promise<Deck>} */
export async function renameDeckCommand(deckId, name) {
  try {
    return await invoke("rename_deck", { deckId, name });
  } catch (e) {
    console.error(e);
    throw e;
  }
}

/** @param {Deck} deck */
export async function setDeckNameCommand(deck, name) {
  return renameDeckCommand(deck.id, name);
}

export async function deleteDeckCommand(deckId) {
  try {
    await invoke("delete_deck", { deckId });
  } catch (e) {
    console.error(e);
    throw e;
  }
}

/** @returns {Promise<Deck>} */
export async function duplicateDeckCommand(deckId) {
  try {
    return await invoke("duplicate_deck", { deckId });
  } catch (e) {
    console.error(e);
    throw e;
  }
}

/** @returns {Promise<Deck[]>} */
export async function getDecksCommand() {
  try {
    return await invoke("get_decks");
  } catch (e) {
    console.error(e);
    throw e;
  }
}
