import { defineStore } from 'pinia'

export enum selectedMap {
    roomfinder,
    interactive
}

export const useDetailsStore = defineStore({
    id: "details",
    state: () => ({
        map: {
            // "interactive" is default, because it should show a loading indication.
            selected: selectedMap.interactive,
            roomfinder: {
                selected_id: null, // Map id
                selected_index: null, // Index in the 'available' list
                x: -1023 - 10, // Outside in top left corner
                y: -1023 - 10,
                width: 400,
                height: 300,
            },
        },
        buildings_overview: {
            expanded: false,
        },
        rooms_overview: {
            expanded: false,
            selected: null,
            filter: "",
        },
    }),
    actions: {
        reset() {
            this.map = {
                selected: selectedMap.interactive,
                roomfinder: {
                    selected_id: null, // Map id
                    selected_index: null, // Index in the 'available' list
                    x: -1023 - 10, // Outside in top left corner
                    y: -1023 - 10,
                    width: 400,
                    height: 300,
                },
            };
            this.buildings_overview = {
                expanded: false,
            };
            this.rooms_overview = {
                expanded: false,
                selected: null,
                filter: "",
            };
        },
    },
});
