import { invoke } from "@tauri-apps/api/core";

/** @typedef {import('./types.js').Deck} Deck */
/** @typedef {import('./types.js').Package} Package */

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

/** @returns {Promise<Deck>} */
export async function getDeckCommand(deckId) {
  try {
    return await invoke("get_deck", { deckId });
  } catch (e) {
    console.error(e);
    throw e;
  }
}

/** @returns {Promise<Deck>} */
export async function addCardToDeckCommand(deckId, name) {
  try {
    return await invoke("add_card_to_deck", { deckId, name });
  } catch (e) {
    console.error(e);
    throw e;
  }
}

/** @returns {Promise<Package[]>} */
export async function getPackagesCommand() {
  try {
    return await invoke("get_packages");
  } catch (e) {
    console.error(e);
    throw e;
  }
}

/** @returns {Promise<Package>} */
export async function getPackageCommand(packageId) {
  try {
    return await invoke("get_package", { packageId });
  } catch (e) {
    console.error(e);
    throw e;
  }
}

/** @returns {Promise<Package>} */
export async function createPackageCommand(name) {
  try {
    return await invoke("create_package", { name });
  } catch (e) {
    console.error(e);
    throw e;
  }
}

/** @returns {Promise<Package>} */
export async function renamePackageCommand(packageId, name) {
  try {
    return await invoke("rename_package", { packageId, name });
  } catch (e) {
    console.error(e);
    throw e;
  }
}

/** @param {Package} packageEntry */
export async function setPackageNameCommand(packageEntry, name) {
  return renamePackageCommand(packageEntry.id, name);
}

/** @returns {Promise<Package>} */
export async function setPackageDescriptionCommand(packageId, description) {
  try {
    return await invoke("set_package_description", { packageId, description });
  } catch (e) {
    console.error(e);
    throw e;
  }
}

export async function deletePackageCommand(packageId) {
  try {
    await invoke("delete_package", { packageId });
  } catch (e) {
    console.error(e);
    throw e;
  }
}

/** @returns {Promise<Package>} */
export async function duplicatePackageCommand(packageId) {
  try {
    return await invoke("duplicate_package", { packageId });
  } catch (e) {
    console.error(e);
    throw e;
  }
}

/** @returns {Promise<Package>} */
export async function addCardToPackageCommand(packageId, cardName) {
  try {
    return await invoke("add_card_to_package", { packageId, cardName });
  } catch (e) {
    console.error(e);
    throw e;
  }
}

/** @returns {Promise<Package>} */
export async function removeCardFromPackageCommand(packageId, cardId) {
  try {
    return await invoke("remove_card_from_package", { packageId, cardId });
  } catch (e) {
    console.error(e);
    throw e;
  }
}

/** @returns {Promise<Deck>} */
export async function addPackageToDeckCommand(deckId, packageId) {
  try {
    return await invoke("add_package_to_deck", { deckId, packageId });
  } catch (e) {
    console.error(e);
    throw e;
  }
}

/** @returns {Promise<import('./types.js').CardSearchSuggestion[]>} */
export async function searchCardSuggestionsCommand(query) {
  try {
    return await invoke("search_card_suggestions", { query });
  } catch (e) {
    console.error(e);
    throw e;
  }
}

/** @returns {Promise<Deck>} */
export async function removeCardFromDeckCommand(deckId, cardId) {
  try {
    return await invoke("remove_card_from_deck", { deckId, cardId });
  } catch (e) {
    console.error(e);
    throw e;
  }
}

/** @returns {Promise<Deck>} */
export async function setDeckCommanderCommand(deckId, cardId) {
  try {
    return await invoke("set_deck_commander", { deckId, cardId });
  } catch (e) {
    console.error(e);
    throw e;
  }
}

/** @returns {Promise<Deck>} */
export async function setDeckPartnerCommand(deckId, cardId) {
  try {
    return await invoke("set_deck_partner", { deckId, cardId });
  } catch (e) {
    console.error(e);
    throw e;
  }
}

/**@returns {Promise<Deck>} */
export async function removeDeckPartnerCommand(deckId, cardId){
  try {
    return await invoke("remove_deck_partner", { deckId, cardId });
  } catch (e){
    console.error(e);
    throw e;
  }
}



/** @returns {Promise<Deck>} */
export async function removeDeckCommanderCommand(deckId) {
  try {
    return await invoke("remove_deck_commander", { deckId });
  } catch (e) {
    console.error(e);
    throw e;
  }
}

/** @returns {Promise<Deck>} */
export async function deleteDeckCommanderCommand(deckId) {
  try {
    return await invoke("delete_deck_commander", { deckId });
  } catch (e) {
    console.error(e);
    throw e;
  }
}
