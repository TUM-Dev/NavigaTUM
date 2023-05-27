<script setup lang="ts">
import { useGlobalStore } from "@/stores/global";
import { watch, ref, reactive } from "vue";
import { Translation, useI18n } from "vue-i18n";
const { t } = useI18n({
  inheritLocale: true,
  useScope: "global",
});

const global = useGlobalStore();
const loading = ref(false);
const successUrl = ref("");
const error = reactive({
  message: "",
  blockSend: false,
});
const token = ref<Token | null>(null);
const privacyChecked = ref(false);
const deleteIssueRequested = ref(false);

type Token = {
  readonly created_at: number;
  readonly token: string;
};

// To work even when the rest of the JS code failed, the code for the
// feedback form is mostly separate from the rest of the codebase.
// It is only loaded when the feedback form is being opened.
import { setLocalStorageWithExpiry, getLocalStorageWithExpiry } from "@/composables/storage";

// Token are renewed after 6 hours here to be sure, even though they may be valid for longer on the server side.
assuereTokenValidity();
watch(() => global.feedback.open, assuereTokenValidity);
function assuereTokenValidity() {
  if (token.value === null) {
    token.value = getLocalStorageWithExpiry<Token | null>("feedback-token", null);
  }
  if (token.value === null || Date.now() - token.value.created_at > 1000 * 3600 * 6) {
    fetch(`/api/feedback/get_token`, { method: "POST" })
      .then((r) => {
        if (r.status === 201) {
          r.json()
            .then((j: Token) => {
              token.value = j;
              setLocalStorageWithExpiry("feedback-token", token.value, 6);
            })
            .catch((r) => {
              _showError(t("feedback.error.token_req_failed"), false);
              console.error(r);
            });
        } else if (r.status === 429) {
          _showError(t("feedback.error.429"), true);
        } else if (r.status === 503) {
          _showError(t("feedback.error.503"), true);
        } else {
          const unexpectedTS = t("feedback.error.token_unexpected_status");
          _showError(`${unexpectedTS}${r.status}`, true);
        }
      })
      .catch((r) => {
        _showError(t("feedback.error.token_req_failed"), false);
        console.error(r);
      });
  }
}

function _showError(msg = "", blockSend = false) {
  error.message = msg;
  error.blockSend = blockSend;
}

function closeForm() {
  global.feedback.open = false;
  successUrl.value = "";
  error.blockSend = false;
  error.message = "";
  document.body.classList.remove("no-scroll");
}

function mayCloseForm() {
  if (global.feedback.body.length === 0) closeForm();
}

function _send() {
  console.log("Sending feedback for token", token.value);
  const data = {
    token: token.value?.token,
    category: global.feedback.category,
    subject: global.feedback.subject,
    body: global.feedback.body,
    privacy_checked: privacyChecked.value,
    deletion_requested: deleteIssueRequested.value,
  };
  fetch(`/api/feedback/feedback`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(data),
  })
    .then((r) => {
      loading.value = false;
      if (r.status === 201) {
        localStorage.removeItem("feedback-coords");
        token.value = null;
        localStorage.removeItem("feedback-token");
        const e = new Event("storage");
        window.dispatchEvent(e);
        successUrl.value = r.text();
      } else if (r.status === 500) {
        const serverError = t("feedback.error.server_error");
        _showError(`${serverError} (${r.text()})`, false);
      } else if (r.status === 451) {
        _showError(t("feedback.error.privacy_not_checked"), false);
      } else if (r.status === 403) {
        localStorage.removeItem("feedback-token");
        token.value = null;
        const invalidTokenError = t("feedback.error.send_invalid_token");
        _showError(`${invalidTokenError} (${r.text()})`, false);
      } else {
        // we reset the token here to be sure that it is the cause of the error
        localStorage.removeItem("feedback-token");
        token.value = null;
        const unexpectedStatusError = t("feedback.error.send_unexpected_status");
        _showError(`${unexpectedStatusError}${r.status}`, false);
      }
    })
    .catch((r) => {
      loading.value = false;
      _showError(t("feedback.error.send_req_failed"), false);
      console.error(r);
    });
}

function sendForm() {
  if (token.value === null) {
    _showError(t("feedback.error.send_no_token"), true);
  } else if (global.feedback.subject.length < 3) {
    _showError(t("feedback.error.too_short_subject"));
  } else if (global.feedback.body.length < 10) {
    _showError(t("feedback.error.too_short_body"));
  } else if (!privacyChecked.value) {
    _showError(t("feedback.error.privacy_not_checked"));
  } else {
    loading.value = true;
    // Token may only be used after a short delay. In case that has not passed
    // yet, we wait until for a short time.
    if (Date.now() - token.value.created_at < 1000 * 10)
      window.setTimeout(_send, 1000 * 10 - (Date.now() - token.value.created_at));
    else _send();
  }
}
</script>

<template>
  <div class="modal active" id="feedback-modal" v-if="!successUrl">
    <div id="feedback-overlay modal-overlay" @click="mayCloseForm"></div>
    <div class="modal-container">
      <div class="modal-header">
        <button class="btn btn-clear float-right" aria-label="Close" @click="closeForm"></button>
        <div class="modal-title h5">{{ $t("feedback.title") }}</div>
      </div>
      <div class="modal-body">
        <div class="content">
          <div id="feedback-error">{{ error.message }}</div>
          <div class="form-group">
            <div id="feedback-coordinate-picker-helptext" class="d-none toast toast-primary">
              {{ $t("feedback.coordinatepicker.helptext.enter_serveral") }}<br />
              {{ $t("feedback.coordinatepicker.helptext.saved_for_12h") }}<br />
              {{ $t("feedback.coordinatepicker.helptext.limitation_to_coordinates") }}
            </div>
            <label class="form-label" for="feedback-subject"> {{ $t("feedback.subject") }}</label>
            <div class="input-group">
              <select
                class="form-select"
                id="feedback-category"
                v-bind:aria-label="$t('feedback.category')"
                v-model="global.feedback.category"
              >
                <option value="general">{{ $t("feedback.type.general") }}</option>
                <option value="bug">{{ $t("feedback.type.bug") }}</option>
                <option value="features">{{ $t("feedback.type.features") }}</option>
                <option value="search">{{ $t("feedback.type.search") }}</option>
                <option value="entry">{{ $t("feedback.type.entry") }}</option>
              </select>
              <input
                class="form-input"
                type="text"
                v-bind:placeholder="$t('feedback.subject')"
                v-model="global.feedback.subject"
                id="feedback-subject"
              />
            </div>
          </div>

          <div class="form-group">
            <div>
              <label class="form-label" for="feedback-body">
                {{ $t("feedback.message") }}
              </label>
              <button
                id="feedback-coordinate-picker"
                v-if="global.feedback.category === 'entry'"
                class="btn btn-sm btn-link"
              >
                {{ $t("feedback.coordinatepicker.title") }}
              </button>
            </div>
            <textarea
              class="form-input"
              id="feedback-body"
              v-bind:placeholder="$t('feedback.message')"
              v-model="global.feedback.body"
              rows="6"
            >
            </textarea>
            <p class="text-gray text-tiny">
              {{
                {
                  general: t("feedback.helptext.general"),
                  bug: t("feedback.helptext.bug"),
                  feature: t("feedback.helptext.features"),
                  search: t("feedback.helptext.search"),
                  entry: t("feedback.helptext.entry"),
                  other: t("feedback.helptext.other"), // This is only here to make the linter happy, backend uses "other" as a fallback if the category is not known
                }[global.feedback.category]
              }}
            </p>
          </div>

          <!-- only visible if called through a view, because then the context of the calling building is availible -->
          <div>
            <button id="feedback-coordinate-picker" class="btn btn-sm d-none">
              {{ $t("feedback.coordinatepicker.title") }}
            </button>
          </div>

          <div class="form-group">
            <label class="form-checkbox">
              <input type="checkbox" id="feedback-privacy" v-model="privacyChecked" />
              <i class="form-icon"></i>
              <b>
                <Translation keypath="feedback.public.agreement" tag="span">
                  <template v-slot:github_project_issues_url>
                    <a href="https://github.com/TUM-Dev/navigatum/issues" target="_blank">
                      {{ $t("feedback.public.github_project_issues") }}
                    </a>
                  </template>
                </Translation>
              </b>
              <br />
              <Translation keypath="feedback.public.disclaimer" tag="span">
                <template v-slot:github_site_policy_url>
                  <a href="https://docs.github.com/en/github/site-policy" target="_blank">
                    {{ $t("feedback.public.github_site_policy") }}
                  </a>
                </template>
                <template v-slot:imprint_url>
                  <a href="/about/impressum" target="_blank">{{ $t("feedback.public.imprint") }}</a>
                </template>
              </Translation>
              <Translation keypath="feedback.public.legal_fluff" tag="span">
                <template v-slot:tum_data_protection_url>
                  <a href="https://datenschutz.tum.de" target="_blank">datenschutz.tum.de</a>
                </template>
              </Translation>
            </label>
            <label class="form-checkbox" id="feedback-delete-label">
              <input type="checkbox" id="feedback-delete" v-model="deleteIssueRequested" />
              <i class="form-icon"></i> {{ $t("feedback.delete") }}
            </label>
          </div>

          <div class="float-right">
            <button class="btn" @click="closeForm">
              {{ $t("feedback.cancel") }}
            </button>
            <button
              class="btn btn-primary"
              id="feedback-send"
              @click="sendForm"
              v-bind:class="{ loading: loading }"
              v-bind="{ disabled: loading || error.blockSend }"
            >
              {{ $t("feedback.send") }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
  <div class="modal active" id="feedback-success-modal" v-if="successUrl">
    <div class="feedback-overlay modal-overlay" @click="closeForm"></div>
    <div class="modal-container">
      <div class="modal-header">
        <button class="btn btn-clear float-right" aria-label="Close" @click="closeForm"></button>
        <div class="modal-title h5">{{ $t("feedback.success.title") }}</div>
      </div>
      <div class="modal-body">
        <div class="content">
          <p>{{ $t("feedback.success.thank_you") }}</p>
          <p>
            {{ $t("feedback.success.response_at") }}
            <a id="feedback-success-url" class="btn-link" v-bind:href="successUrl">{{
              $t("feedback.success.this_issue")
            }}</a>
          </p>
          <div class="buttons">
            <button class="btn btn-primary" @click="closeForm">
              {{ $t("feedback.success.ok") }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style lang="scss" scoped>
@import "../assets/variables";

.modal {
  z-index: 3000;

  .modal-container {
    max-height: 95vh;
    box-shadow: $feedback-box-shadow;
  }

  label {
    width: fit-content;
    display: inline-block;
  }

  .btn {
    margin: 0 0.1em;
  }

  .feedback-overlay {
    background: $feedback-overlay-bg;
  }

  #feedback-error {
    color: $error-color;
  }

  .form-select {
    flex: none;
  }

  #feedback-body {
    min-width: 100%;
  }

  #feedback-coordinate-picker {
    float: right;
    margin-top: 0.5em;
  }

  #feedback-coordinate-picker-helptext {
    font-size: 14px;
  }
}
</style>
