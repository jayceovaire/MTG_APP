import { invoke } from "@tauri-apps/api/core";

/** @typedef {import('./types.js').Card} Card */

/** @returns {Promise<Card>} */
export async function createCollectionCardCommand(name = "Unnamed Card") {
  try {
    return await invoke("add_card_to_collection", { name });
  } catch (e) {
    console.error(e);
    throw e;
  }
}

/** @returns {Promise<Card>} */
export async function duplicateCollectionCardCommand(cardId) {
  try {
    return await invoke("duplicate_collection_card", { cardId });
  } catch (e) {
    console.error(e);
    throw e;
  }
}

export async function removeCollectionCardCommand(cardId) {
  try {
    await invoke("remove_collection_card", { cardId });
  } catch (e) {
    console.error(e);
    throw e;
  }
}

/** @returns {Promise<Card[]>} */
export async function getCollectionCommand() {
  try {
    return await invoke("get_collection");
  } catch (e) {
    console.error(e);
    throw e;
  }
}

/** @returns {Promise<Card>} */
export async function setCollectionCardFavoriteCommand(cardId, favorite) {
  try {
    return await invoke("set_collection_card_favorite", { cardId, favorite });
  } catch (e) {
    console.error(e);
    throw e;
  }
}
