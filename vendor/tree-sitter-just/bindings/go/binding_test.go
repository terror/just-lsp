package tree_sitter_just_test

import (
	"testing"

	tree_sitter "github.com/smacker/go-tree-sitter"
	"github.com/tree-sitter/tree-sitter-just"
)

func TestCanLoadGrammar(t *testing.T) {
	language := tree_sitter.NewLanguage(tree_sitter_just.Language())
	if language == nil {
		t.Errorf("Error loading Just grammar")
	}
}
