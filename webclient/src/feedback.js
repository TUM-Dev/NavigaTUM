// To work even when the rest of the JS code failed, the code for the
// feedback form is mostly seperate from the rest of the codebase.
// It is only loaded when the feedback form is being opened.

const feedback = (function () {
  let token = null;

  function requestPage(method, url, data, onsuccess, onerror) {
    const req = new XMLHttpRequest();
    req.open(method, window.encodeURI(url), true);
    req.onload = function () {
      onsuccess(this);
    };
    req.onerror = function () {
      onerror(this);
    };
    if (data === null) {
      req.send();
    } else {
      req.setRequestHeader("Content-Type", "application/json");
      req.send(data);
    }
  }

  function showError(msg, blockSend) {
    msg = msg || "";
    document.getElementById("feedback-error").innerText = msg;
    document.getElementById("feedback-send").disabled = blockSend || false;
  }

  function showLoading(isLoading) {
    if (isLoading) {
      document.getElementById("feedback-send").classList.add("loading");
      document.getElementById("feedback-send").disabled = true;
    } else {
      document.getElementById("feedback-send").classList.remove("loading");
      document.getElementById("feedback-send").disabled = false;
    }
  }

  function openForm(category, subject, body) {
    category = category || "general";
    subject = subject || "";
    body = body || "";

    document.getElementById("feedback-category").value = category;
    document.getElementById("feedback-subject").value = subject;
    document.getElementById("feedback-body").value = body;
    document.getElementById("feedback-privacy").checked = false;
    document.getElementById("feedback-delete").checked = false;

    showError(false);
    showLoading(false);

    document.getElementById("feedback-modal").classList.add("active");
    document.body.classList.add("no-scroll");

    // Token are renewed after 6 hours here to be sure, even though they may be valid
    // for longer on the server side.
    if (token === null && navigatum) {
      token = navigatum.getLocalStorageWithExpiry("feedback-token", null);
    }
    if (token === null || Date.now() - token.creation > 1000 * 3600 * 6) {
      requestPage(
        "POST",
        "/* @echo api_prefix */feedback/get_token",
        null,
        function (r) {
          if (r.status === 201) {
            token = {
              creation: Date.now(),
              value: JSON.parse(r.response).token,
            };
            if (navigatum)
              navigatum.setLocalStorageWithExpiry("feedback-token", token, 6);
          } else if (r.status === 429) {
            showError("${{_.feedback.error.429}}$", true);
          } else if (r.status === 503) {
            showError("${{_.feedback.error.503}}$", true);
          } else {
            const unexpectedTokenError =
              "${{_.feedback.error.token_unexpected_status}}$";
            showError(`${unexpectedTokenError}${r.status}`, true);
          }
        },
        function (r) {
          showError("${{_.feedback.error.token_req_failed}}$");
          console.log(r);
        }
      );
    }
  }

  function updateFeedbackForm(category) {
    if (category === undefined)
      category = document.getElementById("feedback-category").value;

    const helptextLUT = {
      general: "${{_.feedback.helptext.general}}$",
      bug: "${{_.feedback.helptext.bug}}$",
      features: "${{_.feedback.helptext.features}}$",
      search: "${{_.feedback.helptext.search}}$",
      entry: "${{_.feedback.helptext.entry}}$",
    };
    document.getElementById("feedback-helptext").innerText = helptextLUT[category];

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

  function sendForm() {
    if (token === null) {
      showError("${{_.feedback.error.send_no_token}}$", true);
    } else if (document.getElementById("feedback-subject").value.length < 3) {
      showError("${{_.feedback.error.too_short_subject}}$");
    } else if (document.getElementById("feedback-body").value.length < 10) {
      showError("${{_.feedback.error.too_short_body}}$");
    } else if (document.getElementById("feedback-privacy").checked !== true) {
      showError("${{_.feedback.error.privacy_not_checked}}$");
    } else {
      showLoading(true);
      // Token may only be used after a short delay. In case that has not passed
      // yet, we wait until for a short time.
      if (Date.now() - token.creation < 1000 * 10) {
        window.setTimeout(send, 1000 * 10 - (Date.now() - token.creation));
      } else {
        send();
      }
    }
  }

  function showSuccess(href) {
    document.getElementById("feedback-modal").classList.remove("active");
    document.getElementById("feedback-success-modal").classList.add("active");
    document.getElementById("feedback-success-url").setAttribute("href", href);
  }

  function send() {
    const category = document.getElementById("feedback-category").value;
    const subject = document.getElementById("feedback-subject").value;
    const body = document.getElementById("feedback-body").value;
    const privacy = document.getElementById("feedback-privacy").checked;
    const deleteIssue = document.getElementById("feedback-delete").checked;

    requestPage(
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
      function (r) {
        showLoading(false);
        if (r.status === 201) {
          localStorage.removeItem("coordinate-feedback");
          token = null;
          localStorage.removeItem("feedback-token");
          const e = new Event("storage");
          window.dispatchEvent(e);
          showSuccess(r.responseText);
        } else if (r.status === 500) {
          const serverError = "${{_.feedback.error.server_error}}$";
          showError(`${serverError} (${r.responseText})`, false);
        } else if (r.status === 451) {
          showError("${{_.feedback.error.privacy_not_checked}}$", false);
        } else if (r.status === 403) {
          localStorage.removeItem("feedback-token");
          token = null;
          const invalidTokenError = "${{_.feedback.error.send_invalid_token}}$";
          showError(`${invalidTokenError} (${r.responseText})`, false);
        } else {
          const unexpectedStatusError =
            "${{_.feedback.error.send_invalid_token}}$";
          showError(`${unexpectedStatusError}${r.status}`, false);
        }
      },
      function (r) {
        showLoading(false);
        showError("${{_.feedback.error.send_req_failed}}$");
        console.log(r);
      }
    );
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

  document.getElementById("feedback-category").addEventListener(
    "change",
    function (e) {
      updateFeedbackForm(e.value);
    },
    false
  );

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
