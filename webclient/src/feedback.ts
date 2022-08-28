// To work even when the rest of the JS code failed, the code for the
// feedback form is mostly seperate from the rest of the codebase.
// It is only loaded when the feedback form is being opened.
import {
  setLocalStorageWithExpiry,
  getLocalStorageWithExpiry,
} from "@/utils/storage";

type Token = {
  readonly creation: number;
  readonly value: string;
};

window.feedback = (() => {
  let token: Token | null = null;
  // form-controls
  const feedbackSend = document.getElementById(
    "feedback-send"
  ) as HTMLButtonElement;
  const feedbackError = document.getElementById(
    "feedback-error"
  ) as HTMLDivElement;
  const feedbackModal = document.getElementById(
    "feedback-modal"
  ) as HTMLDivElement;
  const feedbackSuccessModal = document.getElementById(
    "feedback-success-modal"
  ) as HTMLDivElement;
  // form
  const feedbackCategory = document.getElementById(
    "feedback-category"
  ) as HTMLSelectElement;
  const feedbackSubject = document.getElementById(
    "feedback-subject"
  ) as HTMLInputElement;
  const feedbackBody = document.getElementById(
    "feedback-body"
  ) as HTMLTextAreaElement;
  const feedbackPrivacy = document.getElementById(
    "feedback-privacy"
  ) as HTMLInputElement;
  const feedbackDelete = document.getElementById(
    "feedback-delete"
  ) as HTMLInputElement;
  // coordinate picker
  const feedbackCoordinatePicker = document.getElementById(
    "feedback-coordinate-picker"
  ) as HTMLButtonElement;
  const feedbackCoordinatePickerHelp = document.getElementById(
    "feedback-coordinate-picker-helptext"
  ) as HTMLDivElement;

  function _requestPage(
    method: string,
    url: string,
    data: any,
    onsuccess,
    onerror
  ) {
    const req = new XMLHttpRequest();
    req.open(method, window.encodeURI(url), true);
    req.onload = onsuccess(req);
    req.onerror = onerror(req);
    if (data === null) {
      req.send();
    } else {
      req.setRequestHeader("Content-Type", "application/json");
      req.send(data);
    }
  }

  function _showError(msg = "", blockSend = false) {
    feedbackError.innerText = msg;
    feedbackSend.disabled = blockSend;
  }

  function _showLoading(isLoading: boolean) {
    if (isLoading) {
      feedbackSend.classList.add("loading");
    } else {
      feedbackSend.classList.remove("loading");
    }
    feedbackSend.disabled = isLoading;
  }

  function openForm(category = "general", subject = "", body = "") {
    feedbackCategory.value = category;
    feedbackSubject.value = subject;
    feedbackBody.value = body;
    feedbackPrivacy.checked = false;
    feedbackDelete.checked = false;

    _showError();
    _showLoading(false);

    feedbackModal.classList.add("active");
    document.body.classList.add("no-scroll");

    // Token are renewed after 6 hours here to be sure, even though they may be valid
    // for longer on the server side.
    if (token === null) {
      token = getLocalStorageWithExpiry("feedback-token", null);
    }
    if (token === null || Date.now() - token.creation > 1000 * 3600 * 6) {
      _requestPage(
        "POST",
        `/api/feedback/get_token`,
        null,
        (r) => {
          if (r.status === 201) {
            token = {
              creation: Date.now(),
              value: JSON.parse(r.response).token,
            };
            setLocalStorageWithExpiry("feedback-token", token, 6);
          } else if (r.status === 429) {
            _showError("{{ $t('feedback.error.429') }}", true);
          } else if (r.status === 503) {
            _showError("{{ $t('feedback.error.503') }}", true);
          } else {
            const unexpectedTokenError =
              "{{ $t('feedback.error.token_unexpected_status') }}";
            _showError(`${unexpectedTokenError}${r.status}`, true);
          }
        },
        (r) => {
          _showError("{{ $t('feedback.error.token_req_failed') }}", false);
          console.error(r);
        }
      );
    }
  }

  function updateFeedbackForm(category = feedbackCategory.value) {
    const helptextLUT = {
      general: "{{ $t('feedback.helptext.general') }}",
      bug: "{{ $t('feedback.helptext.bug') }}",
      features: "{{ $t('feedback.helptext.features') }}",
      search: "{{ $t('feedback.helptext.search') }}",
      entry: "{{ $t('feedback.helptext.entry') }}",
    };
    document.getElementById("feedback-helptext")!.innerText =
      helptextLUT[category];

    if (category === "entry")
      feedbackCoordinatePicker.classList.remove("d-none");
    else feedbackCoordinatePicker.classList.add("d-none");
  }

  function closeForm() {
    feedbackCoordinatePicker.classList.add("d-none");
    feedbackCoordinatePickerHelp.classList.add("d-none");

    feedbackModal.classList.remove("active");
    feedbackSuccessModal.classList.remove("active");

    document.body.classList.remove("no-scroll");
  }

  function mayCloseForm() {
    if (feedbackBody.value.length === 0) closeForm();
  }

  function _showSuccess(href: string) {
    feedbackModal.classList.remove("active");
    feedbackSuccessModal.classList.add("active");
    document.getElementById("feedback-success-url")?.setAttribute("href", href);
  }

  function _send() {
    const category = feedbackCategory.value;
    const subject = feedbackSubject.value;
    const body = feedbackBody.value;
    const privacy: boolean = feedbackPrivacy.checked;
    const deleteIssue: boolean = feedbackDelete.checked;

    _requestPage(
      "POST",
      `/api/feedback/feedback`,
      JSON.stringify({
        token: token!.value,
        category: category,
        subject: subject,
        body: body,
        privacy_checked: privacy,
        delete_issue_requested: deleteIssue,
      }),
      (r) => {
        _showLoading(false);
        if (r.status === 201) {
          localStorage.removeItem("coordinate-feedback");
          token = null;
          localStorage.removeItem("feedback-token");
          const e = new Event("storage");
          window.dispatchEvent(e);
          _showSuccess(r.responseText);
        } else if (r.status === 500) {
          const serverError = "{{ $t('feedback.error.server_error') }}";
          _showError(`${serverError} (${r.responseText})`, false);
        } else if (r.status === 451) {
          _showError("{{ $t('feedback.error.privacy_not_checked') }}", false);
        } else if (r.status === 403) {
          localStorage.removeItem("feedback-token");
          token = null;
          const invalidTokenError = $t("feedback.error.send_invalid_token");
          _showError(`${invalidTokenError} (${r.responseText})`, false);
        } else {
          const unexpectedStatusError =
            "{{ $t('feedback.error.send_invalid_token') }}";
          _showError(`${unexpectedStatusError}${r.status}`, false);
        }
      },
      (r) => {
        _showLoading(false);
        _showError("{{ $t('feedback.error.send_req_failed') }}");
        console.error(r);
      }
    );
  }

  function sendForm() {
    if (token === null) {
      _showError($t("feedback.error.send_no_token"), true);
    } else if (feedbackSubject.value.length < 3) {
      _showError("{{ $t('feedback.error.too_short_subject') }}");
    } else if (feedbackBody.value.length < 10) {
      _showError("{{ $t('feedback.error.too_short_body') }}");
    } else if (!feedbackPrivacy.checked) {
      _showError("{{ $t('feedback.error.privacy_not_checked') }}");
    } else {
      _showLoading(true);
      // Token may only be used after a short delay. In case that has not passed
      // yet, we wait until for a short time.
      if (Date.now() - token.creation < 1000 * 10) {
        window.setTimeout(_send, 1000 * 10 - (Date.now() - token.creation));
      } else {
        _send();
      }
    }
  }

  document
    .getElementById("feedback-cancel")
    ?.addEventListener("click", closeForm, false);
  document
    .getElementById("feedback-close")
    ?.addEventListener("click", closeForm, false);
  document
    .getElementById("feedback-overlay")
    ?.addEventListener("click", mayCloseForm, false);

  document
    .getElementById("feedback-close-2")
    ?.addEventListener("click", closeForm, false);
  document
    .getElementById("feedback-overlay-2")
    ?.addEventListener("click", closeForm, false);

  feedbackCategory.addEventListener(
    "change",
    (e) => updateFeedbackForm(e.value),
    false
  );

  feedbackSend.addEventListener("click", sendForm, false);

  /* global feedbackPreload */
  if (feedbackPreload) {
    openForm(
      feedbackPreload.category,
      feedbackPreload.subject,
      feedbackPreload.body
    );
    updateFeedbackForm(feedbackPreload.category);
  }

  return {
    openForm: openForm,
    closeForm: closeForm,
    updateFeedbackForm: updateFeedbackForm,
  };
})();
