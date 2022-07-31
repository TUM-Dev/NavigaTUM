<div id="view-404">
  <div class="toast toast-error">${{_.core_js.error.404}}$</div>
  <h5 style="margin-top: 25px">${{_.view_404.header}}$</h5>
  <p>${{_.view_404.description}}$</p>
  <button
    onclick="openFeedback('bug', `404 on ${window.location.href}`, '${{_.view_404.got_here}}$')"
    class="btn btn-link p-0"
    aria-label="Open the feedback-form"
  >
    ${{_.view_404.call_to_action}}$
  </button>
</div>
