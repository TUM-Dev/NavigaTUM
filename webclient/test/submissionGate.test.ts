import { describe, expect, it } from "vitest";
import { type SubmissionGateState, submissionBlock } from "../app/composables/submissionGate";

const ready: SubmissionGateState = {
  submitting: false,
  succeeded: false,
  blockedByToken: false,
  draftReady: true,
  privacyChecked: true,
};

describe("submissionBlock", () => {
  it("allows sending once every gate is satisfied", () => {
    expect(submissionBlock(ready)).toBeNull();
  });

  it.each([
    ["submitting", { submitting: true }, "submitting"],
    ["already succeeded", { succeeded: true }, "already_succeeded"],
    ["token unavailable", { blockedByToken: true }, "token_unavailable"],
    ["incomplete form", { draftReady: false }, "incomplete_fields"],
    ["consent missing", { privacyChecked: false }, "consent_missing"],
  ] as const)("blocks on %s", (_label, override, expected) => {
    expect(submissionBlock({ ...ready, ...override })).toBe(expected);
  });

  it("never silently disables for a user-actionable reason", () => {
    for (const draftReady of [true, false]) {
      for (const privacyChecked of [true, false]) {
        const block = submissionBlock({ ...ready, draftReady, privacyChecked });
        const expected = draftReady
          ? privacyChecked
            ? null
            : "consent_missing"
          : "incomplete_fields";
        expect(block).toBe(expected);
      }
    }
  });
});
