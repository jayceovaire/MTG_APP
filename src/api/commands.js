import { invoke } from "@tauri-apps/api/core";

export {
  createDeckCommand,
  renameDeckCommand,
  setDeckNameCommand,
  deleteDeckCommand,
  duplicateDeckCommand,
  getDecksCommand,
  getDeckCommand,
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
} from "./collectionCommands.js";

export async function runTestCommand(greeting) {
  return await invoke("test_command", { greeting });
}
