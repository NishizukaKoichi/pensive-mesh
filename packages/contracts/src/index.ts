export const protocols = {
  contextPack: "pensive-context-pack/1",
  memoryEvent: "pensive-memory-event/1",
  ritual: "pensive-ritual/1",
  simworld: "pensive-simworld/1",
  spellTicket: "pensive-spell-ticket/1",
  sync: "pensive-sync/1",
} as const;

export type Sensitivity =
  "PERSONAL" | "SENSITIVE" | "HIGHLY_SENSITIVE" | "SECRET";
export type ReviewState =
  | "CANDIDATE"
  | "ACCEPTED"
  | "REJECTED"
  | "DISPUTED"
  | "SUPERSEDED"
  | "EXPIRED"
  | "REVOKED"
  | "ORPHANED";
