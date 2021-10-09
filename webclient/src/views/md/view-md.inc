<div id="view-md" v-if="content">
  <md-content :content="content"></md-content>

  <!-- This content is here to not purge the spectre css classes -->
  <template v-if="false">
    <pre class="code" data-lang="HTML"><code></code></pre>
  </template>
</div>
