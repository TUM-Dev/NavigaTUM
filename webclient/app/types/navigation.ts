// Shared types for navigation components and pages

export interface TimeSelection {
  type: "depart_at" | "arrive_by";
  time: Date;
}
