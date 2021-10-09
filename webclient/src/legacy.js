import "regenerator-runtime/runtime";

// For some reason this polyfill is not included automatically
if (typeof String.prototype.startsWith === 'undefined') {
  String.prototype.startsWith = function (needle) {
    return this.indexOf(needle) === 0;
  };
}

/*split*/
