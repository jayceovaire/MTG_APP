import { invoke } from "@tauri-apps/api/core";
import { ref } from "vue";

export {
  createDeckCommand,
  renameDeckCommand,
  setDeckNameCommand,
  deleteDeckCommand,
  duplicateDeckCommand,
  getDecksCommand,
} from "./deckCommands.js";

export {
  createCollectionCardCommand,
  duplicateCollectionCardCommand,
  removeCollectionCardCommand,
} from "./collectionCommands.js";

// v TEST COMMAND v
export const greeting = ref("");

export async function runCommand(inputGreeting) {
  try {
    const result = await invoke("test_command", { greeting: inputGreeting });
    greeting.value = result;
    console.log(result);
    return result;
  } catch (e) {
    const errorMessage = String(e);
    greeting.value = errorMessage;
    console.error(e);
    return errorMessage;
  }
}
// ^ TEST COMMAND ^
