import { invoke } from "@tauri-apps/api/core";

export {
  createDeckCommand,
  renameDeckCommand,
  setDeckNameCommand,
  deleteDeckCommand,
  duplicateDeckCommand,
  getDecksCommand,
  getDeckCommand,
  getCardCommand,
  addCardToDeckCommand,
  searchCardSuggestionsCommand,
  removeCardFromDeckCommand,
  setDeckCommanderCommand,
  setDeckPartnerCommand,
  removeDeckPartnerCommand,
  removeDeckCommanderCommand,
  deleteDeckCommanderCommand,
} from "./deckCommands.js";

export {
  createCollectionCardCommand,
  duplicateCollectionCardCommand,
  removeCollectionCardCommand,
  getRandomCardCommand,
} from "./collectionCommands.js";

export async function runTestCommand(greeting) {
  return await invoke("test_command", { greeting });
}

export {
  getMostRecentCachedImageCommand,
  fetchCardImagesCommand,
} from "./imageCommands.js";

export {
  isSidecarRunningCommand,
  getSidecarIndexCommand,
  getSidecarVariantsCommand,
  submitComboToSidecarCommand,
  killSidecarCommand,
} from "./sidecarCommands.js";

export {
  checkForUpdatesCommand,
  installUpdateCommand,
} from "./settingsCommands.js";
