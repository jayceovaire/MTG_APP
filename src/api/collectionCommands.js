import { invoke } from "@tauri-apps/api/core";
import { fetchCardImagesCommand } from "./imageCommands.js";

/** @typedef {import('./types.js').Card} Card */

/** @returns {Promise<Card>} */
export async function createCollectionCardCommand(name = "Unnamed Card") {
  try {
    const card = await invoke("add_card_to_collection", { name });
    // Trigger image fetching in background
    fetchCardImagesCommand().catch(console.error);
    return card;
  } catch (e) {
    console.error(e);
    throw e;
  }
}

/** @returns {Promise<Card>} */
export async function duplicateCollectionCardCommand(cardId) {
  try {
    const card = await invoke("duplicate_collection_card", { cardId });
    fetchCardImagesCommand().catch(console.error);
    return card;
  } catch (e) {
    console.error(e);
    throw e;
  }
}

/** 
 * @param {[number, string][]} cards - Array of [quantity, cardName]
 * @returns {Promise<Card[]>} 
 */
export async function bulkAddCardsToCollectionCommand(cards) {
  try {
    const addedCards = await invoke("bulk_add_cards_to_collection", { cards });
    fetchCardImagesCommand().catch(console.error);
    return addedCards;
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

/** @returns {Promise<Card|null>} */
export async function getRandomCardCommand() {
  try {
    return await invoke("get_random_card");
  } catch (e) {
    console.error(e);
    throw e;
  }
}
