<template>
  <div class="toast toast-error">{{ $t("core_js.error.404") }}</div>
  <h5 style="margin-top: 25px">{{ $t("view_404.header") }}</h5>
  <p>{{ $t("view_404.description") }}</p>
  <button
    onclick="openFeedback('bug', `404 on ${window.location.href}`, {{ $t('view_404.got_here') }} )"
    class="btn btn-link p-0"
    aria-label="Open the feedback-form"
  >
    {{ $t("view_404.call_to_action") }}
  </button>
</template>
