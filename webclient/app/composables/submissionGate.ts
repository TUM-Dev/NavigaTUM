export type SubmissionBlock =
  | "submitting"
  | "already_succeeded"
  | "token_unavailable"
  | "incomplete_fields"
  | "consent_missing"
  | null;

export interface SubmissionGateState {
  submitting: boolean;
  succeeded: boolean;
  blockedByToken: boolean;
  draftReady: boolean;
  privacyChecked: boolean;
}

export function submissionBlock(state: SubmissionGateState): SubmissionBlock {
  if (state.submitting) return "submitting";
  if (state.succeeded) return "already_succeeded";
  if (state.blockedByToken) return "token_unavailable";
  if (!state.draftReady) return "incomplete_fields";
  if (!state.privacyChecked) return "consent_missing";
  return null;
}
