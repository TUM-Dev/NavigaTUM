import "regenerator-runtime/runtime";

/* eslint no-extend-native: "off" */
// For some reason this polyfill is not included automatically
if (typeof String.prototype.startsWith === "undefined") {
  String.prototype.startsWith = function (needle) {
    return this.indexOf(needle) === 0;
  };
}

/* split */
// This comment is here to separate these polyfills and the ones provided by babel from the rest of the code concatenated to this file to make babel work
