import type { DeepWritable } from "ts-essentials";
import type { components } from "~/api_types";

type EditRequest = components["schemas"]["EditRequest"];
type EditProposalState = {
  open: boolean;
  selected: {
    id: string;
    name: string;
  } | null;
  data: DeepWritable<Omit<EditRequest, "privacy_checked" | "token">>;
  locationPicker: {
    open: boolean;
    lat: number;
    lon: number;
  };
};

export const useEditProposal = () => {
  const state = useState<EditProposalState>("editProposal", () => ({
    open: false,
    selected: null,
    data: {
      additional_context: "",
      edits: {},
    },
    locationPicker: {
      open: false,
      lat: 0,
      lon: 0,
    },
  }));

  // Helper function to initialize an edit for a room
  const initializeRoomEdit = (roomId: string) => {
    if (!state.value.data.edits[roomId]) {
      state.value.data.edits[roomId] = {
        coordinate: null,
        image: null,
      };
    }
  };

  // Helper function to suggest a location fix
  const suggestLocationFix = (
    roomId: string,
    roomName: string,
    coordinates: { lat: number; lon: number }
  ) => {
    state.value.data.additional_context = `The location for ${roomName} (${roomId}) is only accurate to building level. I can help provide a more precise location within the building.`;

    initializeRoomEdit(roomId);
    const roomEdit = state.value.data.edits[roomId];
    if (roomEdit) {
      roomEdit.coordinate = {
        lat: coordinates.lat,
        lon: coordinates.lon,
      };
    }

    state.value.open = true;
  };

  // Helper function to suggest an image
  const suggestImage = (roomId: string, roomName: string, context?: string) => {
    const defaultContext = `I would like to suggest a new image for ${roomName} (${roomId}) that would be helpful for students trying to find this room.`;
    state.value.data.additional_context = context || defaultContext;

    // Clear existing edits to start fresh for image suggestion
    state.value.data.edits = {};

    state.value.open = true;
  };

  // Helper function to open edit proposal with custom context
  const openWithContext = (
    context: string,
    roomId?: string,
    coordinates?: { lat: number; lon: number }
  ) => {
    state.value.data.additional_context = context;

    if (roomId && coordinates) {
      initializeRoomEdit(roomId);
      const roomEdit = state.value.data.edits[roomId];
      if (roomEdit) {
        roomEdit.coordinate = {
          lat: coordinates.lat,
          lon: coordinates.lon,
        };
      }
    }

    state.value.open = true;
  };

  // Helper function to reset the state
  const reset = () => {
    state.value.open = false;
    state.value.selected = null;
    state.value.data.additional_context = "";
    state.value.data.edits = {};
  };

  return {
    ...state,
    // Helper functions
    initializeRoomEdit,
    suggestLocationFix,
    suggestImage,
    openWithContext,
    reset,
  };
};
