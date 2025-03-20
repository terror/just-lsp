#include "tree_sitter/api.h"
#include <stdio.h>
#include <stdlib.h>

TSLanguage *tree_sitter_just(void);

int LLVMFuzzerTestOneInput(const uint8_t *data, const size_t len) {
  TSParser *parser = ts_parser_new();
  ts_parser_set_language(parser, tree_sitter_just());

  // Build a syntax tree based on source code stored in a string.
  TSTree *tree = ts_parser_parse_string(parser, NULL, (const char *)data, len);
  // Free all of the heap-allocated memory.
  ts_tree_delete(tree);
  ts_parser_delete(parser);
  return 0;
}
