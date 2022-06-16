// To work even when the rest of the JS code failed, the code for the
// feedback form is mostly seperate from the rest of the codebase.
// It is only loaded when the feedback form is being opened.

window.feedback = (() => {
  let token = null;

  function _requestPage(method, url, data, onsuccess, onerror) {
    const req = new XMLHttpRequest();
    req.open(method, window.encodeURI(url), true);
    req.onload = onsuccess(this);
    req.onerror = onerror(this);
    if (data === null) {
      req.send();
    } else {
      req.setRequestHeader("Content-Type", "application/json");
      req.send(data);
    }
  }

  function _showError(msg = "", blockSend = false) {
    document.getElementById("feedback-error").innerText = msg;
    document.getElementById("feedback-send").disabled = blockSend;
  }

  function _showLoading(isLoading) {
    if (isLoading) {
      document.getElementById("feedback-send").classList.add("loading");
      document.getElementById("feedback-send").disabled = true;
    } else {
      document.getElementById("feedback-send").classList.remove("loading");
      document.getElementById("feedback-send").disabled = false;
    }
  }

  function openForm(category = "general", subject = "", body = "") {
    document.getElementById("feedback-category").value = category;
    document.getElementById("feedback-subject").value = subject;
    document.getElementById("feedback-body").value = body;
    document.getElementById("feedback-privacy").checked = false;
    document.getElementById("feedback-delete").checked = false;

    _showError();
    _showLoading(false);

    document.getElementById("feedback-modal").classList.add("active");
    document.body.classList.add("no-scroll");

    // Token are renewed after 6 hours here to be sure, even though they may be valid
    // for longer on the server side.
    if (token === null && navigatum) {
      token = navigatum.getLocalStorageWithExpiry("feedback-token", null);
    }
    if (token === null || Date.now() - token.creation > 1000 * 3600 * 6) {
      _requestPage(
        "POST",
        "/* @echo api_prefix */feedback/get_token",
        null,
        (r) => {
          if (r.status === 201) {
            token = {
              creation: Date.now(),
              value: JSON.parse(r.response).token,
            };
            if (navigatum)
              navigatum.setLocalStorageWithExpiry("feedback-token", token, 6);
          } else if (r.status === 429) {
            _showError("${{_.feedback.error.429}}$", true);
          } else if (r.status === 503) {
            _showError("${{_.feedback.error.503}}$", true);
          } else {
            const unexpectedTokenError =
              "${{_.feedback.error.token_unexpected_status}}$";
            _showError(`${unexpectedTokenError}${r.status}`, true);
          }
        },
        (r) => {
          _showError("${{_.feedback.error.token_req_failed}}$", false);
          console.error(r);
        }
      );
    }
  }

  function updateFeedbackForm(
    category = document.getElementById("feedback-category").value
  ) {
    const helptextLUT = {
      general: "${{_.feedback.helptext.general}}$",
      bug: "${{_.feedback.helptext.bug}}$",
      features: "${{_.feedback.helptext.features}}$",
      search: "${{_.feedback.helptext.search}}$",
      entry: "${{_.feedback.helptext.entry}}$",
    };
    document.getElementById("feedback-helptext").innerText =
      helptextLUT[category];

    const coordinatePicker = document.getElementById(
      "feedback-coordinate-picker"
    );
    if (category === "entry") {
      coordinatePicker.classList.remove("d-none");
    } else {
      coordinatePicker.classList.add("d-none");
    }
  }

  function closeForm() {
    document
      .getElementById("feedback-coordinate-picker")
      .classList.add("d-none");
    document
      .getElementById("feedback-coordinate-picker-helptext")
      .classList.add("d-none");

    document.getElementById("feedback-modal").classList.remove("active");
    document
      .getElementById("feedback-success-modal")
      .classList.remove("active");

    document.body.classList.remove("no-scroll");
  }

  function mayCloseForm() {
    if (document.getElementById("feedback-body").value.length === 0)
      closeForm();
  }

  function _showSuccess(href) {
    document.getElementById("feedback-modal").classList.remove("active");
    document.getElementById("feedback-success-modal").classList.add("active");
    document.getElementById("feedback-success-url").setAttribute("href", href);
  }

  function _send() {
    const category = document.getElementById("feedback-category").value;
    const subject = document.getElementById("feedback-subject").value;
    const body = document.getElementById("feedback-body").value;
    const privacy = document.getElementById("feedback-privacy").checked;
    const deleteIssue = document.getElementById("feedback-delete").checked;

    _requestPage(
      "POST",
      "/* @echo api_prefix */feedback/feedback",
      JSON.stringify({
        token: token.value,
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
          const serverError = "${{_.feedback.error.server_error}}$";
          _showError(`${serverError} (${r.responseText})`, false);
        } else if (r.status === 451) {
          _showError("${{_.feedback.error.privacy_not_checked}}$", false);
        } else if (r.status === 403) {
          localStorage.removeItem("feedback-token");
          token = null;
          const invalidTokenError = "${{_.feedback.error.send_invalid_token}}$";
          _showError(`${invalidTokenError} (${r.responseText})`, false);
        } else {
          const unexpectedStatusError =
            "${{_.feedback.error.send_invalid_token}}$";
          _showError(`${unexpectedStatusError}${r.status}`, false);
        }
      },
      (r) => {
        _showLoading(false);
        _showError("${{_.feedback.error.send_req_failed}}$");
        console.error(r);
      }
    );
  }

  function sendForm() {
    if (token === null) {
      _showError("${{_.feedback.error.send_no_token}}$", true);
    } else if (document.getElementById("feedback-subject").value.length < 3) {
      _showError("${{_.feedback.error.too_short_subject}}$");
    } else if (document.getElementById("feedback-body").value.length < 10) {
      _showError("${{_.feedback.error.too_short_body}}$");
    } else if (document.getElementById("feedback-privacy").checked !== true) {
      _showError("${{_.feedback.error.privacy_not_checked}}$");
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
    .addEventListener("click", closeForm, false);
  document
    .getElementById("feedback-close")
    .addEventListener("click", closeForm, false);
  document
    .getElementById("feedback-overlay")
    .addEventListener("click", mayCloseForm, false);

  document
    .getElementById("feedback-close-2")
    .addEventListener("click", closeForm, false);
  document
    .getElementById("feedback-overlay-2")
    .addEventListener("click", closeForm, false);

  document
    .getElementById("feedback-category")
    .addEventListener("change", (e) => updateFeedbackForm(e.value), false);

  document
    .getElementById("feedback-send")
    .addEventListener("click", sendForm, false);

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
