import { createRouter, createWebHashHistory } from 'vue-router'

import DeckEditor from "../views/DeckEditor.vue";
import HomeView from "../views/HomeView.vue";
import DeckLibrary from "../views/DeckLibrary.vue";
import Collection from "../views/Collection.vue";
import PowerCalculator from "../views/PowerCalculator.vue";
import Roast from "../views/Roast.vue";

const routes = [
    { path: "/", component: HomeView },
    { path: "/deck-editor/:deckId", component: DeckEditor, props: true },
    { path: "/deck-library", component: DeckLibrary },
    {path: "/power-calculator", component: PowerCalculator},
    {path: "/collection", component: Collection},
    {path: "/roast", component: Roast},

]

export const router = createRouter({
    history: createWebHashHistory(),
    routes
})
