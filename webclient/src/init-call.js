// This code is executed after the vendor js files (e.g. vue) are loaded.
// Might be removed in the future if inlining js is not helpful (inlined js is not deferred,
// so it is not sure that this call would be called after vendor scripts if it was inlined)
navigatum.init();
