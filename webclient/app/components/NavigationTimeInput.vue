<script setup lang="ts">
import { computed } from "vue";

type TimeSelection = { type: "depart_at" | "arrive_by"; time: Date };
const timeSelection = defineModel<TimeSelection | undefined>("timeSelection");

// Helper to format time to HH:mm string
const formatTimeForInput = (date: Date): string => {
  const pad = (num: number) => num.toString().padStart(2, "0");
  const hours = pad(date.getHours());
  const minutes = pad(date.getMinutes());
  return `${hours}:${minutes}`;
};

// Helper to format date to YYYY-MM-DD string
const formatDateForInput = (date: Date): string => {
  const pad = (num: number) => num.toString().padStart(2, "0");
  const year = date.getFullYear();
  const month = pad(date.getMonth() + 1);
  const day = pad(date.getDate());
  return `${year}-${month}-${day}`;
};

const timeValue = computed({
  get: () => {
    if (!timeSelection.value) return "";
    return formatTimeForInput(timeSelection.value.time);
  },
  set: (value: string) => {
    if (!timeSelection.value || !value) return;
    try {
      const [hoursStr = "0", minutesStr = "0"] = value.split(":");
      const hours = parseInt(hoursStr, 10);
      const minutes = parseInt(minutesStr, 10);

      if (!isNaN(hours) && !isNaN(minutes)) {
        const newDate = new Date(timeSelection.value.time);
        newDate.setHours(hours, minutes);
        timeSelection.value = {
          ...timeSelection.value,
          time: newDate,
        };
      }
    } catch (e) {
      // ignore invalid time
    }
  },
});

const dateValue = computed({
  get: () => {
    if (!timeSelection.value) return "";
    return formatDateForInput(timeSelection.value.time);
  },
  set: (value: string) => {
    if (!timeSelection.value || !value) return;
    try {
      const newDate = new Date(value);
      if (!isNaN(newDate.getTime())) {
        // Preserve the current time when changing the date
        const currentTime = timeSelection.value.time;
        newDate.setHours(currentTime.getHours(), currentTime.getMinutes());
        timeSelection.value = {
          ...timeSelection.value,
          time: newDate,
        };
      }
    } catch (e) {
      // ignore invalid date
    }
  },
});

const isVisible = computed(() => timeSelection.value !== undefined);

// Minimum date (today)
const minDate = computed(() => {
  const today = new Date();
  return formatDateForInput(today);
});
</script>

<template>
  <div v-if="isVisible" class="mt-3 mb-4">
    <div class="grid grid-cols-2 gap-3">
      <!-- Time Input -->
      <div>
        <input
          type="time"
          v-model="timeValue"
          class="block w-full rounded-md border border-zinc-300 px-3 py-2 shadow-sm focus:border-blue-500 focus:ring-blue-500 sm:text-sm"
          step="600"
        />
      </div>

      <!-- Date Input -->
      <div>
        <input
          type="date"
          v-model="dateValue"
          :min="minDate"
          class="block w-full rounded-md border border-zinc-300 px-3 py-2 shadow-sm focus:border-blue-500 focus:ring-blue-500 sm:text-sm"
        />
      </div>
    </div>
  </div>
</template>
